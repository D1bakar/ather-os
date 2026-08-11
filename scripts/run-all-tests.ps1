# Run the full host test matrix and optional headless QEMU smoke (non-interactive).
param(
    [switch]$Qemu
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$env:CARGO_TERM_COLOR = "always"
if (-not $env:RUSTFLAGS) { $env:RUSTFLAGS = "-Dwarnings" }

Write-Host "==> cargo test (workspace)"
cargo test --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> property + fuzz + integration targets"
cargo test --manifest-path tests/Cargo.toml `
    --test property_abi --test property_gdt --test property_vfs `
    --test fuzz_syscall --test fuzz_ramfs --test integration_suite
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if ($Qemu) {
    Write-Host "==> headless QEMU smoke"
    & (Join-Path $Root "scripts\qemu-test.ps1")
    if ($LASTEXITCODE -eq 77) {
        Write-Host "QEMU smoke skipped (tooling not installed)" -ForegroundColor Yellow
    } elseif ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

Write-Host ""
Write-Host "run-all-tests: PASS"
