param([Parameter(Mandatory = $true)][string]$Port)
$ErrorActionPreference = 'Stop'
Get-Command espflash -ErrorAction Stop | Out-Null
& "$PSScriptRoot\build.ps1" -Release
$projectRoot = Split-Path -Parent $PSScriptRoot
$image = Join-Path $projectRoot 'target\xtensa-esp32-none-elf\release\esp-wifi-analyzer'
& espflash flash --chip esp32 --port $Port --monitor $image
if ($LASTEXITCODE -ne 0) { throw 'Flashing or serial monitor failed' }
