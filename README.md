# ESP32 Wi-Fi Analyzer

Rust `no_std` firmware for a classic ESP32 / WROOM-32 board, using `esp-hal`,
`esp-radio`, `esp-rtos`, and Embassy. The application is written in Rust;
the radio stack also uses Espressif wireless driver binaries.

## Current state

The scan CLI is implemented. Basic command acceptance passed on the user's board;
long-duration and fault-injection checks remain:

- A two-crate workspace with a hardware-independent `no_std` library.
- ESP32 startup, heap initialization, scheduler startup, radio initialization,
  and separate Embassy application/console tasks.
- UART0 commands: `help`, `status`, `scan`, `networks`, `watch`, `stop`,
  `history`, and `channels` at 115200 baud.
- Active/passive scans of configured channels 1..11, with results ordered by RSSI.
- Cached snapshots with age/coverage, bounded per-BSSID RSSI history, and channel summaries.
- Non-overlapping repeated scans, bounded command editing, and clear input errors.
- Host contract tests, build/check/flash scripts, and architecture documentation.

At the `wifi>` prompt, enter `help` for syntax. `scan` collects fresh observations;
`networks`, `history`, and `channels` inspect retained observations. `watch`
repeats full scans; `stop` or Ctrl+X prevents further repeats after any active
scan finishes. Use Backspace to edit and Ctrl+U to clear the current line.
Results are observations at the board's location; AP counts do not measure
airtime, throughput, packet loss, or actual channel utilization.

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
wifi> scan
wifi> scan --channel 6 --passive
wifi> networks
wifi> watch --interval 10s --count 3
wifi> history 02:11:22:33:44:55
wifi> channels
wifi> stop
```

The user verified boot, help/status, watch/stop, active/channel/passive scans,
cached networks, and channel summaries on an ESP32 revision 3.1 with 4 MB flash.
See [validation evidence](docs/validation.md) for observed durations and limits
of that acceptance run. Multi-hour soak and controlled-fault checks remain.

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
