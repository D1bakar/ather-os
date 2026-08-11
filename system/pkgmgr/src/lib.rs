//! Package manager for Aether OS.
//!
//! Provides a TOML-based package manifest format, ed25519 signature verification
//! (stub by default; enable the `verify` feature for host-side crypto), and
//! install/uninstall APIs backed by a pluggable filesystem root.
//!
//! **Status:** M11 scaffold — host-buildable; no kernel or init integration yet.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

mod error;
mod install;
mod manifest;
mod package;
mod signature;

pub use error::{PkgError, PkgResult};
pub use install::{InstallRecord, PackageManager, PackageManagerConfig};
pub use manifest::{FileEntry, Manifest, PackageMeta, MANIFEST_VERSION};
pub use package::{Package, PackageLayout};
pub use signature::{
    Signature, SignatureVerifier, SignatureVerifyError, ED25519_PUBLIC_KEY_LEN,
    ED25519_SIGNATURE_LEN, SIGNATURE_VERSION,
};
