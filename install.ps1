# One-command Aether OS developer setup (Windows).
# Usage: .\install.ps1 [-SkipQemu] [-SkipBuild]

param(
    [switch]$SkipQemu,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$Root = $PSScriptRoot
Set-Location $Root

Write-Host "==> Aether OS one-command setup"
Write-Host "    Repository: $Root"
Write-Host ""

if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    Write-Host "==> Rust not found — attempting install via rustup-init"
    $Rustup = Join-Path $env:TEMP "rustup-init.exe"
    if (-not (Test-Path $Rustup)) {
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $Rustup
    }
    & $Rustup -y --default-toolchain none
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" +
        [System.Environment]::GetEnvironmentVariable("Path", "User")
    if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
        Write-Error "rustup still not found after install. Restart your shell and re-run install.ps1."
    }
}

& (Join-Path $Root "scripts\setup-dev.ps1")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (-not $SkipQemu) {
    $Qemu = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue
    if (-not $Qemu) {
        Write-Host ""
        Write-Host "==> QEMU not found — attempting winget install"
        if (Get-Command winget -ErrorAction SilentlyContinue) {
            winget install --id SoftwareFreedomConservancy.QEMU -e --accept-source-agreements --accept-package-agreements
            $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" +
                [System.Environment]::GetEnvironmentVariable("Path", "User")
        } else {
            Write-Host "    winget unavailable — install QEMU manually: https://www.qemu.org/download/#windows"
        }
    }
}

if (-not $SkipBuild) {
    Write-Host ""
    Write-Host "==> Building host workspace and developer tools"
    cargo build --workspace
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    cargo build -p aether-serial -p aether-img-builder --release
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host ""
Write-Host "==> Setup complete"
Write-Host "    make boot       # UEFI loader + kernel.elf"
Write-Host "    make run        # QEMU smoke test"
Write-Host "    make image      # build/aether.img from ESP"
Write-Host "    make serial     # follow build/qemu-serial.log"
Write-Host ""
