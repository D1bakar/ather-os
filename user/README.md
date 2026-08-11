# User Space

> **Status:** Not started — planned post-M4 (syscall dispatch) and M6 (init/shell).

This directory will contain user-space programs, test binaries, and eventually the
init process. User programs will link against a libc that uses the stable ABI
defined in `aether-abi`.

## Planned layout (M6+)

```
user/
├── init/       # First user process
├── shell/      # Interactive shell (optional milestone)
└── tests/      # User-mode integration tests
```

See [ARCHITECTURE.md](../ARCHITECTURE.md) for the process and capability models.
