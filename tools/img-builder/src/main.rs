mod fat32;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_ESP: &str = "build/esp";
const DEFAULT_IMAGE: &str = "build/aether.img";
const DEFAULT_SIZE_MB: u64 = 64;
const MIN_SIZE_MB: u64 = 16;

const REQUIRED_PATHS: &[&str] = &["EFI/BOOT/BOOTX64.EFI", "aether/kernel.elf"];

fn main() {
    if let Err(err) = run() {
        eprintln!("aether-img-builder: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Ok(());
    };

    match command.as_str() {
        "verify" => {
            let esp = next_path(&mut args, DEFAULT_ESP);
            verify_esp(&esp)
        }
        "info" => {
            let esp = next_path(&mut args, DEFAULT_ESP);
            info_esp(&esp)
        }
        "build" => {
            let esp = next_path(&mut args, DEFAULT_ESP);
            let output = next_path(&mut args, DEFAULT_IMAGE);
            let size_mb = parse_size_mb_flag(&mut args).unwrap_or(DEFAULT_SIZE_MB);
            build_image(&esp, &output, size_mb)
        }
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(())
        }
        other => Err(format!("unknown command: {other}")),
    }
}

fn next_path(args: &mut impl Iterator<Item = String>, default: &str) -> PathBuf {
    args.next().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(default))
}

fn print_usage() {
    eprintln!(
        "Usage:
  aether-img-builder verify [ESP]
  aether-img-builder info [ESP]
  aether-img-builder build [ESP] [OUTPUT] [--size-mb N]
  aether-img-builder help"
    );
}

fn parse_size_mb_flag(args: &mut impl Iterator<Item = String>) -> Option<u64> {
    let mut size_mb = None;
    while let Some(arg) = args.next() {
        if arg == "--size-mb" {
            size_mb = args.next()?.parse().ok();
        }
    }
    size_mb
}

fn verify_esp(esp: &Path) -> Result<(), String> {
    if !esp.is_dir() {
        return Err(format!("ESP directory not found: {}", esp.display()));
    }

    let mut missing = Vec::new();
    for rel in REQUIRED_PATHS {
        if !esp.join(normalize_rel(rel)).is_file() {
            missing.push(*rel);
        }
    }

    if missing.is_empty() {
        println!("ESP OK: {}", esp.display());
        for rel in REQUIRED_PATHS {
            let path = esp.join(normalize_rel(rel));
            let size = fs::metadata(&path).map_err(|e| e.to_string())?.len();
            println!("  {rel} ({size} bytes)");
        }
        Ok(())
    } else {
        Err(format!(
            "ESP missing required files under {}:\n  {}",
            esp.display(),
            missing.join("\n  ")
        ))
    }
}

fn info_esp(esp: &Path) -> Result<(), String> {
    if !esp.is_dir() {
        return Err(format!("ESP directory not found: {}", esp.display()));
    }

    println!("ESP: {}", esp.display());
    let (files, bytes) = count_tree(esp)?;
    println!("  files: {files}");
    println!("  bytes: {bytes}");
    for rel in REQUIRED_PATHS {
        let path = esp.join(normalize_rel(rel));
        let status = if path.is_file() { "present" } else { "MISSING" };
        println!("  {rel}: {status}");
    }
    Ok(())
}

fn count_tree(root: &Path) -> Result<(u64, u64), String> {
    let mut files = 0u64;
    let mut bytes = 0u64;
    walk(root, &mut |path| {
        if path.is_file() {
            files += 1;
            bytes += fs::metadata(path).map_err(|e| e.to_string())?.len();
        }
        Ok(())
    })?;
    Ok((files, bytes))
}

fn build_image(esp: &Path, output: &Path, size_mb: u64) -> Result<(), String> {
    verify_esp(esp)?;
    if size_mb < MIN_SIZE_MB {
        return Err(format!("size_mb must be at least {MIN_SIZE_MB}"));
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }

    let size_bytes =
        size_mb.checked_mul(1024 * 1024).ok_or_else(|| "image size overflow".to_string())?;

    let mut entries = Vec::new();
    walk(esp, &mut |path| {
        if path.is_file() {
            let rel = path.strip_prefix(esp).map_err(|_| "path outside ESP".to_string())?;
            let rel_unix = rel.to_string_lossy().replace('\\', "/");
            let data = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
            entries.push((rel_unix, data));
        }
        Ok(())
    })?;

    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let file_refs: Vec<(&str, Vec<u8>)> =
        entries.iter().map(|(name, data)| (name.as_str(), data.clone())).collect();
    fat32::build_image(output, size_bytes, &file_refs)?;

    let final_size = fs::metadata(output).map_err(|e| e.to_string())?.len();
    println!("Image ready: {} ({final_size} bytes)", output.display());
    println!("  QEMU: -drive if=none,format=raw,file={}", output.display());
    Ok(())
}

fn normalize_rel(rel: &str) -> PathBuf {
    PathBuf::from(rel.replace('/', std::path::MAIN_SEPARATOR_STR))
}

fn walk(dir: &Path, visit: &mut dyn FnMut(&Path) -> Result<(), String>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        visit(&path)?;
        if path.is_dir() {
            walk(&path, visit)?;
        }
    }
    Ok(())
}
