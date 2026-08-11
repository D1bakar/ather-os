# Production release build — workspace release + ESP boot artifacts.

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$env:CARGO_TERM_COLOR = "always"
$env:RUSTC_BOOTSTRAP = "1"
if (-not $env:RUSTFLAGS) { $env:RUSTFLAGS = "-Dwarnings" }
if (-not $env:SOURCE_DATE_EPOCH) {
    try {
        $env:SOURCE_DATE_EPOCH = (git log -1 --format=%ct 2>$null)
    } catch {
        $env:SOURCE_DATE_EPOCH = "0"
    }
}

Write-Host "==> Aether OS production build"
Write-Host "    SOURCE_DATE_EPOCH=$($env:SOURCE_DATE_EPOCH)"

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

Write-Host "==> cargo build --workspace --release"
cargo build --workspace --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> boot artifacts (ESP layout)"
& (Join-Path $Root "scripts\build-boot.ps1")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ""
Write-Host "build-release: complete — ESP at build\esp\"
