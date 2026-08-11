//! Rollback API skeleton for A/B slot recovery.
//!
//! See [docs/updates/rollback-api.md](../../../docs/updates/rollback-api.md).

use crate::error::{UpdateError, UpdateErrorCode};
use crate::partition::{BootControlBlock, BootSlot, SlotState};

/// Reason recorded when initiating a rollback.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum RollbackReason {
    /// User or admin explicitly requested rollback.
    UserRequested = 1,
    /// Boot attempt counter exceeded threshold on active slot.
    BootFailureThreshold = 2,
    /// Post-boot health check failed (future init watchdog).
    HealthCheckFailed = 3,
    /// Update verification failed before commit.
    UpdateVerificationFailed = 4,
}

impl RollbackReason {
    /// Parses a reason from its numeric discriminator.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::UserRequested),
            2 => Some(Self::BootFailureThreshold),
            3 => Some(Self::HealthCheckFailed),
            4 => Some(Self::UpdateVerificationFailed),
            _ => None,
        }
    }
}

/// Request to roll back to the previously active slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RollbackRequest {
    /// Why rollback was initiated.
    pub reason: RollbackReason,
    /// If `true`, mark the failed slot as unbootable after rollback.
    pub quarantine_failed_slot: bool,
}

impl RollbackRequest {
    /// Creates a user-initiated rollback request.
    #[must_use]
    pub const fn user_requested() -> Self {
        Self { reason: RollbackReason::UserRequested, quarantine_failed_slot: true }
    }

    /// Creates an automatic rollback triggered by boot failures.
    #[must_use]
    pub const fn boot_failure() -> Self {
        Self { reason: RollbackReason::BootFailureThreshold, quarantine_failed_slot: true }
    }
}

/// Result of a rollback planning operation (no reboot performed in M12).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RollbackResult {
    /// Slot that will become active on next boot.
    pub new_active_slot: BootSlot,
    /// Slot that was active before rollback.
    pub previous_active_slot: BootSlot,
    /// Reason recorded in the control block audit trail (future).
    pub reason: RollbackReason,
}

/// Manages rollback decisions against an in-memory boot control block.
///
/// M12 skeleton: mutates the control block only; persistence and reboot are out of scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackManager {
    /// Current boot control block state.
    pub control: BootControlBlock,
}

impl RollbackManager {
    /// Creates a manager wrapping the given control block.
    #[must_use]
    pub fn new(control: BootControlBlock) -> Self {
        Self { control }
    }

    /// Returns `true` if rollback is possible (inactive slot is bootable or active).
    #[must_use]
    pub fn can_rollback(&self) -> bool {
        let fallback = self.control.active_slot.inactive();
        matches!(self.control.status(fallback).state, SlotState::Bootable | SlotState::Active)
    }

    /// Plans and applies a rollback to the inactive slot.
    ///
    /// Returns [`UpdateErrorCode::RollbackUnavailable`] if the fallback slot cannot boot.
    pub fn rollback(&mut self, request: RollbackRequest) -> Result<RollbackResult, UpdateError> {
        self.control.validate()?;

        let previous = self.control.active_slot;
        let fallback = previous.inactive();
        let fallback_state = self.control.status(fallback).state;

        if !matches!(fallback_state, SlotState::Bootable | SlotState::Active) {
            return Err(UpdateError::new(UpdateErrorCode::RollbackUnavailable));
        }

        if request.quarantine_failed_slot {
            self.control.status_mut(previous).state = SlotState::Unbootable;
        }

        self.control.active_slot = fallback;
        self.control.failed_boots = 0;
        self.control.status_mut(fallback).state = SlotState::Active;

        Ok(RollbackResult {
            new_active_slot: fallback,
            previous_active_slot: previous,
            reason: request.reason,
        })
    }

    /// Evaluates automatic rollback when boot failure threshold is reached.
    pub fn maybe_auto_rollback(&mut self) -> Result<Option<RollbackResult>, UpdateError> {
        if !self.control.should_rollback() {
            return Ok(None);
        }
        if !self.can_rollback() {
            return Err(UpdateError::new(UpdateErrorCode::RollbackUnavailable));
        }
        let result = self.rollback(RollbackRequest::boot_failure())?;
        Ok(Some(result))
    }

    /// Commits a successful boot of the active slot (resets failure counter).
    pub fn commit_boot_success(&mut self) -> Result<(), UpdateError> {
        self.control.validate()?;
        self.control.failed_boots = 0;
        let slot = self.control.active_slot;
        self.control.status_mut(slot).state = SlotState::Active;
        self.control.status_mut(slot).boot_attempts += 1;
        Ok(())
    }

    /// Records a failed boot attempt on the active slot.
    pub fn record_boot_failure(&mut self) -> Result<(), UpdateError> {
        self.control.validate()?;
        self.control.failed_boots += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::partition::{SlotStatus, BOOT_CONTROL_MAGIC, BOOT_CONTROL_VERSION};

    fn control_with_bootable_b() -> BootControlBlock {
        BootControlBlock {
            magic: BOOT_CONTROL_MAGIC,
            version: BOOT_CONTROL_VERSION,
            active_slot: BootSlot::A,
            failed_boots: 3,
            rollback_threshold: 3,
            slot_a: SlotStatus {
                slot: BootSlot::A,
                state: SlotState::Active,
                boot_attempts: 2,
                version_len: 0,
                version: [0; 16],
            },
            slot_b: SlotStatus {
                slot: BootSlot::B,
                state: SlotState::Bootable,
                boot_attempts: 0,
                version_len: 0,
                version: [0; 16],
            },
        }
    }

    #[test]
    fn rollback_switches_active_slot() {
        let mut mgr = RollbackManager::new(control_with_bootable_b());
        let result = mgr.rollback(RollbackRequest::user_requested()).expect("rollback");
        assert_eq!(result.new_active_slot, BootSlot::B);
        assert_eq!(result.previous_active_slot, BootSlot::A);
        assert_eq!(mgr.control.active_slot, BootSlot::B);
        assert_eq!(mgr.control.status(BootSlot::A).state, SlotState::Unbootable);
    }

    #[test]
    fn auto_rollback_on_threshold() {
        let mut mgr = RollbackManager::new(control_with_bootable_b());
        let result = mgr.maybe_auto_rollback().expect("auto rollback").expect("some rollback");
        assert_eq!(result.reason, RollbackReason::BootFailureThreshold);
    }

    #[test]
    fn rollback_unavailable_when_fallback_empty() {
        let bcb = BootControlBlock::default_initial();
        let mut mgr = RollbackManager::new(bcb);
        let err = mgr.rollback(RollbackRequest::user_requested()).unwrap_err();
        assert_eq!(err.code, UpdateErrorCode::RollbackUnavailable);
    }
}
