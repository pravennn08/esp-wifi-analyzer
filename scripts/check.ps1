$ErrorActionPreference = 'Stop'
Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    cargo +stable fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting check failed' }
    cargo +stable test -p analyzer-core --locked
    if ($LASTEXITCODE -ne 0) { throw 'Host tests failed' }
    cargo +stable clippy -p analyzer-core --all-targets --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Host lint failed' }
    & "$PSScriptRoot\build.ps1"
    cargo +esp clippy -p analyzer-firmware --target xtensa-esp32-none-elf '-Zbuild-std=core,alloc' --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Firmware lint failed' }
} finally { Pop-Location }
