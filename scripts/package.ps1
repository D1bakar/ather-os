# Package distributable release archive with ESP layout + README.

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$VersionLine = Select-String -Path (Join-Path $Root "Cargo.toml") -Pattern '^version = ' | Select-Object -First 1
$Version = ($VersionLine.Line -replace 'version = "(.*)"', '$1')

$Dist = Join-Path $Root "dist"
$Staging = Join-Path $Dist "aether-os-$Version"
$PkgZip = Join-Path $Dist "aether-os-$Version.zip"
$EspTar = Join-Path $Dist "aether-os-$Version-esp.tar.gz"

$BootEfi = Join-Path $Root "build\esp\EFI\BOOT\BOOTX64.EFI"
$KernelElf = Join-Path $Root "build\esp\aether\kernel.elf"

if (-not (Test-Path $BootEfi) -or -not (Test-Path $KernelElf)) {
    Write-Error "boot artifacts missing — run scripts/build-release.ps1 first"
}

if (Test-Path $Staging) { Remove-Item -Recurse -Force $Staging }
New-Item -ItemType Directory -Force -Path (Join-Path $Staging "EFI\BOOT") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $Staging "aether") | Out-Null
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

Copy-Item -Force $BootEfi (Join-Path $Staging "EFI\BOOT\BOOTX64.EFI")
Copy-Item -Force $KernelElf (Join-Path $Staging "aether\kernel.elf")

@"
Aether OS $Version
====================

Contents:
  EFI/BOOT/BOOTX64.EFI   UEFI boot loader
  aether/kernel.elf      Bare-metal kernel (M2)

Copy the EFI/ and aether/ directories onto a FAT32 ESP, or extract
aether-os-$Version-esp.tar.gz for the same layout.

Boot under QEMU + OVMF — see docs/BUILD.md in the repository.
"@ | Set-Content -Encoding UTF8 (Join-Path $Staging "README.txt")

if (Test-Path $PkgZip) { Remove-Item -Force $PkgZip }
Compress-Archive -Path $Staging -DestinationPath $PkgZip -Force

# tar.gz for Unix consumers (requires tar on Windows 10+)
$EspFlat = Join-Path $Dist "esp-flat"
if (Test-Path $EspFlat) { Remove-Item -Recurse -Force $EspFlat }
New-Item -ItemType Directory -Force -Path $EspFlat | Out-Null
Copy-Item -Recurse -Force (Join-Path $Staging "EFI") $EspFlat
Copy-Item -Recurse -Force (Join-Path $Staging "aether") $EspFlat
Copy-Item -Force (Join-Path $Staging "README.txt") $EspFlat
if (Test-Path $EspTar) { Remove-Item -Force $EspTar }
tar -czf $EspTar -C $EspFlat .

Write-Host "package: created $PkgZip"
Write-Host "package: created $EspTar"
Write-Host "  EFI/BOOT/BOOTX64.EFI"
Write-Host "  aether/kernel.elf"
Write-Host "  README.txt"
