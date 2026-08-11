# Reproducible build environment for Aether OS (canonical Linux release builds).
# Usage:
#   docker build -t aether-os-build .
#   docker run --rm -v "$PWD":/workspace -w /workspace aether-os-build scripts/ci-check.sh
#   docker run --rm -v "$PWD":/workspace -w /workspace aether-os-build scripts/build-release.sh

FROM rust:1.85.0-bookworm

RUN rustup component add rustfmt clippy rust-src llvm-tools-preview \
    && rustup target add x86_64-unknown-none x86_64-unknown-uefi

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        qemu-system-x86 \
        ovmf \
        zip \
        git \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTC_BOOTSTRAP=1 \
    RUSTFLAGS=-Dwarnings \
    CARGO_TERM_COLOR=always

WORKDIR /workspace

# Default: run the full CI gate (override CMD for release builds).
CMD ["bash", "scripts/ci-check.sh"]
