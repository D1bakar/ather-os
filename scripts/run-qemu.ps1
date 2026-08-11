# Launch Aether OS in QEMU with OVMF and capture serial output.
param(
    [int]$TimeoutSeconds = 35,
    [switch]$NoBuild
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

if (-not $NoBuild) {
    & (Join-Path $Root "scripts\build-boot.ps1")
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$EspDir = Join-Path $Root "build\esp"
if (-not (Test-Path (Join-Path $EspDir "EFI\BOOT\BOOTX64.EFI"))) {
    Write-Error "ESP not found. Run scripts/build-boot.ps1 first."
}

$Qemu = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue
if (-not $Qemu) {
    Write-Error "qemu-system-x86_64 not found in PATH."
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

$OvmfCode = Find-OvmfFile "OVMF_CODE.fd"
if (-not $OvmfCode) { $OvmfCode = Find-OvmfFile "OVMF_CODE.4MB.fd" }
$OvmfVars = Find-OvmfFile "OVMF_VARS.fd"
if (-not $OvmfVars) { $OvmfVars = Find-OvmfFile "OVMF_VARS.4MB.fd" }

if (-not $OvmfCode -or -not $OvmfVars) {
    Write-Error @"
OVMF firmware not found. Install QEMU/OVMF and place files under ovmf/ or system paths.
Expected OVMF_CODE.fd and OVMF_VARS.fd (or 4MB variants).
"@
}

$VarsCopy = Join-Path $Root "build\OVMF_VARS.runtime.fd"
Copy-Item -Force $OvmfVars $VarsCopy

$LogFile = Join-Path $Root "build\qemu-serial.log"
if (Test-Path $LogFile) { Remove-Item -Force $LogFile }

Write-Host "==> Starting QEMU (timeout ${TimeoutSeconds}s)"
$qemuArgs = @(
    "-machine", "q35",
    "-cpu", "max",
    "-m", "256M",
    "-drive", "if=pflash,format=raw,readonly=on,file=$OvmfCode",
    "-drive", "if=pflash,format=raw,file=$VarsCopy",
    "-drive", "format=raw,file=fat:rw:$EspDir",
    "-serial", "file:$LogFile",
    "-display", "none"
)

$proc = Start-Process -FilePath $Qemu.Source -ArgumentList $qemuArgs -PassThru -NoNewWindow
$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 250
    if ($proc.HasExited) { break }
    if ((Test-Path $LogFile) -and (Select-String -Path $LogFile -Pattern "Aether OS M2: GDT/IDT/interrupts initialized" -Quiet)) {
        break
    }
}

if (-not $proc.HasExited) {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
}

if (-not (Test-Path $LogFile)) {
    Write-Error "QEMU did not produce serial output."
}

Write-Host "--- serial log ---"
Get-Content $LogFile
Write-Host "------------------"

if (Select-String -Path $LogFile -Pattern "Aether OS M2: GDT/IDT/interrupts initialized" -Quiet) {
    Write-Host "QEMU boot smoke test: PASS"
    exit 0
}

Write-Error "Expected serial output not found: 'Aether OS M2: GDT/IDT/interrupts initialized'"
