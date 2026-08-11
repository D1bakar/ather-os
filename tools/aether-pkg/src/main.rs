//! Host-side package manager CLI for Aether OS (M11).

use std::path::PathBuf;
use std::process::ExitCode;

use aether_pkgmgr::{Manifest, Package, PackageManager, PackageManagerConfig, SignatureVerifier};
use clap::{Parser, Subcommand};

/// Default state directory for installed package metadata.
const DEFAULT_STATE_ROOT: &str = "/var/lib/aether/pkg";

/// Default filesystem root for file placement.
const DEFAULT_INSTALL_ROOT: &str = "/";

/// Aether OS package manager.
#[derive(Parser, Debug)]
#[command(name = "aether-pkg", version, about = "Aether OS package manager")]
struct Cli {
    /// Directory for install state (metadata database).
    #[arg(long, global = true, default_value = DEFAULT_STATE_ROOT)]
    state_root: PathBuf,

    /// Target root for installed files.
    #[arg(long, global = true, default_value = DEFAULT_INSTALL_ROOT)]
    install_root: PathBuf,

    /// Require a valid ed25519 signature sidecar.
    #[arg(long, global = true)]
    require_signature: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Install a package from an `.apkg` directory bundle.
    Install {
        /// Path to the package directory.
        path: PathBuf,
    },
    /// Remove an installed package by name.
    Uninstall {
        /// Package name to remove.
        name: String,
    },
    /// List installed packages.
    List,
    /// Show manifest metadata for a package bundle.
    Show {
        /// Path to the package directory.
        path: PathBuf,
    },
    /// Verify manifest and optional signature sidecar.
    Verify {
        /// Path to the package directory.
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("aether-pkg: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let manager = PackageManager::new(PackageManagerConfig {
        state_root: cli.state_root.to_string_lossy().into(),
        install_root: cli.install_root.to_string_lossy().into(),
        require_signature: cli.require_signature,
    });

    match cli.command {
        Command::Install { path } => {
            let package = Package::load_from_dir(&path)?;
            let record = manager.install(&package)?;
            println!("installed {}", record.id);
        }
        Command::Uninstall { name } => {
            let record = manager.uninstall(&name)?;
            println!("removed {}", record.id);
        }
        Command::List => {
            let records = manager.list_installed()?;
            if records.is_empty() {
                println!("no packages installed");
            } else {
                for record in records {
                    println!("{} ({})", record.id, record.files.len());
                }
            }
        }
        Command::Show { path } => {
            let package = Package::load_from_dir(&path)?;
            print_manifest(&package.manifest);
        }
        Command::Verify { path } => {
            let package = Package::load_from_dir(&path)?;
            match package.signature {
                Some(ref signature) => {
                    SignatureVerifier::new().verify(
                        &package.manifest_bytes,
                        &package.payload_bytes,
                        signature,
                    )?;
                    println!("signature OK for {}", package.id());
                }
                None => {
                    package.manifest.validate()?;
                    println!("manifest OK for {} (unsigned)", package.id());
                }
            }
        }
    }

    Ok(())
}

fn print_manifest(manifest: &Manifest) {
    println!("{}", manifest.id());
    if !manifest.package.description.is_empty() {
        println!("  description: {}", manifest.package.description);
    }
    println!("  architecture: {}", manifest.package.architecture);
    if let Some(license) = &manifest.package.license {
        println!("  license: {license}");
    }
    if manifest.files.is_empty() {
        println!("  files: (none)");
    } else {
        println!("  files:");
        for file in &manifest.files {
            println!("    {} -> {} ({:#o})", file.source, file.dest, file.mode);
        }
    }
    if manifest.dependencies.is_empty() {
        println!("  dependencies: (none)");
    } else {
        println!("  dependencies:");
        for (name, constraint) in &manifest.dependencies {
            println!("    {name} {constraint}");
        }
    }
}
