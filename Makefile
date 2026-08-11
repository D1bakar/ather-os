# Aether OS — Build Orchestration

CARGO := cargo
TARGET_DIR := target

.PHONY: all build run test clean fmt clippy check help

all: build

help:
	@echo "Aether OS Build Targets"
	@echo ""
	@echo "  make build   — Build all workspace crates"
	@echo "  make test    — Run fmt, clippy, and tests"
	@echo "  make run     — Boot in QEMU (requires M1)"
	@echo "  make clean   — Remove build artifacts"
	@echo "  make fmt     — Format all Rust code"
	@echo "  make clippy  — Run Clippy with warnings denied"
	@echo "  make check   — Alias for test"

build:
	$(CARGO) build --workspace

test: fmt clippy
	$(CARGO) test --workspace

run:
	@echo "============================================================"
	@echo "  Aether OS boot target is not yet available."
	@echo ""
	@echo "  Boot loader and kernel implementation begins in M1."
	@echo "  Once M1 lands, this target will:"
	@echo "    1. Build boot loader (BOOTX64.EFI) and kernel (kernel.elf)"
	@echo "    2. Create a FAT32 disk image with ESP layout"
	@echo "    3. Launch QEMU with OVMF UEFI firmware"
	@echo ""
	@echo "  See docs/architecture/001-initial-decisions.md for details."
	@echo "============================================================"
	@exit 1

clean:
	$(CARGO) clean
	@if exist "$(TARGET_DIR)" rmdir /s /q "$(TARGET_DIR)" 2>nul
	@if exist "build" rmdir /s /q "build" 2>nul

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

check: test
