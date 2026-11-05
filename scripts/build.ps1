param([switch]$Release)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\env.ps1"
Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    $buildArgs = @('+esp', 'esp-build', '--locked')
    if ($Release) { $buildArgs += '--release' }
    & cargo @buildArgs
    if ($LASTEXITCODE -ne 0) { throw 'Firmware build failed' }
} finally { Pop-Location }
