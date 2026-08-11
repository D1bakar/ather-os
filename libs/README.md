# User-Space Libraries

> **Status:** Not started — planned alongside libc (M4–M6).

Shared user-space libraries (libc, runtime support) will live here or in dedicated
crates under `crates/` depending on whether they must be host-testable during development.

Kernel-shared types remain in `crates/aether-types` and `crates/aether-abi`.
