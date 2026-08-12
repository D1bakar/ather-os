# One-command Universal Platform launcher (run from repository root: .\web\serve.ps1)
$ErrorActionPreference = "Stop"
$WebRoot = $PSScriptRoot
$RepoRoot = Split-Path -Parent $WebRoot

Push-Location $RepoRoot
try {
    $manifest = Join-Path $WebRoot "public\manifest.json"
    if (-not (Test-Path $manifest)) {
        Write-Host "==> manifest.json missing; building web artifacts"
        & (Join-Path $RepoRoot "scripts\build-web-artifacts.ps1")
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    Push-Location $WebRoot
    if (-not (Test-Path "node_modules")) {
        Write-Host "==> npm install"
        npm install
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    Write-Host "==> Serving http://localhost:8080 (Ctrl+C to stop)"
    npm run serve
} finally {
    Pop-Location
    Pop-Location
}
