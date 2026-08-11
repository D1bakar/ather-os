//! Register layout for syscall arguments on x86_64.

/// Syscall arguments as they appear in CPU registers at the syscall entry point.
///
/// # Register Mapping
///
/// | Register | Field   |
/// |----------|---------|
/// | RDI      | arg0    |
/// | RSI      | arg1    |
/// | RDX      | arg2    |
/// | R10      | arg3    |
/// | R8       | arg4    |
/// | R9       | arg5    |
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SyscallArgs {
    /// First argument (RDI).
    pub arg0: u64,
    /// Second argument (RSI).
    pub arg1: u64,
    /// Third argument (RDX).
    pub arg2: u64,
    /// Fourth argument (R10).
    pub arg3: u64,
    /// Fifth argument (R8).
    pub arg4: u64,
    /// Sixth argument (R9).
    pub arg5: u64,
}

impl SyscallArgs {
    /// Creates a new argument set with the given values.
    #[must_use]
    pub const fn new(arg0: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> Self {
        Self { arg0, arg1, arg2, arg3, arg4, arg5 }
    }

    /// Returns argument at the given index (0–5).
    #[must_use]
    pub const fn get(self, index: usize) -> Option<u64> {
        match index {
            0 => Some(self.arg0),
            1 => Some(self.arg1),
            2 => Some(self.arg2),
            3 => Some(self.arg3),
            4 => Some(self.arg4),
            5 => Some(self.arg5),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_get_by_index() {
        let args = SyscallArgs::new(1, 2, 3, 4, 5, 6);
        assert_eq!(args.get(0), Some(1));
        assert_eq!(args.get(5), Some(6));
        assert_eq!(args.get(6), None);
    }
}
