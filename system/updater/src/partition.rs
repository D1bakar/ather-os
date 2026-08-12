//! A/B partition slot model and boot control block.
//!
//! See [docs/updates/ab-partitions.md](../../../docs/updates/ab-partitions.md).

use crate::error::{UpdateError, UpdateErrorCode};

/// Maximum length of a human-readable slot label (e.g. `"slot_a"`).
pub const MAX_SLOT_LABEL_LEN: usize = 16;

/// Identifies an A/B update slot.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum BootSlot {
    /// Primary slot (slot A).
    A = 0,
    /// Secondary slot (slot B).
    B = 1,
}

impl BootSlot {
    /// Returns the inactive slot paired with this one.
    #[must_use]
    pub const fn inactive(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }

    /// Parses a slot from its numeric discriminator.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::A),
            1 => Some(Self::B),
            _ => None,
        }
    }
}

/// Lifecycle state of a slot's installed image.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum SlotState {
    /// Slot contains no bootable image.
    Empty = 0,
    /// Image written but not yet marked bootable (pending verification).
    Pending = 1,
    /// Image verified and eligible for next boot.
    Bootable = 2,
    /// Slot was booted successfully at least once (committed).
    Active = 3,
    /// Slot failed boot verification; do not select.
    Unbootable = 4,
}

impl SlotState {
    /// Parses a slot state from its numeric discriminator.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Empty),
            1 => Some(Self::Pending),
            2 => Some(Self::Bootable),
            3 => Some(Self::Active),
            4 => Some(Self::Unbootable),
            _ => None,
        }
    }

    /// Returns `true` if this slot may receive a staged update payload.
    #[must_use]
    pub const fn accepts_staging(self) -> bool {
        matches!(self, Self::Empty | Self::Pending | Self::Unbootable)
    }
}

/// Per-slot status stored in the boot control block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SlotStatus {
    /// Which slot this status describes.
    pub slot: BootSlot,
    /// Current lifecycle state.
    pub state: SlotState,
    /// Monotonic boot attempt counter (for rollback heuristics).
    pub boot_attempts: u32,
    /// Semantic version string length (bytes used in `version`).
    pub version_len: u8,
    /// NUL-padded version label (e.g. `"0.2.0"`).
    pub version: [u8; MAX_SLOT_LABEL_LEN],
}

impl SlotStatus {
    /// Creates an empty slot status.
    #[must_use]
    pub const fn empty(slot: BootSlot) -> Self {
        Self {
            slot,
            state: SlotState::Empty,
            boot_attempts: 0,
            version_len: 0,
            version: [0; MAX_SLOT_LABEL_LEN],
        }
    }

    /// Returns the version label as a byte slice.
    #[must_use]
    pub fn version_bytes(&self) -> &[u8] {
        let len = usize::from(self.version_len.min(MAX_SLOT_LABEL_LEN as u8));
        &self.version[..len]
    }
}

/// Fixed-layout boot control block persisted on the ESP or dedicated metadata partition.
///
/// Layout is versioned; consumers must check `magic` and `version` before use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootControlBlock {
    /// Magic bytes (`b"AETHBCB!"`).
    pub magic: [u8; 8],
    /// Structure version (currently `1`).
    pub version: u32,
    /// Slot selected for the next boot.
    pub active_slot: BootSlot,
    /// Number of consecutive failed boots on the active slot.
    pub failed_boots: u32,
    /// Maximum failed boots before automatic rollback.
    pub rollback_threshold: u32,
    /// Status for slot A.
    pub slot_a: SlotStatus,
    /// Status for slot B.
    pub slot_b: SlotStatus,
}

/// Magic value for [`BootControlBlock`].
pub const BOOT_CONTROL_MAGIC: [u8; 8] = *b"AETHBCB!";

/// Current boot control block layout version.
pub const BOOT_CONTROL_VERSION: u32 = 1;

impl BootControlBlock {
    /// Creates a default control block with slot A active and empty slot B.
    #[must_use]
    pub const fn default_initial() -> Self {
        Self {
            magic: BOOT_CONTROL_MAGIC,
            version: BOOT_CONTROL_VERSION,
            active_slot: BootSlot::A,
            failed_boots: 0,
            rollback_threshold: 3,
            slot_a: SlotStatus {
                slot: BootSlot::A,
                state: SlotState::Active,
                boot_attempts: 1,
                version_len: 0,
                version: [0; MAX_SLOT_LABEL_LEN],
            },
            slot_b: SlotStatus::empty(BootSlot::B),
        }
    }

    /// Validates magic and version fields.
    pub fn validate(&self) -> Result<(), UpdateError> {
        if self.magic != BOOT_CONTROL_MAGIC {
            return Err(UpdateError::new(UpdateErrorCode::InvalidManifest));
        }
        if self.version != BOOT_CONTROL_VERSION {
            return Err(UpdateError::new(UpdateErrorCode::InvalidManifest));
        }
        Ok(())
    }

    /// Returns status for the given slot.
    #[must_use]
    pub const fn status(&self, slot: BootSlot) -> &SlotStatus {
        match slot {
            BootSlot::A => &self.slot_a,
            BootSlot::B => &self.slot_b,
        }
    }

    /// Returns a mutable reference to status for the given slot.
    pub fn status_mut(&mut self, slot: BootSlot) -> &mut SlotStatus {
        match slot {
            BootSlot::A => &mut self.slot_a,
            BootSlot::B => &mut self.slot_b,
        }
    }

    /// Returns the inactive slot suitable for staging the next update.
    #[must_use]
    pub fn staging_slot(&self) -> BootSlot {
        self.active_slot.inactive()
    }

    /// Returns `true` if automatic rollback should be triggered.
    #[must_use]
    pub fn should_rollback(&self) -> bool {
        self.failed_boots >= self.rollback_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_control_block_validates() {
        let bcb = BootControlBlock::default_initial();
        assert!(bcb.validate().is_ok());
        assert_eq!(bcb.staging_slot(), BootSlot::B);
    }

    #[test]
    fn slot_inactive_pair() {
        assert_eq!(BootSlot::A.inactive(), BootSlot::B);
        assert_eq!(BootSlot::B.inactive(), BootSlot::A);
    }

    #[test]
    fn pending_slot_accepts_staging() {
        assert!(SlotState::Pending.accepts_staging());
        assert!(!SlotState::Active.accepts_staging());
    }
}
