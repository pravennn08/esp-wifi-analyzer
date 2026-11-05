# Dot-source this file: . .\scripts\env.ps1
$projectRoot = Split-Path -Parent $PSScriptRoot
$localTools = Join-Path $projectRoot '.tools'
if (Test-Path -LiteralPath (Join-Path $localTools 'cargo\bin\rustup.exe')) {
    $env:CARGO_HOME = Join-Path $localTools 'cargo'
    $env:RUSTUP_HOME = Join-Path $localTools 'rustup'
    $toolPaths = @((Join-Path $env:CARGO_HOME 'bin'), (Join-Path $localTools 'bin'))
    $toolchainRoot = Join-Path $env:RUSTUP_HOME 'toolchains\esp'
    $gccBin = Join-Path $toolchainRoot 'xtensa-esp-elf\bin'
    $clangBin = Join-Path $toolchainRoot 'xtensa-esp32-elf-clang\esp-clang\bin'
    if (Test-Path -LiteralPath $gccBin) { $toolPaths += $gccBin }
    if (Test-Path -LiteralPath $clangBin) {
        $toolPaths += $clangBin
        $env:LIBCLANG_PATH = Join-Path $clangBin 'libclang.dll'
    }
    # Idempotent within the current process; never persist the user's PATH.
    $env:PATH = (($toolPaths + ($env:PATH -split ';')) |
        Where-Object { $_ } | Select-Object -Unique) -join ';'
}
