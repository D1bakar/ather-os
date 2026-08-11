#!/usr/bin/env bash
# Headless QEMU boot test — no user input required.
# Builds boot artifacts, runs QEMU with -display none, validates serial output, exits 0/1.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TIMEOUT="${TIMEOUT:-35}"
NO_BUILD=0
SKIP_QEMU=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-build) NO_BUILD=1; shift ;;
        --skip-qemu) SKIP_QEMU=1; shift ;;
        --timeout) TIMEOUT="${2:?}"; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [--no-build] [--skip-qemu] [--timeout SECS]"
            exit 0
            ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

if [[ "$SKIP_QEMU" -eq 1 ]]; then
    echo "qemu-test: skipped (--skip-qemu)"
    exit 0
fi

if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
    echo "qemu-test: SKIP — qemu-system-x86_64 not in PATH" >&2
    exit 77
fi

if [[ "$NO_BUILD" -eq 0 ]]; then
    "$ROOT/scripts/build-boot.sh"
fi

exec "$ROOT/scripts/run-qemu.sh" --no-build
