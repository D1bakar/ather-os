//! On-disk package layout and loading.

use alloc::string::String;
use alloc::vec::Vec;

use crate::error::{PkgError, PkgResult};
use crate::manifest::Manifest;
use crate::signature::Signature;

/// Filenames inside an `.apkg` directory bundle.
pub const MANIFEST_FILE: &str = "aether.toml";
/// Signature sidecar filename.
pub const SIGNATURE_FILE: &str = "aether.sig";
/// Payload directory name.
pub const PAYLOAD_DIR: &str = "payload";

/// Loaded package contents ready for verification and installation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Package {
    /// Parsed manifest.
    pub manifest: Manifest,
    /// Raw manifest bytes (canonical input for signing).
    pub manifest_bytes: Vec<u8>,
    /// Opaque payload bytes (typically a tar archive in future milestones).
    pub payload_bytes: Vec<u8>,
    /// Optional signature sidecar.
    pub signature: Option<Signature>,
}

/// Logical layout of files within a package directory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageLayout;

impl PackageLayout {
    /// Returns standard manifest path relative to package root.
    #[must_use]
    pub const fn manifest_path() -> &'static str {
        MANIFEST_FILE
    }

    /// Returns standard signature path relative to package root.
    #[must_use]
    pub const fn signature_path() -> &'static str {
        SIGNATURE_FILE
    }

    /// Returns payload directory name relative to package root.
    #[must_use]
    pub const fn payload_dir() -> &'static str {
        PAYLOAD_DIR
    }
}

impl Package {
    /// Builds a package from in-memory parts (host tests and tooling).
    pub fn from_parts(
        manifest_bytes: Vec<u8>,
        payload_bytes: Vec<u8>,
        signature: Option<Signature>,
    ) -> PkgResult<Self> {
        let manifest_text = core::str::from_utf8(&manifest_bytes)
            .map_err(|_| PkgError::InvalidPackage("manifest is not valid UTF-8"))?;
        let manifest = Manifest::parse(manifest_text)?;

        Ok(Self { manifest, manifest_bytes, payload_bytes, signature })
    }

    /// Returns the package identifier (`name@version`).
    #[must_use]
    pub fn id(&self) -> String {
        self.manifest.id()
    }

    /// Returns the package name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.manifest.package.name
    }

    /// Returns the package version string.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.manifest.package.version
    }
}

#[cfg(feature = "std")]
impl Package {
    /// Loads a package directory bundle from disk.
    pub fn load_from_dir(root: &std::path::Path) -> PkgResult<Self> {
        let manifest_path = root.join(MANIFEST_FILE);
        let payload_path = root.join(PAYLOAD_DIR);
        let signature_path = root.join(SIGNATURE_FILE);

        let manifest_bytes = std::fs::read(&manifest_path)
            .map_err(|err| PkgError::Io(alloc::format!("read {manifest_path:?}: {err}")))?;
        let payload_bytes = read_payload_dir(&payload_path)?;
        let signature = if signature_path.is_file() {
            let sig_text = std::fs::read_to_string(&signature_path)
                .map_err(|err| PkgError::Io(alloc::format!("read {signature_path:?}: {err}")))?;
            Some(Signature::parse(&sig_text)?)
        } else {
            None
        };

        Self::from_parts(manifest_bytes, payload_bytes, signature)
    }
}

#[cfg(feature = "std")]
fn read_payload_dir(path: &std::path::Path) -> PkgResult<Vec<u8>> {
    if !path.is_dir() {
        return Err(PkgError::InvalidPackage("payload directory missing"));
    }

    let mut entries = Vec::new();
    collect_payload_entries(path, path, &mut entries)?;

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut out = Vec::new();
    for (relative, bytes) in entries {
        out.extend_from_slice(relative.as_bytes());
        out.push(0);
        out.extend_from_slice(&bytes);
        out.push(0);
    }

    Ok(out)
}

#[cfg(feature = "std")]
fn collect_payload_entries(
    root: &std::path::Path,
    current: &std::path::Path,
    out: &mut Vec<(String, Vec<u8>)>,
) -> PkgResult<()> {
    for entry in std::fs::read_dir(current)
        .map_err(|err| PkgError::Io(alloc::format!("read_dir {current:?}: {err}")))?
    {
        let entry = entry.map_err(|err| PkgError::Io(alloc::format!("read_dir entry: {err}")))?;
        let path = entry.path();
        if path.is_dir() {
            collect_payload_entries(root, &path, out)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| PkgError::InvalidPackage("payload path prefix"))?;
            let key = relative.to_string_lossy().replace('\\', "/");
            let bytes = std::fs::read(&path)
                .map_err(|err| PkgError::Io(alloc::format!("read {path:?}: {err}")))?;
            out.push((key, bytes));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Manifest;

    #[test]
    fn from_parts_roundtrip() {
        let manifest = Manifest::parse(
            r#"
manifest-version = 1

[package]
name = "demo"
version = "1.0.0"
architecture = "x86_64"
"#,
        )
        .expect("manifest");

        let manifest_bytes = manifest.to_toml().expect("toml").into_bytes();
        let package =
            Package::from_parts(manifest_bytes.clone(), vec![1, 2, 3], None).expect("package");

        assert_eq!(package.name(), "demo");
        assert_eq!(package.manifest_bytes, manifest_bytes);
    }
}
