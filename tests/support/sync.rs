//! Serializes tests that mutate kernel-global bring-up state.

use std::sync::Mutex;

static GLOBAL_CAP_LOCK: Mutex<()> = Mutex::new(());

/// Runs `f` while holding an exclusive lock on global capability-table tests.
pub fn with_global_cap_lock<R>(f: impl FnOnce() -> R) -> R {
    let _guard = GLOBAL_CAP_LOCK.lock().expect("cap test lock");
    f()
}
