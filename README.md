# ESP32 Wi-Fi Analyzer

Rust `no_std` firmware for a classic ESP32 / WROOM-32 board, using `esp-hal`,
`esp-radio`, `esp-rtos`, and Embassy. The application is written in Rust;
the radio stack also uses Espressif wireless driver binaries.

## Current state

This is the initial foundation, not the completed v0.1 MVP. It contains:

- A two-crate workspace with a hardware-independent `no_std` library.
- ESP32 startup, heap initialization, scheduler startup, radio initialization,
  and an Embassy application task.
- Bounded models, initial CLI parser, RSSI ring history, and AP channel summaries.
- Host contract tests, build/check/flash scripts, and architecture documentation.

The firmware prints a startup banner at 115200 baud. Serial command input,
executing scans, repeated scans, and rendering reports are the next milestones.
Do not expect typing `scan` to work in this foundation build.

## Windows quick start

The initial installation uses project-local tools under ignored `.tools/`.
The existing Visual Studio C++ build tools provide the host linker.

```powershell
Set-Location C:\Users\Raven\Documents\Projects\hardware\esp-wifi-analyzer
. .\scripts\env.ps1
rustc +stable --version
rustc +esp --version
.\scripts\check.ps1
.\scripts\build.ps1 -Release
```

After connecting a board, inspect the available COM ports and flash explicitly:

```powershell
[System.IO.Ports.SerialPort]::GetPortNames()
.\scripts\flash.ps1 -Port COM5
```

COM5 is an example. Flashing overwrites application firmware on that board.
The scaffold has not been flashed or tested on hardware by this setup task.

The current Espressif generator targets ESP32 revision 3.0 and later. Confirm
the actual chip revision with `espflash board-info --port COM5` before flashing;
older WROOM-32 boards need a separately validated compatible dependency set.

For a fresh checkout run `.\scripts\setup.ps1`, then dot-source `env.ps1`.
The scripts change the current process environment, not the user's global PATH.
Keep `Cargo.lock` committed and use the tested tool versions listed in
`docs/development.md`.

## Layout

```text
crates/analyzer-core/  Domain models, parser, analysis, host tests
firmware/             Board startup, platform adapters, application tasks
scripts/              Local tooling, checks, build, explicit flashing
docs/                 Architecture, CLI contract, development and validation
```

See [architecture](docs/architecture.md), [CLI contract](docs/cli.md),
[development](docs/development.md), [validation](docs/validation.md),
and the [initial setup report](docs/setup-report.md).
