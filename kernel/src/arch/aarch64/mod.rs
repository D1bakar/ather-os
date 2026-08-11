//! AArch64 CPU bring-up scaffold (M13 — not bootable).
//!
//! Module layout mirrors [`super::x86_64`] for future GIC, MMU, and timer work.
//! Nothing in this tree is linked into a shipped boot path or verified in CI yet.

pub mod boot;
pub mod serial;
