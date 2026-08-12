#!/usr/bin/env bash
# Launch Aether OS in QEMU with OVMF and capture serial output.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TIMEOUT="${TIMEOUT:-35}"
NO_BUILD=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-build) NO_BUILD=1; shift ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

if [[ "$NO_BUILD" -eq 0 ]]; then
    bash "$ROOT/scripts/build-boot.sh"
fi

ESP_DIR="$ROOT/build/esp"
if [[ ! -f "$ESP_DIR/EFI/BOOT/BOOTX64.EFI" ]]; then
    echo "ESP not found. Run scripts/build-boot.sh first." >&2
    exit 1
fi

if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
    echo "qemu-system-x86_64 not found in PATH." >&2
    exit 1
fi

find_ovmf() {
    local name="$1"
    local candidates=(
        "$ROOT/ovmf/$name"
        "/usr/share/OVMF/$name"
        "/usr/share/ovmf/x64/$name"
        "/usr/share/edk2/ovmf/$name"
        "/usr/share/edk2-ovmf/OVMF/$name"
    )
    for path in "${candidates[@]}"; do
        if [[ -f "$path" ]]; then
            echo "$path"
            return 0
        fi
    done
    return 1
}

OVMF_CODE="$(find_ovmf OVMF_CODE.fd || find_ovmf OVMF_CODE.4MB.fd || true)"
OVMF_VARS="$(find_ovmf OVMF_VARS.fd || find_ovmf OVMF_VARS.4MB.fd || true)"

if [[ -z "$OVMF_CODE" || -z "$OVMF_VARS" ]]; then
    echo "OVMF firmware not found. Install ovmf/edk2-ovmf package or place files under ovmf/." >&2
    exit 1
fi

VARS_COPY="$ROOT/build/OVMF_VARS.runtime.fd"
cp "$OVMF_VARS" "$VARS_COPY"
LOG_FILE="$ROOT/build/qemu-serial.log"
rm -f "$LOG_FILE"

echo "==> Starting QEMU (timeout ${TIMEOUT}s)"
qemu-system-x86_64 \
    -machine q35 \
    -cpu max \
    -m 256M \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$VARS_COPY" \
    -drive format=raw,file=fat:rw:"$ESP_DIR" \
    -serial "file:$LOG_FILE" \
    -display none &
QEMU_PID=$!

deadline=$((SECONDS + TIMEOUT))
while (( SECONDS < deadline )); do
    if [[ -f "$LOG_FILE" ]] && grep -q "Aether init started" "$LOG_FILE"; then
        break
    fi
    if ! kill -0 "$QEMU_PID" 2>/dev/null; then
        break
    fi
    sleep 0.25
done

kill "$QEMU_PID" 2>/dev/null || true
wait "$QEMU_PID" 2>/dev/null || true

if [[ ! -f "$LOG_FILE" ]]; then
    echo "QEMU did not produce serial output." >&2
    exit 1
fi

echo "--- serial log ---"
cat "$LOG_FILE"
echo "------------------"

missing=()
for pattern in \
    "Aether OS kernel started" \
    "Aether OS M2: GDT/IDT/interrupts initialized" \
    "Aether OS M4: scheduler initialized" \
    "Aether OS M6: userland started" \
    "Aether init started"
do
    if ! grep -q "$pattern" "$LOG_FILE"; then
        missing+=("$pattern")
    fi
done

if ((${#missing[@]} == 0)); then
    echo "QEMU boot smoke test: PASS (ring-3 init verified)"
    exit 0
fi

echo "QEMU boot smoke test: FAIL — missing serial output:" >&2
printf '  - %s\n' "${missing[@]}" >&2
exit 1
