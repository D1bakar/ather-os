# PowerShell build helper for Aether OS (Windows)

param(
    [Parameter(Position = 0)]
    [ValidateSet("check", "build", "clean", "help")]
    [string]$Command = "help"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

function Show-Help {
    Write-Host @"
Aether OS build helpers

Usage: build.ps1 <command>

Commands:
  check       Run fmt, clippy, and tests
  build       Build workspace crates
  clean       Remove build artifacts
  help        Show this message
"@
}

function Invoke-Check {
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    cargo clippy --workspace --exclude aether-boot --all-targets -- -D warnings
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    cargo clippy -p aether-boot --target x86_64-unknown-uefi -- -D warnings
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    cargo test --workspace
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    cargo build --workspace
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    Write-Host "All checks passed."
}

function Invoke-Build {
    cargo build --workspace
}

function Invoke-Clean {
    cargo clean
    if (Test-Path "build") { Remove-Item -Recurse -Force "build" }
}

switch ($Command) {
    "check" { Invoke-Check }
    "build" { Invoke-Build }
    "clean" { Invoke-Clean }
    "help"  { Show-Help }
}
