//! Embedded user binaries for the M6 kernel demo.
//!
//! `build.rs` copies `build/user/init.elf` into `OUT_DIR/init.elf` when present.

include!(concat!(env!("OUT_DIR"), "/embedded_init.rs"));
