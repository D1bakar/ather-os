//! User-space bootstrap: load embedded init ELF and spawn the first ring-3 task.

mod embedded;

use crate::elf::{load_elf_user, ElfError};
use crate::fs::mount_root;
use crate::process::{self, Process};
use crate::sched;
use crate::serial;

/// Mounts ramfs and spawns the embedded init process when present.
#[cfg(not(feature = "host-stub"))]
pub fn start_userland() {
    mount_root();
    serial::write_str("Aether OS M6: userland started\r\n");

    let image = embedded::init_elf();
    if image.is_empty() {
        serial::write_str("M6: no embedded init ELF (run scripts/build-user.ps1)\r\n");
        return;
    }

    match spawn_init(image) {
        Ok(pid) => {
            serial::write_str("M6: init process spawned pid=");
            write_decimal(pid.0);
            serial::write_str("\r\n");
        }
        Err(err) => {
            serial::write_str("M6: init spawn failed: ");
            serial::write_str(err.as_str());
            serial::write_str("\r\n");
        }
    }
}

/// Host-stub no-op.
#[cfg(feature = "host-stub")]
pub fn start_userland() {}

#[cfg(not(feature = "host-stub"))]
fn spawn_init(image: &[u8]) -> Result<crate::process::ProcessId, ElfError> {
    let page_table = crate::mm::user::create_user_address_space().ok_or(ElfError::MapFailed)?;
    let loaded = load_elf_user(page_table, image)?;

    let pid = sched::allocate_process_id();
    let task_id = sched::allocate_task_id();

    let mut proc = Process::new(pid, page_table, task_id);
    proc.grant_default_io_caps();
    process::register(proc);

    sched::spawn_init_user_task(task_id, pid, page_table.as_u64(), loaded.entry, loaded.stack_top);
    Ok(pid)
}

fn write_decimal(mut value: u32) {
    if value == 0 {
        serial::write_byte(b'0');
        return;
    }
    let mut buf = [0u8; 10];
    let mut index = buf.len();
    while value > 0 {
        index -= 1;
        buf[index] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    serial::write_str(core::str::from_utf8(&buf[index..]).unwrap_or("?"));
}

/// Formats ELF errors for host tests.
#[cfg(feature = "host-stub")]
pub fn describe_elf_error(err: ElfError) -> &'static str {
    err.as_str()
}
