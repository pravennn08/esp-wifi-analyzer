# ESP32 Wi-Fi Analyzer

Rust `no_std` firmware for a classic ESP32 / WROOM-32 board, using `esp-hal`,
`esp-radio`, `esp-rtos`, and Embassy. The application is written in Rust;
the radio stack also uses Espressif wireless driver binaries.

## Current state

The serial CLI milestone is implemented; the full v0.1 MVP is still in progress:

- A two-crate workspace with a hardware-independent `no_std` library.
- ESP32 startup, heap initialization, scheduler startup, radio initialization,
  and separate Embassy application/console tasks.
- Interactive `help` and `status` commands on UART0 at 115200 baud.
- Bounded command editing, clear input errors, and asynchronous status snapshots.
- Core models, RSSI ring history, and AP summaries ready for the scan milestone.
- Host contract tests, build/check/flash scripts, and architecture documentation.

At the `wifi>` prompt, enter `help` or `status`. Status reports monotonic uptime,
radio/scanner state, actual allocator usage, scan counters, and UART receive
errors. Scan execution is the next milestone; `scan` currently reports that it
is unavailable. Use Backspace to edit and Ctrl+U to clear the current line.

## Windows quick start

Use your standard Windows Rust installation and the ESP32 Xtensa toolchain.
The Visual Studio C++ build tools provide the host linker. Follow
[development setup](docs/development.md) for a fresh machine.

```powershell
Set-Location C:\Users\Raven\Documents\Projects\hardware\esp-wifi-analyzer
rustc +stable --version
rustc +esp --version
.\scripts\check.ps1
.\scripts\build.ps1 -Release
```

After connecting a board, inspect the available COM ports and flash explicitly:

```powershell
[System.IO.Ports.SerialPort]::GetPortNames()
.\scripts\flash.ps1 -Port COM3
```

COM3 is this board's observed port; verify it if Windows assigns a different one.
Close any existing serial monitor before flashing. Once the monitor starts,
type commands there, rather than at the PowerShell prompt:

```text
wifi> help
wifi> status
```

The previous foundation image booted successfully on the user's ESP32 revision
3.1 with 4 MB flash. The new interactive CLI still needs its board acceptance
checks. The implementation task builds the image without flashing the board.

The current Espressif generator targets ESP32 revision 3.0 and later. Confirm
the actual chip revision with `espflash board-info --port COM3` before flashing;
older WROOM-32 boards need a separately validated compatible dependency set.

No activation script is required. The build script discovers the installed
ESP32 linker through rustup and configures it for the current process.
Keep `Cargo.lock` committed and use the tested tool versions listed in
`docs/development.md`.

## Layout

```text
crates/analyzer-core/  Domain models, parser, analysis, host tests
firmware/             Board startup, platform adapters, application tasks
scripts/              Checks, build, explicit flashing
docs/                 Architecture, CLI contract, development and validation
```

See [architecture](docs/architecture.md), [CLI contract](docs/cli.md),
[development](docs/development.md), [validation](docs/validation.md),
and the [initial setup report](docs/setup-report.md).
