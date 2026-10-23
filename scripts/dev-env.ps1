# Sets up the toolchain for this shell: dot-source it with `. .\scripts\dev-env.ps1`.
# Defaults match the main dev machine; override any of them by setting the variable before dot-sourcing.

if (-not $env:SWIFTPDF_DEV_ROOT) { $env:SWIFTPDF_DEV_ROOT = 'D:\dev' }

if (-not $env:RUSTUP_HOME) { $env:RUSTUP_HOME = Join-Path $env:SWIFTPDF_DEV_ROOT 'rust\rustup' }
if (-not $env:CARGO_HOME) { $env:CARGO_HOME = Join-Path $env:SWIFTPDF_DEV_ROOT 'rust\cargo' }
if (-not $env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR = Join-Path $env:SWIFTPDF_DEV_ROOT 'swiftpdf-target' }

$nodeDir = Join-Path $env:SWIFTPDF_DEV_ROOT 'node'
$cargoBin = Join-Path $env:CARGO_HOME 'bin'
$env:Path = "$nodeDir;$cargoBin;$env:Path"

Write-Host "node $(node --version), $(cargo --version)"
