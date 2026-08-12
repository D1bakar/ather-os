//! Root filesystem mount table (M6 ramfs at `/`).

use super::RamFs;

static mut ROOT_FS: Option<RamFs> = None;

/// Mounts an empty ramfs at `/`.
pub fn init() {
    // SAFETY: BSP-only init before other CPUs or user tasks exist.
    unsafe {
        core::ptr::write(core::ptr::addr_of_mut!(ROOT_FS), Some(RamFs::new()));
    }
}

/// Returns `true` when the root ramfs has been mounted.
#[must_use]
pub fn is_mounted() -> bool {
    // SAFETY: Read-only check after init.
    unsafe { (*core::ptr::addr_of!(ROOT_FS)).is_some() }
}

/// Runs `f` with mutable access to the root ramfs.
pub fn with_root<R>(f: impl FnOnce(&mut RamFs) -> R) -> R {
    // SAFETY: Single-threaded during early boot; later serialized by syscall path.
    unsafe { f((*core::ptr::addr_of_mut!(ROOT_FS)).as_mut().expect("root ramfs not mounted")) }
}

/// Re-initializes the root ramfs for host integration tests.
#[cfg(test)]
pub fn init_for_test() {
    init();
}
