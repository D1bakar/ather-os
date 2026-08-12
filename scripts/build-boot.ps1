# Build boot artifacts and populate the ESP directory tree.
# Output: build/esp/EFI/BOOT/BOOTX64.EFI and build/esp/aether/kernel.elf

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$BuildDir = Join-Path $Root "build"
$EspDir = Join-Path $BuildDir "esp"
$BootEfiDir = Join-Path $EspDir "EFI\BOOT"
$KernelDir = Join-Path $EspDir "aether"
$TargetDir = Join-Path $Root "target"
$env:CARGO_TARGET_DIR = $TargetDir

New-Item -ItemType Directory -Force -Path $BootEfiDir | Out-Null
New-Item -ItemType Directory -Force -Path $KernelDir | Out-Null

Write-Host "==> Adding Rust targets"
rustup target add x86_64-unknown-uefi x86_64-unknown-none
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> Building user-space binaries (init.elf)"
& (Join-Path $Root "scripts\build-user.ps1")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> Building UEFI boot loader"
cargo build -p aether-boot --target x86_64-unknown-uefi --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> Building bare-metal kernel"
$env:RUSTC_BOOTSTRAP = "1"
cargo build -p aether-kernel --no-default-features --features bare-metal --target x86_64-unknown-none --release -Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$BootSrc = Join-Path $TargetDir "x86_64-unknown-uefi\release\bootx64.efi"
if (-not (Test-Path $BootSrc)) {
    $BootSrc = Join-Path $TargetDir "x86_64-unknown-uefi\release\bootx64"
}
$KernelSrc = Join-Path $TargetDir "x86_64-unknown-none\release\kernel"
if (-not (Test-Path $KernelSrc)) {
    $KernelSrc = Join-Path $TargetDir "x86_64-unknown-none\release\kernel.exe"
}
$BootDst = Join-Path $BootEfiDir "BOOTX64.EFI"
$KernelDst = Join-Path $KernelDir "kernel.elf"

Copy-Item -Force $BootSrc $BootDst
Copy-Item -Force $KernelSrc $KernelDst

Write-Host "ESP ready at $EspDir"
Write-Host "  $BootDst"
Write-Host "  $KernelDst"
