#!/usr/bin/env bash
# Run the full host test matrix and optional headless QEMU smoke (non-interactive).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RUN_QEMU=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --qemu) RUN_QEMU=1; shift ;;
        -h|--help)
            echo "Usage: $0 [--qemu]"
            exit 0
            ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

export CARGO_TERM_COLOR=always
export RUSTFLAGS="${RUSTFLAGS:--Dwarnings}"

echo "==> cargo test (workspace)"
cargo test --workspace

echo "==> property + fuzz + integration targets"
cargo test --manifest-path tests/Cargo.toml \
    --test property_abi --test property_gdt --test property_vfs \
    --test fuzz_syscall --test fuzz_ramfs --test integration_suite

if [[ "$RUN_QEMU" -eq 1 ]]; then
    echo "==> headless QEMU smoke"
    set +e
    bash "$ROOT/scripts/qemu-test.sh"
    QEMU_EXIT=$?
    set -e
    if [[ "$QEMU_EXIT" -eq 77 ]]; then
        echo "QEMU smoke skipped (tooling not installed)"
    elif [[ "$QEMU_EXIT" -ne 0 ]]; then
        exit "$QEMU_EXIT"
    fi
fi

echo ""
echo "run-all-tests: PASS"
