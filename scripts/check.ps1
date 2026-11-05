$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\env.ps1"
Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    cargo +stable fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting check failed' }
    cargo +stable test -p analyzer-core --locked
    if ($LASTEXITCODE -ne 0) { throw 'Host tests failed' }
    cargo +stable clippy -p analyzer-core --all-targets --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Host lint failed' }
    cargo +esp esp-build --locked
    if ($LASTEXITCODE -ne 0) { throw 'Firmware build failed' }
} finally { Pop-Location }
