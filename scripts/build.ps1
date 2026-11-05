param([switch]$Release)
$ErrorActionPreference = 'Stop'

# Use the caller's standard Rust installation. Do not redirect Cargo/Rustup homes.
$compilerPath = & rustup which --toolchain esp rustc
if ($LASTEXITCODE -ne 0) {
    throw 'ESP32 toolchain missing. Follow docs/development.md to install it with espup.'
}
$toolchainRoot = Split-Path -Parent (Split-Path -Parent $compilerPath.Trim())
$gccBin = Join-Path $toolchainRoot 'xtensa-esp-elf\bin'
if (!(Test-Path -LiteralPath (Join-Path $gccBin 'xtensa-esp32-elf-gcc.exe'))) {
    throw 'ESP32 linker missing. Install the no_std ESP32 toolchain using espup; see docs/development.md.'
}
# Discover the relocated linker without depending on espup's registry/PATH updates.
$toolPaths = @($gccBin)
$clangBin = Join-Path $toolchainRoot 'xtensa-esp32-elf-clang\esp-clang\bin'
if (Test-Path -LiteralPath (Join-Path $clangBin 'libclang.dll')) {
    $toolPaths += $clangBin
    $env:LIBCLANG_PATH = Join-Path $clangBin 'libclang.dll'
}
$env:PATH = (($toolPaths + ($env:PATH -split ';')) |
    Where-Object { $_ } | Select-Object -Unique) -join ';'

Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    $buildArgs = @('+esp', 'esp-build', '--locked')
    if ($Release) { $buildArgs += '--release' }
    & cargo @buildArgs
    if ($LASTEXITCODE -ne 0) { throw 'Firmware build failed' }
} finally { Pop-Location }
