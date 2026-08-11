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
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-qemu.ps1

clean:
	$(CARGO) clean
	@if exist "$(TARGET_DIR)" rmdir /s /q "$(TARGET_DIR)" 2>nul
	@if exist "build" rmdir /s /q "build" 2>nul

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy --workspace --exclude aether-boot --all-targets -- -D warnings
	$(CARGO) clippy -p aether-boot --target x86_64-unknown-uefi -- -D warnings

check: test
