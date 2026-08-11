//! Install and uninstall APIs.

use alloc::string::String;
use alloc::vec::Vec;

use crate::error::{PkgError, PkgResult};
use crate::package::Package;
use crate::signature::SignatureVerifier;

/// Configuration for [`PackageManager`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageManagerConfig {
    /// Root directory for installed state (database + staged files).
    pub state_root: String,
    /// Target filesystem root for file placement (host tests use a temp dir).
    pub install_root: String,
    /// When true, signature sidecars must be present and pass verification.
    pub require_signature: bool,
}

impl Default for PackageManagerConfig {
    fn default() -> Self {
        Self {
            state_root: "/var/lib/aether/pkg".into(),
            install_root: "/".into(),
            require_signature: false,
        }
    }
}

/// Metadata recorded for an installed package.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallRecord {
    /// Package name.
    pub name: String,
    /// Installed version.
    pub version: String,
    /// Canonical package id (`name@version`).
    pub id: String,
    /// Files installed on the target system.
    pub files: Vec<String>,
}

/// Host-side package manager with install/uninstall operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageManager {
    config: PackageManagerConfig,
    verifier: SignatureVerifier,
}

impl PackageManager {
    /// Creates a manager with the given configuration.
    #[must_use]
    pub fn new(config: PackageManagerConfig) -> Self {
        Self { config, verifier: SignatureVerifier::new() }
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub fn config(&self) -> &PackageManagerConfig {
        &self.config
    }

    /// Installs a package after optional signature verification and dependency checks.
    pub fn install(&self, package: &Package) -> PkgResult<InstallRecord> {
        if self.is_installed(package.name())? {
            return Err(PkgError::AlreadyInstalled);
        }

        self.check_dependencies(package)?;
        self.verify_package(package)?;

        let record = self.materialize(package)?;
        self.write_record(&record)?;
        Ok(record)
    }

    /// Removes a previously installed package by name.
    pub fn uninstall(&self, name: &str) -> PkgResult<InstallRecord> {
        let record = self.read_record(name)?;
        self.remove_files(&record)?;
        self.delete_record(name)?;
        Ok(record)
    }

    /// Lists installed package records.
    pub fn list_installed(&self) -> PkgResult<Vec<InstallRecord>> {
        let mut records = Vec::new();
        let db_dir = self.db_dir();

        #[cfg(feature = "std")]
        {
            if !std::path::Path::new(&db_dir).is_dir() {
                return Ok(records);
            }

            for entry in std::fs::read_dir(&db_dir)
                .map_err(|err| PkgError::Io(alloc::format!("read_dir {db_dir}: {err}")))?
            {
                let entry =
                    entry.map_err(|err| PkgError::Io(alloc::format!("read_dir entry: {err}")))?;
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }
                let text = std::fs::read_to_string(&path)
                    .map_err(|err| PkgError::Io(alloc::format!("read {path:?}: {err}")))?;
                records.push(parse_record_json(&text)?);
            }

            records.sort_by(|a, b| a.name.cmp(&b.name));
        }

        Ok(records)
    }

    /// Returns true when the named package has an install record.
    pub fn is_installed(&self, name: &str) -> PkgResult<bool> {
        Ok(self.read_record(name).is_ok())
    }

    fn verify_package(&self, package: &Package) -> PkgResult<()> {
        match (&package.signature, self.config.require_signature) {
            (Some(signature), true) => {
                self.verifier.verify(&package.manifest_bytes, &package.payload_bytes, signature)
            }
            (Some(signature), false) => signature.validate_structure(),
            (None, true) => Err(PkgError::InvalidSignature("signature required")),
            (None, false) => Ok(()),
        }
    }

    fn check_dependencies(&self, package: &Package) -> PkgResult<()> {
        for dependency in package.manifest.dependencies.keys() {
            if !self.is_installed(dependency)? {
                return Err(PkgError::UnsatisfiedDependency {
                    package: package.name().into(),
                    dependency: dependency.clone(),
                });
            }
        }
        Ok(())
    }

    fn materialize(&self, package: &Package) -> PkgResult<InstallRecord> {
        let mut installed_files = Vec::new();

        #[cfg(feature = "std")]
        {
            use std::path::{Component, Path, PathBuf};

            for file in &package.manifest.files {
                let source = parse_payload_file(&package.payload_bytes, &file.source)?;
                let dest = PathBuf::from(&self.config.install_root)
                    .join(Path::new(&file.dest).strip_prefix("/").unwrap_or(Path::new(&file.dest)));

                if dest.components().any(|component| matches!(component, Component::ParentDir)) {
                    return Err(PkgError::InvalidManifest("dest path escapes install root"));
                }

                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).map_err(|err| {
                        PkgError::Io(alloc::format!("create_dir_all {parent:?}: {err}"))
                    })?;
                }

                std::fs::write(&dest, &source)
                    .map_err(|err| PkgError::Io(alloc::format!("write {dest:?}: {err}")))?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = file.mode & 0o7777;
                    std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(mode))
                        .map_err(|err| PkgError::Io(alloc::format!("chmod {dest:?}: {err}")))?;
                }

                installed_files.push(file.dest.clone());
            }
        }

        Ok(InstallRecord {
            name: package.name().into(),
            version: package.version().into(),
            id: package.id(),
            files: installed_files,
        })
    }

    fn db_dir(&self) -> String {
        alloc::format!("{}/installed", self.config.state_root)
    }

    fn record_path(&self, name: &str) -> String {
        alloc::format!("{}/{}.json", self.db_dir(), name)
    }

    fn write_record(&self, record: &InstallRecord) -> PkgResult<()> {
        #[cfg(feature = "std")]
        {
            let path = self.record_path(&record.name);
            if let Some(parent) = std::path::Path::new(&path).parent() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    PkgError::Io(alloc::format!("create_dir_all {parent:?}: {err}"))
                })?;
            }
            let json = serialize_record_json(record)?;
            std::fs::write(&path, json)
                .map_err(|err| PkgError::Io(alloc::format!("write {path}: {err}")))?;
        }
        Ok(())
    }

    fn read_record(&self, name: &str) -> PkgResult<InstallRecord> {
        #[cfg(feature = "std")]
        {
            let path = self.record_path(name);
            let text = std::fs::read_to_string(&path)
                .map_err(|err| PkgError::Io(alloc::format!("read {path}: {err}")))?;
            parse_record_json(&text)
        }

        #[cfg(not(feature = "std"))]
        {
            let _ = name;
            Err(PkgError::NotInstalled)
        }
    }

    fn delete_record(&self, name: &str) -> PkgResult<()> {
        #[cfg(feature = "std")]
        {
            let path = self.record_path(name);
            std::fs::remove_file(&path).map_err(|_| PkgError::NotInstalled)?;
        }
        Ok(())
    }

    fn remove_files(&self, record: &InstallRecord) -> PkgResult<()> {
        #[cfg(feature = "std")]
        {
            for dest in &record.files {
                let path = std::path::PathBuf::from(&self.config.install_root).join(
                    std::path::Path::new(dest)
                        .strip_prefix("/")
                        .unwrap_or(std::path::Path::new(dest)),
                );
                if path.is_file() {
                    std::fs::remove_file(&path).map_err(|err| {
                        PkgError::Io(alloc::format!("remove_file {path:?}: {err}"))
                    })?;
                }
            }
        }
        Ok(())
    }
}

fn parse_payload_file(payload_bytes: &[u8], source: &str) -> PkgResult<Vec<u8>> {
    let mut offset = 0usize;
    while offset < payload_bytes.len() {
        let Some(nul) = payload_bytes[offset..].iter().position(|&b| b == 0) else {
            break;
        };
        let path_bytes = &payload_bytes[offset..offset + nul];
        offset += nul + 1;

        let Some(content_nul) = payload_bytes[offset..].iter().position(|&b| b == 0) else {
            break;
        };
        let content = payload_bytes[offset..offset + content_nul].to_vec();
        offset += content_nul + 1;

        let path = core::str::from_utf8(path_bytes)
            .map_err(|_| PkgError::InvalidPackage("payload path is not UTF-8"))?;
        if path == source {
            return Ok(content);
        }
    }

    Err(PkgError::InvalidPackage("payload file missing"))
}

fn serialize_record_json(record: &InstallRecord) -> PkgResult<String> {
    let files =
        record.files.iter().map(|file| alloc::format!("\"{file}\"")).collect::<Vec<_>>().join(",");
    Ok(alloc::format!(
        "{{\"name\":\"{}\",\"version\":\"{}\",\"id\":\"{}\",\"files\":[{}]}}",
        record.name,
        record.version,
        record.id,
        files
    ))
}

fn parse_record_json(text: &str) -> PkgResult<InstallRecord> {
    let text = text.trim();
    let name = extract_json_string_field(text, "name")?;
    let version = extract_json_string_field(text, "version")?;
    let id = extract_json_string_field(text, "id")?;
    let files = extract_json_string_array(text, "files")?;

    Ok(InstallRecord { name, version, id, files })
}

fn extract_json_string_field(text: &str, key: &str) -> PkgResult<String> {
    let needle = alloc::format!("\"{key}\":\"");
    let start =
        text.find(&needle).ok_or_else(|| PkgError::Parse(alloc::format!("missing field {key}")))?
            + needle.len();
    let rest = &text[start..];
    let end = rest
        .find('"')
        .ok_or_else(|| PkgError::Parse(alloc::format!("unterminated field {key}")))?;
    Ok(rest[..end].into())
}

fn extract_json_string_array(text: &str, key: &str) -> PkgResult<Vec<String>> {
    let needle = alloc::format!("\"{key}\":[");
    let start =
        text.find(&needle).ok_or_else(|| PkgError::Parse(alloc::format!("missing array {key}")))?
            + needle.len();
    let rest = &text[start..];
    let end = rest
        .find(']')
        .ok_or_else(|| PkgError::Parse(alloc::format!("unterminated array {key}")))?;
    let inner = rest[..end].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }

    inner.split(',').map(|item| parse_json_string(item.trim())).collect()
}

fn parse_json_string(raw: &str) -> PkgResult<String> {
    let trimmed = raw.trim().trim_end_matches(',').trim();
    let trimmed = trimmed.strip_prefix('"').ok_or(PkgError::Parse("json string".into()))?;
    let trimmed = trimmed.strip_suffix('"').ok_or(PkgError::Parse("json string".into()))?;
    Ok(trimmed.into())
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::manifest::Manifest;
    use crate::package::Package;

    fn sample_package(name: &str, payload: &[u8]) -> Package {
        let manifest = Manifest::parse(&alloc::format!(
            r#"
manifest-version = 1

[package]
name = "{name}"
version = "1.0.0"
architecture = "x86_64"

[[files]]
source = "bin/{name}"
dest = "/usr/bin/{name}"
"#
        ))
        .expect("manifest");

        let manifest_bytes = manifest.to_toml().expect("toml").into_bytes();
        let mut payload_bytes = Vec::new();
        payload_bytes.extend_from_slice(format!("bin/{name}").as_bytes());
        payload_bytes.push(0);
        payload_bytes.extend_from_slice(payload);
        payload_bytes.push(0);

        Package::from_parts(manifest_bytes, payload_bytes, None).expect("package")
    }

    #[test]
    fn install_and_uninstall_roundtrip() {
        let temp = std::env::temp_dir().join(format!("aether-pkgmgr-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).expect("temp dir");

        let config = PackageManagerConfig {
            state_root: temp.join("state").to_string_lossy().into(),
            install_root: temp.join("root").to_string_lossy().into(),
            require_signature: false,
        };

        let mgr = PackageManager::new(config);
        let package = sample_package("hello", b"#!/bin/sh\necho hi\n");
        let record = mgr.install(&package).expect("install");

        assert_eq!(record.name, "hello");
        assert!(mgr.is_installed("hello").expect("query"));

        let installed = temp.join("root/usr/bin/hello");
        assert!(installed.is_file());

        mgr.uninstall("hello").expect("uninstall");
        assert!(!mgr.is_installed("hello").expect("query"));
        assert!(!installed.exists());

        let _ = std::fs::remove_dir_all(&temp);
    }
}
