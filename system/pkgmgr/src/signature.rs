//! ed25519 signature format and verification stub.
//!
//! Signed packages ship a sidecar `aether.sig` TOML file alongside `aether.toml`.
//! The signed payload is the SHA-256 digest of the concatenation:
//! `manifest_bytes || 0x00 || payload_bytes`.
//!
//! With the `verify` feature, host tooling performs real ed25519 checks. Without it,
//! structural validation runs but cryptographic verification returns
//! [`PkgError::VerificationNotEnabled`].

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::{PkgError, PkgResult};

/// ed25519 public key length in bytes.
pub const ED25519_PUBLIC_KEY_LEN: usize = 32;

/// ed25519 signature length in bytes.
pub const ED25519_SIGNATURE_LEN: usize = 64;

/// Signature sidecar schema version.
pub const SIGNATURE_VERSION: u32 = 1;

/// Parsed `aether.sig` sidecar.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    /// Schema version; must equal [`SIGNATURE_VERSION`].
    #[serde(rename = "signature-version")]
    pub signature_version: u32,
    /// Signing algorithm identifier (`ed25519` only in v1).
    pub algorithm: String,
    /// Hex-encoded ed25519 public key (64 hex chars).
    #[serde(rename = "public-key")]
    pub public_key: String,
    /// Hex-encoded ed25519 signature (128 hex chars).
    pub signature: String,
    /// Hex-encoded SHA-256 of the signed payload (64 hex chars).
    #[serde(rename = "payload-digest")]
    pub payload_digest: String,
}

/// Signature verification failures with stable messages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignatureVerifyError {
    /// Sidecar failed structural validation.
    Invalid(&'static str),
    /// Digest mismatch (manifest/payload do not match sidecar).
    DigestMismatch,
    /// Cryptographic verification failed.
    BadSignature,
}

impl Signature {
    /// Parses a signature sidecar from TOML.
    pub fn parse(toml_text: &str) -> PkgResult<Self> {
        let sig: Self =
            toml::from_str(toml_text).map_err(|err| PkgError::Parse(alloc::format!("{err}")))?;
        sig.validate_structure()?;
        Ok(sig)
    }

    /// Serializes the signature sidecar to TOML.
    pub fn to_toml(&self) -> PkgResult<String> {
        toml::to_string_pretty(self).map_err(|err| PkgError::Parse(alloc::format!("{err}")))
    }

    /// Validates field shapes without verifying cryptography.
    pub fn validate_structure(&self) -> PkgResult<()> {
        if self.signature_version != SIGNATURE_VERSION {
            return Err(PkgError::InvalidSignature("unsupported signature-version"));
        }
        if self.algorithm != "ed25519" {
            return Err(PkgError::InvalidSignature("unsupported algorithm"));
        }

        decode_hex_fixed(&self.public_key, ED25519_PUBLIC_KEY_LEN, "public-key")?;
        decode_hex_fixed(&self.signature, ED25519_SIGNATURE_LEN, "signature")?;
        decode_hex_fixed(&self.payload_digest, 32, "payload-digest")?;

        Ok(())
    }
}

/// Verifies package signatures.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SignatureVerifier;

impl SignatureVerifier {
    /// Creates a new verifier instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validates the sidecar, optionally checks ed25519 when the `verify` feature is enabled.
    pub fn verify(
        &self,
        manifest_bytes: &[u8],
        payload_bytes: &[u8],
        signature: &Signature,
    ) -> PkgResult<()> {
        signature.validate_structure()?;

        #[cfg(not(feature = "verify"))]
        {
            let _ = (manifest_bytes, payload_bytes, signature);
            Err(PkgError::VerificationNotEnabled)
        }

        #[cfg(feature = "verify")]
        {
            let digest = compute_payload_digest(manifest_bytes, payload_bytes);
            let expected = decode_hex_fixed(&signature.payload_digest, 32, "payload-digest")?;
            if digest != expected {
                return Err(PkgError::InvalidSignature("payload-digest mismatch"));
            }

            verify_ed25519(manifest_bytes, payload_bytes, signature)
        }
    }
}

#[cfg(feature = "verify")]
fn compute_payload_digest(manifest_bytes: &[u8], payload_bytes: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(manifest_bytes);
    hasher.update([0u8]);
    hasher.update(payload_bytes);
    hasher.finalize().to_vec()
}

#[cfg(feature = "verify")]
fn verify_ed25519(
    manifest_bytes: &[u8],
    payload_bytes: &[u8],
    signature: &Signature,
) -> PkgResult<()> {
    #[cfg(feature = "verify")]
    {
        use ed25519_dalek::{Signature as DalekSignature, Verifier, VerifyingKey};

        let public_key_bytes =
            decode_hex_fixed(&signature.public_key, ED25519_PUBLIC_KEY_LEN, "public-key")?;
        let signature_bytes =
            decode_hex_fixed(&signature.signature, ED25519_SIGNATURE_LEN, "signature")?;

        let verifying_key = VerifyingKey::from_bytes(
            public_key_bytes
                .as_slice()
                .try_into()
                .map_err(|_| PkgError::InvalidSignature("public-key length"))?,
        )
        .map_err(|_| PkgError::InvalidSignature("invalid public-key"))?;

        let sig = DalekSignature::from_bytes(
            signature_bytes
                .as_slice()
                .try_into()
                .map_err(|_| PkgError::InvalidSignature("signature length"))?,
        );

        let mut message = Vec::with_capacity(manifest_bytes.len() + 1 + payload_bytes.len());
        message.extend_from_slice(manifest_bytes);
        message.push(0);
        message.extend_from_slice(payload_bytes);

        verifying_key
            .verify(&message, &sig)
            .map_err(|_| PkgError::InvalidSignature("ed25519 verification failed"))?;

        Ok(())
    }
}

fn decode_hex_fixed(field: &str, expected_len: usize, name: &str) -> PkgResult<Vec<u8>> {
    if field.len() != expected_len * 2 {
        return Err(PkgError::InvalidSignature("invalid hex field length"));
    }

    if !field.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(PkgError::InvalidSignature("hex field must be lowercase hex"));
    }

    let mut out = Vec::with_capacity(expected_len);
    let bytes = field.as_bytes();
    let mut idx = 0;
    while idx < bytes.len() {
        let hi = from_hex_nibble(bytes[idx])?;
        let lo = from_hex_nibble(bytes[idx + 1])?;
        out.push((hi << 4) | lo);
        idx += 2;
    }

    if out.len() != expected_len {
        return Err(PkgError::InvalidSignature("invalid hex field length"));
    }

    let _ = name;
    Ok(out)
}

fn from_hex_nibble(byte: u8) -> PkgResult<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(PkgError::InvalidSignature("invalid hex digit")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIG_TOML: &str = r#"
signature-version = 1
algorithm = "ed25519"
public-key = "0000000000000000000000000000000000000000000000000000000000000001"
signature = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
payload-digest = "0000000000000000000000000000000000000000000000000000000000000000"
"#;

    #[test]
    fn parse_signature_sidecar() {
        let sig = Signature::parse(SIG_TOML).expect("valid sidecar");
        assert_eq!(sig.algorithm, "ed25519");
    }

    #[test]
    fn stub_verify_requires_feature() {
        let sig = Signature::parse(SIG_TOML).expect("valid sidecar");
        let verifier = SignatureVerifier::new();
        let err = verifier.verify(b"manifest", b"payload", &sig).unwrap_err();
        assert!(matches!(err, PkgError::VerificationNotEnabled));
    }
}
