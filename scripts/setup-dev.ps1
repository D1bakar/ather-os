# Prepare a local Aether OS development environment.

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

Write-Host "==> Aether OS developer setup"
Write-Host "    Repository: $Root"
Write-Host ""

if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    Write-Error "rustup not found. Install Rust from https://rustup.rs/ and re-run this script."
}

Write-Host "==> Ensuring pinned toolchain and components"
rustup show active-toolchain
rustup component add rustfmt clippy rust-src llvm-tools-preview
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> Adding cross-compilation targets"
rustup target add x86_64-unknown-uefi x86_64-unknown-none
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ""
Write-Host "==> Optional: QEMU + OVMF (required for 'make run')"
$Qemu = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue
if ($Qemu) {
    Write-Host "    qemu-system-x86_64: $($Qemu.Source)"
} else {
    Write-Host "    qemu-system-x86_64: NOT FOUND"
    Write-Host "    Install QEMU for Windows: https://www.qemu.org/download/#windows"
    Write-Host "    Or via winget: winget install SoftwareFreedomConservancy.QEMU"
}

$OvmfCandidates = @(
    (Join-Path $Root "ovmf\OVMF_CODE.fd"),
    "$env:ProgramFiles\qemu\share\OVMF_CODE.fd",
    "$env:ProgramFiles\qemu\share\edk2\x64\OVMF_CODE.fd",
    "$env:ProgramFiles\qemu\share\OVMF\OVMF_CODE.fd"
)
$OvmfFound = $false
foreach ($path in $OvmfCandidates) {
    if (Test-Path $path) {
        Write-Host "    OVMF_CODE.fd: $path"
        $OvmfFound = $true
        break
    }
}
if (-not $OvmfFound) {
    Write-Host "    OVMF_CODE.fd: NOT FOUND - copy OVMF_CODE.fd and OVMF_VARS.fd to ovmf/"
}

Write-Host ""
Write-Host "==> Quick start"
Write-Host "    make build     # host workspace"
Write-Host "    make boot      # UEFI loader + kernel.elf"
Write-Host "    make run       # QEMU smoke test"
Write-Host "    make test      # fmt + clippy + tests"
Write-Host ""
Write-Host "Setup complete."
