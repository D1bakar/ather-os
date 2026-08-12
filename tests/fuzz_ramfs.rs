//! RamFS fuzz stub — random valid-path operation sequences must not panic.

#[path = "support/rng.rs"]
mod rng;

use aether_kernel::fs::RamFs;
use aether_kernel::vfs::{OpenFlags, Vfs};
use aether_types::ErrorCode;
use rng::for_each_case;

const PATHS: &[&str] = &["/fuzz", "/a", "/etc/cfg", "/dev/null", "/tmp/x"];

#[test]
fn random_write_read_roundtrip() {
    for_each_case(256, |rng, case| {
        let path = PATHS[rng.next_bounded(PATHS.len() as u64) as usize];
        let payload_len = (rng.next_bounded(64) + 1) as usize;
        let mut payload = vec![0u8; payload_len];
        rng.fill_bytes(&mut payload);
        let offset = rng.next_bounded(128);

        let mut fs = RamFs::new();
        let flags = OpenFlags::READ | OpenFlags::WRITE | OpenFlags::CREATE;
        let fd = fs.open(path, flags).expect("open created file");

        if fs.write(fd, &payload, offset).is_ok() {
            let mut buf = vec![0u8; payload.len()];
            let read = fs.read(fd, &mut buf, offset).unwrap_or(0);
            if read == payload.len() {
                assert_eq!(buf, payload, "case {case}");
            }
        }

        let _ = fs.close(fd);
    });
}

#[test]
fn double_close_is_error() {
    for path in PATHS {
        let mut fs = RamFs::new();
        let fd =
            fs.open(path, OpenFlags::READ | OpenFlags::WRITE | OpenFlags::CREATE).expect("open");
        fs.close(fd).expect("first close");
        let err = fs.close(fd).expect_err("second close");
        assert_eq!(err.code, ErrorCode::InvalidArgument);
    }
}

#[test]
fn stat_root_always_succeeds() {
    let fs = RamFs::new();
    let stat = fs.stat("/").expect("root stat");
    assert!(stat.is_dir);
}

#[test]
fn open_missing_without_create_returns_not_found() {
    for path in PATHS {
        if *path == "/" {
            continue;
        }
        let mut fs = RamFs::new();
        let err = fs.open(path, OpenFlags::READ).expect_err("missing");
        assert_eq!(err.code, ErrorCode::NotFound);
    }
}
