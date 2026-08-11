# Validate Aether OS update manifest fixtures and updater crate tests.
# M12 skeleton — structural checks only; no network fetch or real crypto.

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$env:CARGO_TERM_COLOR = "always"

Write-Host "==> aether-updater unit tests"
cargo test -p aether-updater
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> manifest fixture validation"

$FixtureDir = Join-Path $Root "system\updater\fixtures"
if (-not (Test-Path $FixtureDir)) {
    New-Item -ItemType Directory -Path $FixtureDir -Force | Out-Null
}

$SampleManifest = Join-Path $FixtureDir "sample-manifest.json"
if (-not (Test-Path $SampleManifest)) {
    @'
{
  "magic": "AETHUPD!",
  "version": 1,
  "target_slot": "B",
  "payload_kind": "KernelElf",
  "algorithm": "Ed25519",
  "payload_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
  "key_id": "0000000000000000",
  "release_version": "0.2.0-dev",
  "signature": "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
}
'@ | Set-Content -Path $SampleManifest -Encoding UTF8
    Write-Host "    created sample manifest: $SampleManifest"
}

$json = Get-Content -Raw -Path $SampleManifest | ConvertFrom-Json

$RequiredFields = @(
    "magic", "version", "target_slot", "payload_kind", "algorithm",
    "payload_sha256", "key_id", "release_version", "signature"
)
foreach ($field in $RequiredFields) {
    if (-not (Get-Member -InputObject $json -Name $field -MemberType NoteProperty)) {
        Write-Error "manifest missing required field: $field"
    }
}

if ($json.magic -ne "AETHUPD!") {
    Write-Error "invalid manifest magic: $($json.magic)"
}

if ($json.version -lt 1) {
    Write-Error "manifest version must be >= 1"
}

if ($json.target_slot -notin @("A", "B")) {
    Write-Error "target_slot must be A or B"
}

if ($json.payload_sha256.Length -ne 64) {
    Write-Error "payload_sha256 must be 64 hex characters"
}

if ($json.signature.Length -ne 128) {
    Write-Error "signature must be 128 hex characters (64 bytes)"
}

Write-Host "    manifest structure: OK ($SampleManifest)"

Write-Host ""
Write-Host "update-check: all checks passed."
