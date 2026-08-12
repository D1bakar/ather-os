//! Property tests for the stable syscall ABI (`aether-abi`).

#[path = "support/rng.rs"]
mod rng;

use aether_abi::{descriptor_for, lookup_syscall, syscall_count, SyscallArgs, SyscallNumber};
use rng::for_each_case;

#[test]
fn syscall_roundtrip_for_valid_range() {
    for index in 0..syscall_count() {
        let num = SyscallNumber::from_u64(index).expect("valid syscall");
        assert_eq!(num.as_u64(), index);
        assert!(lookup_syscall(index).is_some());
        assert!(descriptor_for(num).is_some());
    }
}

#[test]
fn unknown_syscall_numbers_are_rejected() {
    for_each_case(256, |rng, _| {
        let raw = syscall_count() + rng.next_bounded(u64::MAX - syscall_count());
        assert!(SyscallNumber::from_u64(raw).is_none());
        assert!(lookup_syscall(raw).is_none());
    });
}

#[test]
fn syscall_args_get_index_consistency() {
    for_each_case(512, |rng, _| {
        let args = SyscallArgs::new(
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
        );
        for index in 0..6usize {
            assert_eq!(
                args.get(index),
                Some([args.arg0, args.arg1, args.arg2, args.arg3, args.arg4, args.arg5][index])
            );
        }
        for index in 6..12usize {
            assert_eq!(args.get(index), None);
        }
    });
}

#[test]
fn descriptor_names_are_non_empty() {
    for index in 0..syscall_count() {
        let desc = lookup_syscall(index).expect("descriptor");
        assert!(!desc.name.is_empty());
        assert!(desc.arg_count <= 6);
    }
}
