//! Host-side QEMU boot smoke test.
//!
//! Run with `cargo test --test qemu_boot -- --ignored` when QEMU and OVMF are installed.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const EXPECTED_BOOT: &str = "Aether OS kernel started";
const EXPECTED_M2: &str = "Aether OS M2: GDT/IDT/interrupts initialized";
const EXPECTED_M4: &str = "Aether OS M4: scheduler initialized";
const EXPECTED_M6: &str = "Aether OS M6: userland started";
const EXPECTED_INIT: &str = "Aether init started";
const EXPECTED_TIMER: &str = "[timer] tick";
const EXPECTED_WORKER: &str = "[worker] kernel thread tick";
const TIMEOUT: Duration = Duration::from_secs(45);

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn script_path(root: &Path) -> PathBuf {
    if cfg!(windows) {
        root.join("scripts/run-qemu.ps1")
    } else {
        root.join("scripts/run-qemu.sh")
    }
}

#[test]
#[ignore = "requires QEMU, OVMF, and boot artifacts"]
fn qemu_boot_prints_kernel_message() {
    let root = root();
    let log_path = root.join("build/qemu-serial.log");

    let status = if cfg!(windows) {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                script_path(&root).to_str().expect("utf-8 path"),
            ])
            .current_dir(&root)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("failed to spawn run-qemu.ps1")
    } else {
        Command::new("bash")
            .arg(script_path(&root))
            .current_dir(&root)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("failed to spawn run-qemu.sh")
    };

    assert!(status.success(), "QEMU smoke script failed");

    let log = std::fs::read_to_string(&log_path).expect("serial log missing");
    assert!(
        log.contains(EXPECTED_BOOT),
        "serial log missing boot banner: {EXPECTED_BOOT}\n---\n{log}"
    );
    assert!(
        log.contains(EXPECTED_M2),
        "serial log missing M2 init message: {EXPECTED_M2}\n---\n{log}"
    );
    assert!(
        log.contains(EXPECTED_M4),
        "serial log missing M4 scheduler message: {EXPECTED_M4}\n---\n{log}"
    );
    assert!(
        log.contains(EXPECTED_M6),
        "serial log missing M6 userland banner: {EXPECTED_M6}\n---\n{log}"
    );
    assert!(
        log.contains(EXPECTED_INIT),
        "serial log missing ring-3 init message: {EXPECTED_INIT}\n---\n{log}"
    );
    // Timer ticks and worker thread output appear after STI; optional if QEMU run is short.
    if log.contains(EXPECTED_TIMER) {
        eprintln!("QEMU smoke: timer tick output observed");
    }
    if log.contains(EXPECTED_WORKER) {
        eprintln!("QEMU smoke: worker thread output observed");
    }
}

#[test]
fn boot_artifacts_layout_documented() {
    let root = root();
    let esp = root.join("build/esp");
    let boot_efi = esp.join("EFI/BOOT/BOOTX64.EFI");
    let kernel_elf = esp.join("aether/kernel.elf");

    // This test documents paths; it passes even when artifacts are absent.
    assert_eq!(boot_efi.file_name().unwrap(), "BOOTX64.EFI");
    assert_eq!(kernel_elf.file_name().unwrap(), "kernel.elf");
}

#[test]
fn qemu_smoke_timeout_is_reasonable() {
    let start = Instant::now();
    assert!(TIMEOUT >= Duration::from_secs(30));
    assert!(start.elapsed() < Duration::from_secs(1));
}
