//! Atomic OS update architecture skeleton for Aether OS.
//!
//! This crate defines types and host-testable stubs for A/B partition updates,
//! signed manifest verification, and rollback. Runtime application to disk and
//! boot-chain integration are planned for post-M12 milestones.
//!
//! # Status
//!
//! **M12 skeleton only** — no I/O, no reboot, no boot loader hooks.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod partition;
mod rollback;
mod verify;

pub use error::{UpdateError, UpdateErrorCode};
pub use partition::{BootControlBlock, BootSlot, SlotState, SlotStatus, MAX_SLOT_LABEL_LEN};
pub use rollback::{RollbackManager, RollbackReason, RollbackRequest, RollbackResult};
pub use verify::{
    SignatureAlgorithm, UpdateManifest, UpdatePayloadKind, VerifiedUpdate, VerifyPolicy,
    VerifySignature,
};
