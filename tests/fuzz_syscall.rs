//! Syscall dispatch fuzz stub — hammers random numbers and args without panicking.

#[path = "support/rng.rs"]
mod rng;

use aether_abi::{syscall_count, SyscallArgs, SyscallNumber};
use aether_kernel::syscall::dispatch;
use aether_types::ErrorCode;
use rng::for_each_case;

fn assert_dispatch_result_is_stable(result: i64) {
    if result < 0 {
        let code = ErrorCode::from_i32(result as i32);
        assert!(
            matches!(
                code,
                ErrorCode::NotSupported
                    | ErrorCode::BadAddress
                    | ErrorCode::PermissionDenied
                    | ErrorCode::InvalidArgument
                    | ErrorCode::Internal
            ),
            "unexpected negative syscall result: {result} ({code:?})"
        );
    }
}

#[test]
fn random_syscall_numbers_do_not_panic() {
    for_each_case(512, |rng, _| {
        let number = rng.next_u64();
        let args = SyscallArgs::new(
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
        );
        let result = dispatch(number, args);
        assert_dispatch_result_is_stable(result);
        if number >= syscall_count() {
            assert_eq!(result, ErrorCode::NotSupported.as_i32() as i64);
        }
    });
}

#[test]
fn kernel_space_pointers_on_write_fail() {
    for len in 1u64..=4096u64 {
        for &number in &[SyscallNumber::Write.as_u64(), SyscallNumber::Read.as_u64()] {
            let args = SyscallArgs::new(0, 0xFFFF_8000_0000_0000, len, 0, 0, 0);
            let result = dispatch(number, args);
            assert_eq!(result, ErrorCode::BadAddress.as_i32() as i64);
        }
    }
}

#[test]
fn yield_and_getpid_return_non_error() {
    for &number in &[SyscallNumber::Yield.as_u64(), SyscallNumber::GetPid.as_u64()] {
        let result = dispatch(number, SyscallArgs::default());
        assert!(result >= 0, "expected success, got {result}");
    }
}
