# Aether OS — Build Orchestration (Windows + Unix)

CARGO := cargo
TARGET_DIR := target
TOOLS := aether-serial aether-img-builder

ifeq ($(OS),Windows_NT)
    DETECTED_OS := windows
    RUN_SCRIPT := powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-qemu.ps1
    BOOT_SCRIPT := powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-boot.ps1
    SETUP_SCRIPT := powershell -NoProfile -ExecutionPolicy Bypass -File scripts/setup-dev.ps1
    INSTALL_SCRIPT := powershell -NoProfile -ExecutionPolicy Bypass -File install.ps1
    RM_BUILD := if exist "build" rmdir /s /q "build" 2>nul
    SERIAL_BIN := $(TARGET_DIR)/release/aether-serial.exe
    IMG_BIN := $(TARGET_DIR)/release/aether-img-builder.exe
else
    DETECTED_OS := unix
    RUN_SCRIPT := bash scripts/run-qemu.sh
    BOOT_SCRIPT := bash scripts/build-boot.sh
    SETUP_SCRIPT := bash scripts/setup-dev.sh
    INSTALL_SCRIPT := $(SETUP_SCRIPT) && $(CARGO) build --workspace && $(CARGO) build -p aether-serial -p aether-img-builder --release
    RM_BUILD := rm -rf build/
    SERIAL_BIN := $(TARGET_DIR)/release/aether-serial
    IMG_BIN := $(TARGET_DIR)/release/aether-img-builder
endif

.PHONY: all help setup install build boot run test test-all qemu-test fmt clippy check m2-check clean
.PHONY: tools tools-build serial monitor image image-verify image-info

all: build

help:
	@echo "Aether OS Build Targets ($(DETECTED_OS))"
	@echo ""
	@echo "  make install   — One-command developer setup (install.ps1 on Windows)"
	@echo "  make setup     — Install Rust targets and show QEMU/OVMF hints"
	@echo "  make build     — Build host workspace crates"
	@echo "  make tools     — Build developer tools (aether-serial, aether-img-builder)"
	@echo "  make boot      — Build UEFI loader + bare-metal kernel (ESP)"
	@echo "  make image     — Build raw FAT32 disk image (requires boot)"
	@echo "  make image-verify — Verify ESP layout"
	@echo "  make run       — Boot in QEMU (builds boot artifacts first)"
	@echo "  make serial    — Follow build/qemu-serial.log (aether-serial)"
	@echo "  make test      — Run fmt, clippy, and workspace tests"
	@echo "  make test-all  — Run property/fuzz/integration tests (optional --qemu via script)"
	@echo "  make qemu-test — Headless QEMU boot smoke (non-interactive)"
	@echo "  make fmt       — Format all Rust code"
	@echo "  make clippy    — Run Clippy with warnings denied"
	@echo "  make check     — Alias for test"
	@echo "  make m2-check  — Pre-M2 gate: quality checks + boot build"
	@echo "  make clean     — Remove build artifacts"

setup:
	$(SETUP_SCRIPT)

install:
	$(INSTALL_SCRIPT)

build:
	$(CARGO) build --workspace

tools: tools-build

tools-build:
	$(CARGO) build -p aether-serial -p aether-img-builder --release

boot:
	$(BOOT_SCRIPT)

image: boot tools-build
	$(IMG_BIN) build build/esp build/aether.img

image-verify: tools-build
	$(IMG_BIN) verify build/esp

image-info: tools-build
	$(IMG_BIN) info build/esp

run:
	$(RUN_SCRIPT)

serial: tools-build
	$(SERIAL_BIN) follow build/qemu-serial.log

monitor: serial

test: fmt clippy
	$(CARGO) test --workspace

test-all:
ifeq ($(OS),Windows_NT)
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-all-tests.ps1
else
	bash scripts/run-all-tests.sh
endif

qemu-test:
ifeq ($(OS),Windows_NT)
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts/qemu-test.ps1
else
	bash scripts/qemu-test.sh
endif

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy --workspace --exclude aether-boot --all-targets -- -D warnings
	$(CARGO) clippy -p aether-boot --target x86_64-unknown-uefi -- -D warnings

check: test

m2-check:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --workspace --exclude aether-boot --all-targets -- -D warnings
	$(CARGO) clippy -p aether-boot --target x86_64-unknown-uefi -- -D warnings
	$(CARGO) test --workspace
	$(BOOT_SCRIPT)

clean:
	$(CARGO) clean
	@$(RM_BUILD)
