# Copy boot artifacts into web/public/ and emit manifest.json with SHA-256 checksums.
# Prerequisite: scripts/build-boot.ps1 (or existing build/esp/).

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$EspDir = Join-Path $Root "build\esp"
$WebArtifacts = Join-Path $Root "web\public\artifacts"
$ManifestOut = Join-Path $Root "web\public\manifest.json"
$Template = Join-Path $Root "tools\release-manifest.template.json"

$Required = @(
    @{ Rel = "EFI\BOOT\BOOTX64.EFI"; Role = "uefi-boot-loader" },
    @{ Rel = "aether\kernel.elf"; Role = "kernel" }
)

function Get-Sha256Hex {
    param([string]$Path)
    $hash = Get-FileHash -Algorithm SHA256 -Path $Path
    return $hash.Hash.ToLowerInvariant()
}

function Get-GitCommit {
    try {
        $sha = git rev-parse HEAD 2>$null
        if ($sha) { return $sha.Trim() }
    } catch { }
    return "unknown"
}

function Get-Version {
    $line = Select-String -Path (Join-Path $Root "Cargo.toml") -Pattern '^version = ' | Select-Object -First 1
    if ($line) { return ($line.Line -replace 'version = "(.*)"', '$1') }
    return "0.0.0"
}

function Find-OvmfFile {
    param([string]$Name)
    $candidates = @(
        (Join-Path $Root "ovmf\$Name"),
        "$env:ProgramFiles\qemu\share\$Name",
        "$env:ProgramFiles\qemu\share\edk2\x64\$Name",
        "$env:ProgramFiles\qemu\share\OVMF\$Name",
        "/usr/share/OVMF/$Name",
        "/usr/share/ovmf/x64/$Name",
        "/usr/share/edk2/ovmf/$Name"
    )
    foreach ($path in $candidates) {
        if (Test-Path $path) { return $path }
    }
    return $null
}

if (-not (Test-Path $EspDir)) {
    Write-Host "==> ESP not found; running build-boot.ps1"
    & (Join-Path $Root "scripts\build-boot.ps1")
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

foreach ($item in $Required) {
    $src = Join-Path $EspDir $item.Rel
    if (-not (Test-Path $src)) {
        Write-Error "Missing required artifact: $src - run scripts/build-boot.ps1"
    }
}

# Build aether.img when img-builder is available and image is missing.
$ImgSrc = Join-Path $Root "build\aether.img"
if (-not (Test-Path $ImgSrc)) {
    Write-Host "==> Building aether.img via aether-img-builder"
    cargo run -q -p aether-img-builder -- build $EspDir $ImgSrc --size-mb 64 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  (img-builder skipped - cargo run failed or crate unavailable)"
    }
}

if (Test-Path $WebArtifacts) {
    Remove-Item -Recurse -Force $WebArtifacts
}
New-Item -ItemType Directory -Force -Path $WebArtifacts | Out-Null

$artifactEntries = @()
foreach ($item in $Required) {
    $src = Join-Path $EspDir $item.Rel
    $dst = Join-Path $WebArtifacts $item.Rel
    $dstDir = Split-Path -Parent $dst
    New-Item -ItemType Directory -Force -Path $dstDir | Out-Null
    Copy-Item -Force $src $dst
    $size = (Get-Item $src).Length
    $relUnix = ($item.Rel -replace '\\', '/')
    $artifactEntries += [ordered]@{
        path       = $relUnix
        role       = $item.Role
        sha256     = (Get-Sha256Hex $src)
        size_bytes = $size
    }
    Write-Host ('  copied {0} ({1} bytes)' -f $relUnix, $size)
}

$optionalEntries = @()
if (Test-Path $ImgSrc) {
    $imgDst = Join-Path $WebArtifacts "aether.img"
    Copy-Item -Force $ImgSrc $imgDst
    $optionalEntries += [ordered]@{
        path       = "aether.img"
        role       = "raw-disk-image"
        sha256     = (Get-Sha256Hex $ImgSrc)
        size_bytes = (Get-Item $ImgSrc).Length
        note       = "FAT32 ESP disk image from aether-img-builder"
    }
    Write-Host "  copied aether.img"
}

# OVMF firmware for in-browser UEFI boot.
$firmwareEntries = @{}
$OvmfCode = Find-OvmfFile "OVMF_CODE.fd"
if (-not $OvmfCode) { $OvmfCode = Find-OvmfFile "OVMF_CODE.4MB.fd" }
$OvmfVars = Find-OvmfFile "OVMF_VARS.fd"
if (-not $OvmfVars) { $OvmfVars = Find-OvmfFile "OVMF_VARS.4MB.fd" }

$browserStatus = "not_available"
$browserBlocker = "UEFI/OVMF required; v86 SeaBIOS cannot boot BOOTX64.EFI"

if ($OvmfCode -and $OvmfVars) {
    $FwDir = Join-Path $WebArtifacts "firmware"
    New-Item -ItemType Directory -Force -Path $FwDir | Out-Null
    $codeDst = Join-Path $FwDir "OVMF_CODE.fd"
    $varsDst = Join-Path $FwDir "OVMF_VARS.fd"
    Copy-Item -Force $OvmfCode $codeDst
    Copy-Item -Force $OvmfVars $varsDst
    $firmwareEntries = [ordered]@{
        ovmf_code = [ordered]@{
            path       = "artifacts/firmware/OVMF_CODE.fd"
            role       = "uefi-firmware-code"
            sha256     = (Get-Sha256Hex $codeDst)
            size_bytes = (Get-Item $codeDst).Length
        }
        ovmf_vars = [ordered]@{
            path       = "artifacts/firmware/OVMF_VARS.fd"
            role       = "uefi-firmware-vars"
            sha256     = (Get-Sha256Hex $varsDst)
            size_bytes = (Get-Item $varsDst).Length
        }
    }
    $browserStatus = "ready"
    $browserBlocker = $null
    Write-Host "  copied OVMF firmware (browser boot ready)"
} else {
    Write-Host "  OVMF not found - browser boot will remain blocked until firmware is installed"
}

$manifest = [ordered]@{
    schema_version = 1
    product        = "aether-os"
    version        = (Get-Version)
    git_commit     = (Get-GitCommit)
    generated_at   = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    boot           = [ordered]@{
        architecture     = "x86_64"
        firmware         = "uefi"
        layout           = "esp-fat32"
        verified_runtime = @("qemu-system-x86_64+ovmf")
        browser_runtime  = [ordered]@{
            status  = $browserStatus
            target  = "qemu.wasm"
            blocker = $browserBlocker
            adr     = "docs/adr/ADR-0010-browser-vm-architecture.md"
            qemu    = [ordered]@{
                base_url = "https://ktock.github.io/qemu-wasm-demo/images/alpine-x86_64/"
                js       = "out.js"
                version  = "ktock-qemu-wasm-demo-alpine-x86_64"
                license  = "GPL-2.0 (QEMU) - fetched from CDN, not bundled in repo"
            }
            firmware = $firmwareEntries
        }
    }
    artifacts          = $artifactEntries
    optional_artifacts = $optionalEntries
    qemu_smoke         = [ordered]@{
        machine         = "q35"
        memory_mb       = 256
        serial_patterns = @(
            "Aether OS kernel started",
            "Aether OS M6: userland started",
            "Aether init started"
        )
    }
}

$json = $manifest | ConvertTo-Json -Depth 8
$json | Set-Content -Encoding UTF8 $ManifestOut

# Copy VM worker sources into public/ for static hosting (local + GitHub Pages).
$VmSrc = Join-Path $Root "web\vm"
$VmDst = Join-Path $Root "web\public\vm"
if (Test-Path $VmDst) {
    Remove-Item -Recurse -Force $VmDst
}
New-Item -ItemType Directory -Force -Path $VmDst | Out-Null
foreach ($vmFile in @("worker.js", "artifact-loader.js", "qemu-emulator.js")) {
    $src = Join-Path $VmSrc $vmFile
    if (-not (Test-Path $src)) {
        Write-Error "Missing VM source: $src"
    }
    Copy-Item -Force $src (Join-Path $VmDst $vmFile)
    Write-Host ('  copied vm/{0}' -f $vmFile)
}

Write-Host ""
Write-Host "Web artifacts ready:"
Write-Host "  $WebArtifacts"
Write-Host "  $ManifestOut"
Write-Host "  browser_runtime.status = $browserStatus"
if (Test-Path $Template) {
    Write-Host "  template: $Template"
}
