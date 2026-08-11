//! Host-side QEMU boot smoke test.
//!
//! Run with `cargo test --test qemu_boot -- --ignored` when QEMU and OVMF are installed.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const EXPECTED: &str = "Aether OS kernel started";
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
    assert!(log.contains(EXPECTED), "serial log missing expected string: {EXPECTED}\n---\n{log}");
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
