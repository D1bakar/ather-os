# Headless QEMU boot test — no user input required.
param(
    [int]$TimeoutSeconds = 35,
    [switch]$NoBuild,
    [switch]$SkipQemu
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

if ($SkipQemu) {
    Write-Host "qemu-test: skipped (--SkipQemu)"
    exit 0
}

$Qemu = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue
if (-not $Qemu) {
    Write-Host "qemu-test: SKIP — qemu-system-x86_64 not in PATH" -ForegroundColor Yellow
    exit 77
}

$args = @("-TimeoutSeconds", $TimeoutSeconds)
if ($NoBuild) { $args += "-NoBuild" }

& (Join-Path $Root "scripts\run-qemu.ps1") @args
exit $LASTEXITCODE
