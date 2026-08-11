//! Security subsystem — audit logging and policy re-exports (M5).

pub mod audit;

pub use aether_types::SecurityDefaults;
pub use audit::{clear as clear_audit_log, latest_record, record_count, record_event, AuditLog};
