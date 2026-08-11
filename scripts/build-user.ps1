# Build Aether OS user-space binaries (host + bare-metal cross-compile).
# Output: build/user/init.elf, build/user/shell.elf

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$BuildDir = Join-Path $Root "build\user"
$TargetDir = Join-Path $Root "target"
$LinkerScript = Join-Path $Root "user\linker.ld"
$env:CARGO_TARGET_DIR = $TargetDir

Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue

New-Item -ItemType Directory -Force -Path $BuildDir | Out-Null

Write-Host "==> Building user-space crates (host)"
cargo build -p aether-rt -p aether-init -p aether-shell --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> Running user-space host tests"
cargo test -p aether-rt -p aether-init -p aether-shell
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> Adding bare-metal target"
rustup target add x86_64-unknown-none
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> Cross-compiling init + shell (x86_64-unknown-none)"
$env:RUSTC_BOOTSTRAP = "1"
$UserRustflags = "-C link-arg=-T$LinkerScript -C relocation-model=static"
$env:RUSTFLAGS = $UserRustflags

cargo build -p aether-init --no-default-features --features bare-metal `
    --target x86_64-unknown-none --release `
    -Z build-std=core,compiler_builtins `
    -Z build-std-features=compiler-builtins-mem
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo build -p aether-shell --no-default-features --features bare-metal `
    --target x86_64-unknown-none --release `
    -Z build-std=core,compiler_builtins `
    -Z build-std-features=compiler-builtins-mem
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue

$InitSrc = Join-Path $TargetDir "x86_64-unknown-none\release\init"
if (-not (Test-Path $InitSrc)) {
    $InitSrc = Join-Path $TargetDir "x86_64-unknown-none\release\init.exe"
}
$ShellSrc = Join-Path $TargetDir "x86_64-unknown-none\release\shell"
if (-not (Test-Path $ShellSrc)) {
    $ShellSrc = Join-Path $TargetDir "x86_64-unknown-none\release\shell.exe"
}

$InitDst = Join-Path $BuildDir "init.elf"
$ShellDst = Join-Path $BuildDir "shell.elf"

Copy-Item -Force $InitSrc $InitDst
Copy-Item -Force $ShellSrc $ShellDst

Write-Host "User binaries ready at $BuildDir"
Write-Host "  $InitDst"
Write-Host "  $ShellDst"
Write-Host ""
Write-Host "Status: host builds runnable; bare-metal ELFs parsed by kernel stub."
Write-Host "        Ring-3 execution blocked until M5 paging/syscalls ship."
