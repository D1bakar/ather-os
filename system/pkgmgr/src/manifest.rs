//! Package manifest format (`aether.toml`).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::{PkgError, PkgResult};

/// Current manifest schema version.
pub const MANIFEST_VERSION: u32 = 1;

/// Parsed package manifest.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// Schema version; must equal [`MANIFEST_VERSION`].
    #[serde(rename = "manifest-version")]
    pub manifest_version: u32,
    /// Core package metadata.
    pub package: PackageMeta,
    /// Files installed from the package payload.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<FileEntry>,
    /// Runtime dependencies keyed by package name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, String>,
}

/// Package identity and descriptive metadata.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PackageMeta {
    /// Unique package name (lowercase alphanumeric plus `-` / `_`).
    pub name: String,
    /// Semantic version string (`major.minor.patch`).
    pub version: String,
    /// One-line summary.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Target architecture (e.g. `x86_64`).
    pub architecture: String,
    /// SPDX license identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Package maintainer contact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<String>,
}

/// A single file mapping from payload path to install destination.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileEntry {
    /// Path inside the package payload archive.
    pub source: String,
    /// Absolute install path on the target system (e.g. `/usr/bin/app`).
    pub dest: String,
    /// Unix permission bits (e.g. `0o755`).
    #[serde(default = "default_file_mode")]
    pub mode: u32,
}

const fn default_file_mode() -> u32 {
    0o644
}

impl Manifest {
    /// Parses manifest TOML and validates required fields.
    pub fn parse(toml_text: &str) -> PkgResult<Self> {
        let manifest: Self =
            toml::from_str(toml_text).map_err(|err| PkgError::Parse(alloc::format!("{err}")))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Serializes the manifest to canonical TOML (deterministic key order via `BTreeMap`).
    pub fn to_toml(&self) -> PkgResult<String> {
        toml::to_string_pretty(self).map_err(|err| PkgError::Parse(alloc::format!("{err}")))
    }

    /// Validates manifest invariants without re-parsing.
    pub fn validate(&self) -> PkgResult<()> {
        if self.manifest_version != MANIFEST_VERSION {
            return Err(PkgError::InvalidManifest("unsupported manifest-version"));
        }

        validate_package_name(&self.package.name)?;
        validate_version(&self.package.version)?;

        if self.package.architecture.is_empty() {
            return Err(PkgError::InvalidManifest("package.architecture is required"));
        }

        for file in &self.files {
            if file.source.is_empty() || file.dest.is_empty() {
                return Err(PkgError::InvalidManifest("file source and dest are required"));
            }
            if !file.dest.starts_with('/') {
                return Err(PkgError::InvalidManifest("file dest must be absolute"));
            }
        }

        Ok(())
    }

    /// Returns the canonical package identifier (`name` + `@` + `version`).
    #[must_use]
    pub fn id(&self) -> String {
        alloc::format!("{}@{}", self.package.name, self.package.version)
    }
}

fn validate_package_name(name: &str) -> PkgResult<()> {
    if name.is_empty() {
        return Err(PkgError::InvalidManifest("package.name is required"));
    }

    let valid = name
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_'));

    if !valid {
        return Err(PkgError::InvalidManifest(
            "package.name must be lowercase alphanumeric with '-' or '_'",
        ));
    }

    Ok(())
}

fn validate_version(version: &str) -> PkgResult<()> {
    if version.is_empty() {
        return Err(PkgError::InvalidManifest("package.version is required"));
    }

    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(PkgError::InvalidManifest(
            "package.version must be semantic (major.minor.patch)",
        ));
    }

    if !parts.iter().all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit())) {
        return Err(PkgError::InvalidManifest(
            "package.version components must be non-negative integers",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
manifest-version = 1

[package]
name = "hello-world"
version = "1.0.0"
description = "Demo package"
architecture = "x86_64"
license = "MIT"

[[files]]
source = "bin/hello"
dest = "/usr/bin/hello"
mode = 755
"#;

    #[test]
    fn parse_valid_manifest() {
        let manifest = Manifest::parse(SAMPLE).expect("valid sample");
        assert_eq!(manifest.package.name, "hello-world");
        assert_eq!(manifest.id(), "hello-world@1.0.0");
        assert_eq!(manifest.files.len(), 1);
        assert_eq!(manifest.files[0].dest, "/usr/bin/hello");
    }

    #[test]
    fn reject_bad_version() {
        let bad = SAMPLE.replace("1.0.0", "1.0");
        let err = Manifest::parse(&bad).unwrap_err();
        assert!(matches!(err, PkgError::InvalidManifest(_)));
    }
}
