//! Signed update manifest and verification skeleton.
//!
//! See [docs/updates/signed-verification.md](../../../docs/updates/signed-verification.md).

use crate::error::{UpdateError, UpdateErrorCode};
use crate::partition::BootSlot;

/// Supported signature algorithms for update manifests.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum SignatureAlgorithm {
    /// Ed25519 detached signature (preferred).
    Ed25519 = 1,
}

impl SignatureAlgorithm {
    /// Parses an algorithm from its numeric discriminator.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Ed25519),
            _ => None,
        }
    }

    /// Expected detached signature length in bytes.
    #[must_use]
    pub const fn signature_len(self) -> usize {
        match self {
            Self::Ed25519 => 64,
        }
    }
}

/// Kind of payload referenced by an update manifest.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum UpdatePayloadKind {
    /// Full kernel ELF image for the inactive slot.
    KernelElf = 1,
    /// Combined system image (kernel + initrd bundle) — future.
    SystemBundle = 2,
}

impl UpdatePayloadKind {
    /// Parses a payload kind from its numeric discriminator.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::KernelElf),
            2 => Some(Self::SystemBundle),
            _ => None,
        }
    }
}

/// Fixed-layout update manifest header (canonical fields for signing).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UpdateManifest {
    /// Magic bytes (`b"AETHUPD!"`).
    pub magic: [u8; 8],
    /// Manifest structure version (currently `1`).
    pub version: u32,
    /// Target A/B slot for this update.
    pub target_slot: BootSlot,
    /// Payload kind being delivered.
    pub payload_kind: UpdatePayloadKind,
    /// Signature algorithm used for `signature`.
    pub algorithm: SignatureAlgorithm,
    /// SHA-256 digest of the payload (32 bytes).
    pub payload_sha256: [u8; 32],
    /// Key identifier (hash prefix of trusted public key material).
    pub key_id: [u8; 8],
    /// Semantic version being installed (length in bytes).
    pub release_version_len: u8,
    /// NUL-padded release version label (e.g. `"0.2.0"`).
    pub release_version: [u8; 16],
    /// Detached signature over the canonical manifest bytes (excluding signature field).
    pub signature: [u8; 64],
}

/// Magic value for [`UpdateManifest`].
pub const UPDATE_MANIFEST_MAGIC: [u8; 8] = *b"AETHUPD!";

/// Current update manifest layout version.
pub const UPDATE_MANIFEST_VERSION: u32 = 1;

impl UpdateManifest {
    /// Creates an empty manifest placeholder for tests and tooling.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            magic: UPDATE_MANIFEST_MAGIC,
            version: UPDATE_MANIFEST_VERSION,
            target_slot: BootSlot::B,
            payload_kind: UpdatePayloadKind::KernelElf,
            algorithm: SignatureAlgorithm::Ed25519,
            payload_sha256: [0; 32],
            key_id: [0; 8],
            release_version_len: 0,
            release_version: [0; 16],
            signature: [0; 64],
        }
    }

    /// Validates structural fields (not cryptographic signature).
    #[must_use]
    pub fn validate_structure(&self) -> Result<(), UpdateError> {
        if self.magic != UPDATE_MANIFEST_MAGIC {
            return Err(UpdateError::new(UpdateErrorCode::InvalidManifest));
        }
        if self.version != UPDATE_MANIFEST_VERSION {
            return Err(UpdateError::new(UpdateErrorCode::InvalidManifest));
        }
        if SignatureAlgorithm::from_u8(self.algorithm as u8).is_none() {
            return Err(UpdateError::new(UpdateErrorCode::InvalidManifest));
        }
        if UpdatePayloadKind::from_u8(self.payload_kind as u8).is_none() {
            return Err(UpdateError::new(UpdateErrorCode::InvalidManifest));
        }
        Ok(())
    }

    /// Returns the release version label as a byte slice.
    #[must_use]
    pub fn release_version_bytes(&self) -> &[u8] {
        let len = usize::from(self.release_version_len.min(16));
        &self.release_version[..len]
    }

    /// Serializes canonical manifest bytes used as the signed message (stub layout).
    ///
    /// Excludes the `signature` field. Real implementation will use a stable
    /// canonical encoding (CBOR or fixed struct) documented in the ADR.
    #[must_use]
    pub fn canonical_bytes(&self) -> [u8; 64] {
        let mut buf = [0u8; 64];
        buf[..8].copy_from_slice(&self.magic);
        buf[8..12].copy_from_slice(&self.version.to_le_bytes());
        buf[12] = self.target_slot as u8;
        buf[13] = self.payload_kind as u8;
        buf[14] = self.algorithm as u8;
        buf[15] = self.release_version_len;
        buf[16..48].copy_from_slice(&self.payload_sha256);
        buf[48..56].copy_from_slice(&self.key_id);
        buf
    }
}

/// Policy for trusted signing keys.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifyPolicy {
    /// Minimum manifest version accepted.
    pub min_manifest_version: u32,
    /// Whether rollback to older release versions is permitted.
    pub allow_downgrade: bool,
}

impl Default for VerifyPolicy {
    fn default() -> Self {
        Self { min_manifest_version: UPDATE_MANIFEST_VERSION, allow_downgrade: false }
    }
}

/// Output of a successful signature verification pass.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedUpdate {
    /// Parsed and structurally valid manifest.
    pub manifest: UpdateManifest,
    /// Slot that will receive the payload after apply.
    pub target_slot: BootSlot,
}

/// Signature verification trait — host stub uses test vectors; firmware uses pinned keys.
pub trait VerifySignature {
    /// Verifies the manifest signature against trusted public key material.
    fn verify_manifest(
        &self,
        manifest: &UpdateManifest,
        policy: &VerifyPolicy,
    ) -> Result<VerifiedUpdate, UpdateError>;
}

/// Stub verifier that accepts only the all-zero test signature in development builds.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StubVerifier {
    /// When `true`, accepts manifests whose signature bytes are all zero (dev only).
    pub accept_test_signature: bool,
}

impl VerifySignature for StubVerifier {
    fn verify_manifest(
        &self,
        manifest: &UpdateManifest,
        policy: &VerifyPolicy,
    ) -> Result<VerifiedUpdate, UpdateError> {
        manifest.validate_structure()?;

        if manifest.version < policy.min_manifest_version {
            return Err(UpdateError::new(UpdateErrorCode::InvalidManifest));
        }

        let sig_len = manifest.algorithm.signature_len();
        let sig = &manifest.signature[..sig_len];

        if self.accept_test_signature && sig.iter().all(|&b| b == 0) {
            return Ok(VerifiedUpdate { manifest: *manifest, target_slot: manifest.target_slot });
        }

        // Real Ed25519 verification deferred to post-M12.
        Err(UpdateError::new(UpdateErrorCode::SignatureInvalid))
    }
}

/// Convenience function using the default stub verifier (test signature only).
///
/// Host tooling (`scripts/update-check.ps1`) and integration tests call this entry point.
#[allow(dead_code)] // public API; callers live outside this crate until init lands
pub fn verify_update_stub(manifest: &UpdateManifest) -> Result<VerifiedUpdate, UpdateError> {
    let verifier = StubVerifier { accept_test_signature: true };
    verifier.verify_manifest(manifest, &VerifyPolicy::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_manifest_validates_structure() {
        let manifest = UpdateManifest::empty();
        assert!(manifest.validate_structure().is_ok());
    }

    #[test]
    fn stub_verifier_accepts_zero_signature() {
        let manifest = UpdateManifest::empty();
        let result = verify_update_stub(&manifest);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().target_slot, BootSlot::B);
    }

    #[test]
    fn stub_verifier_rejects_nonzero_signature_without_crypto() {
        let mut manifest = UpdateManifest::empty();
        manifest.signature[0] = 0x01;
        let verifier = StubVerifier { accept_test_signature: false };
        let err = verifier.verify_manifest(&manifest, &VerifyPolicy::default()).unwrap_err();
        assert_eq!(err.code, UpdateErrorCode::SignatureInvalid);
    }
}
