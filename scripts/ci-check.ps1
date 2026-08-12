# Full local CI gate — mirrors .github/workflows/ci.yml quality job.

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$env:CARGO_TERM_COLOR = "always"
$env:RUSTFLAGS = "-Dwarnings"
$env:RUSTC_BOOTSTRAP = "1"

Write-Host "==> rustup targets"
rustup target add x86_64-unknown-none x86_64-unknown-uefi
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> cargo fmt --check"
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> cargo clippy (host workspace)"
cargo clippy --workspace --exclude aether-boot --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> cargo clippy (UEFI boot loader)"
cargo clippy -p aether-boot --target x86_64-unknown-uefi -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> cargo test --workspace"
cargo test --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> cargo build --workspace"
cargo build --workspace --exclude aether-boot
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> cargo build UEFI boot loader (release)"
cargo build -p aether-boot --target x86_64-unknown-uefi --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> cargo build bare-metal kernel (release)"
cargo build -p aether-kernel --no-default-features --features bare-metal `
    --target x86_64-unknown-none --release `
    -Z build-std=core,alloc,compiler_builtins `
    -Z build-std-features=compiler-builtins-mem
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ""
Write-Host "ci-check: all gates passed."
