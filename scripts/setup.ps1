# Installs into the ignored .tools directory. Does not flash a board.
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$toolRoot = Join-Path $projectRoot '.tools'
$binRoot = Join-Path $toolRoot 'bin'
$downloads = Join-Path $toolRoot 'downloads'
New-Item -ItemType Directory -Path $binRoot, $downloads -Force | Out-Null
$env:CARGO_HOME = Join-Path $toolRoot 'cargo'
$env:RUSTUP_HOME = Join-Path $toolRoot 'rustup'
New-Item -ItemType Directory -Path "$env:CARGO_HOME\bin" -Force | Out-Null

function Get-Tool([string]$Url, [string]$Destination) {
    if (!(Test-Path -LiteralPath $Destination)) {
        Invoke-WebRequest -Uri $Url -OutFile $Destination
    }
}

# Copying rustup proxies works on restricted Windows filesystems where creating
# hardlinks/symlinks is denied. The official rustup executable dispatches by name.
$rustup = Join-Path "$env:CARGO_HOME\bin" 'rustup.exe'
Get-Tool 'https://win.rustup.rs/x86_64' $rustup
foreach ($proxy in @('cargo','rustc','rustdoc','rustfmt','cargo-fmt',
    'clippy-driver','cargo-clippy','rust-analyzer','rust-gdb','rust-gdbgui','rust-lldb')) {
    $destination = Join-Path "$env:CARGO_HOME\bin" "$proxy.exe"
    if (!(Test-Path -LiteralPath $destination)) {
        Copy-Item -LiteralPath $rustup -Destination $destination
    }
}
. "$PSScriptRoot\env.ps1"
& $rustup set auto-self-update disable
if ($LASTEXITCODE -ne 0) { throw 'Could not configure local rustup' }
& $rustup toolchain install stable --profile minimal --component rustfmt --component clippy
if ($LASTEXITCODE -ne 0) { throw 'Host Rust installation failed' }
& $rustup default stable
if ($LASTEXITCODE -ne 0) { throw 'Could not select host Rust toolchain' }

Get-Tool 'https://github.com/esp-rs/espup/releases/download/v0.19.0/espup-x86_64-pc-windows-msvc.exe' (Join-Path $binRoot 'espup.exe')
Get-Tool 'https://github.com/esp-rs/esp-generate/releases/download/v2.0.0/esp-generate-x86_64-pc-windows-msvc.exe' (Join-Path $binRoot 'esp-generate.exe')
if (!(Test-Path -LiteralPath (Join-Path $binRoot 'espflash.exe'))) {
    $archive = Join-Path $downloads 'espflash-4.7.0.zip'
    Get-Tool 'https://github.com/esp-rs/espflash/releases/download/v4.7.0/espflash-x86_64-pc-windows-msvc.zip' $archive
    Expand-Archive -LiteralPath $archive -DestinationPath $binRoot -Force
}

$compiler = Join-Path $env:RUSTUP_HOME 'toolchains\esp\bin\rustc.exe'
$linker = Join-Path $env:RUSTUP_HOME 'toolchains\esp\xtensa-esp-elf\bin\xtensa-esp32-elf-gcc.exe'
if (!(Test-Path -LiteralPath $compiler) -or !(Test-Path -LiteralPath $linker)) {
    $output = & espup install --targets esp32 --toolchain-version 1.97.0.0 --crosstool-toolchain-version 16.1.0_20260609 --export-file (Join-Path $toolRoot 'export-esp.ps1') 2>&1
    $installExit = $LASTEXITCODE
    $output | ForEach-Object { Write-Host $_ }
    if ($installExit -ne 0) {
        # espup can finish downloads then fail its Windows user-registry update.
        # Accept only that specific failure and verify both installed executables.
        if (!(Test-Path -LiteralPath $compiler) -or !(Test-Path -LiteralPath $linker) -or
            (($output -join "`n") -notmatch 'Access is denied')) {
            throw "ESP32 toolchain installation failed (exit $installExit)"
        }
        Write-Warning 'espup could not persist its Windows environment; env.ps1 supplies the local environment instead.'
    }
}
. "$PSScriptRoot\env.ps1"
rustc +stable --version
if ($LASTEXITCODE -ne 0) { throw 'Host compiler validation failed' }
rustc +esp --version
if ($LASTEXITCODE -ne 0) { throw 'ESP32 compiler validation failed' }
& $linker --version
if ($LASTEXITCODE -ne 0) { throw 'ESP32 linker validation failed' }
espflash --version
if ($LASTEXITCODE -ne 0) { throw 'espflash validation failed' }
Write-Host 'Tooling ready. Dot-source scripts\env.ps1 in each new shell, then run scripts\check.ps1.'
