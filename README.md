<h1 align="center">ESP32 Wi-Fi Analyzer</h1>

<p align="center">
  Wi-Fi diagnostics for ESP32, built with Rust and Embassy.
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-F74C00?logo=rust&amp;logoColor=white" alt="Rust"></a>
  <a href="https://www.espressif.com/en/products/socs/esp32"><img src="https://img.shields.io/badge/ESP32-E7352C?logo=espressif&amp;logoColor=white" alt="ESP32"></a>
  <a href="https://embassy.dev/"><img src="https://img.shields.io/badge/Embassy-1C7ED6" alt="Embassy"></a>
  <a href="LICENSE.md"><img src="https://img.shields.io/badge/License-Apache--2.0-purple" alt="License: Apache-2.0"></a>
  <a href="https://github.com/pravennn08/esp-wifi-analyzer/actions/workflows/host.yml"><img src="https://github.com/pravennn08/esp-wifi-analyzer/actions/workflows/host.yml/badge.svg?branch=main" alt="CI status"></a>
</p>

<p align="center">
  <a href="#overview">Overview</a> &middot;
  <a href="#quick-start">Quick start</a> &middot;
  <a href="#contributing">Contributing</a> &middot;
  <a href="CHANGELOG.md">Changelog</a> &middot;
  <a href="#security">Security</a> &middot;
  <a href="#license">License</a>
</p>

---

## Overview

ESP32 Wi-Fi Analyzer turns a classic ESP32 / WROOM-32 board into a USB serial
tool for inspecting nearby wireless networks. Discover access points, track
signal strength, inspect channel and security information, and compare recent
observations without additional sensors, displays, or a router login.

The application runs in Rust's `no_std` environment using `esp-hal`, `esp-radio`,
`esp-rtos`, and Embassy. The wireless stack includes Espressif driver binaries.
All commands run on the board through a serial terminal at **115200 baud**.

| Feature             | What it provides                                                                         |
| ------------------- | ---------------------------------------------------------------------------------------- |
| Network discovery   | Active or passive scans across configured channels 1..11, or one selected channel        |
| Network details     | SSID, BSSID, channel, RSSI, and advertised security, ordered by signal strength          |
| Cached observations | The latest successful snapshot with its age, duration, coverage, and completeness        |
| Repeated scans      | Configurable watch interval and count, without overlapping scans                         |
| RSSI history        | Up to 16 samples per tracked AP, with min/average/max and observation ages               |
| Channel summaries   | Observed AP counts and strongest signals from a full scan                                |
| Device diagnostics  | Uptime, radio/scanner state, heap usage, scan counters, and UART errors                  |
| Bounded operation   | Up to 32 APs per snapshot, 32 history entries, and fixed-capacity command/report buffers |

Use `stop` or Ctrl+X to end watch repetition; any active scan finishes safely.
Backspace edits the current command and Ctrl+U clears it. In the espflash
monitor, Ctrl+C exits the monitor.

Channel summaries indicate AP presence at the board's location. They do not
measure airtime, throughput, packet loss, or actual channel utilization.
History is kept in RAM and resets when the board restarts.

See the [architecture](docs/architecture.md) for the code structure,
[CLI reference](docs/cli.md) for command details, and
[validation record](docs/validation.md) for hardware results and remaining checks.

## Quick start

### Requirements

- A classic ESP32 development board with silicon revision **3.0 or later**.
  Hardware acceptance has been exercised on revision 3.1 with 4 MB flash.
- A USB data cable and a computer with a serial terminal.
- Standard Rust, the ESP32 Xtensa toolchain, and `espflash`.
- For the Windows commands below: PowerShell and Visual Studio C++ build tools.

Follow the [development setup](docs/development.md) for installation and tested
tool versions. Older ESP32 revisions need a separately validated dependency set.
For common setup and serial issues, see [troubleshooting](docs/troubleshooting.md).

### Build and check

From the repository root:

```powershell
rustc +stable --version
rustc +esp --version
.\scripts\check.ps1
.\scripts\build.ps1 -Release
```

No activation script is required. The build script discovers the installed
ESP32 linker through rustup. Keep `Cargo.lock` aligned with the reviewed source.

### Flash and open the terminal

Connect the board, find its COM port, and inspect its revision:

```powershell
[System.IO.Ports.SerialPort]::GetPortNames()
espflash board-info --port COM3
```

Replace `COM3` with your board's actual port. Close other serial monitors, then
build, flash, and start the monitor:

```powershell
.\scripts\flash.ps1 -Port COM3
```

### Run commands

Type commands at the device's `wifi>` prompt in the serial monitor:

```text
wifi> help
wifi> status
wifi> scan
wifi> networks
wifi> channels
wifi> scan --channel 6 --passive
wifi> watch --interval 10s --count 3
wifi> history 02:11:22:33:44:55
wifi> stop
```

Replace the example BSSID with one returned by your scan. `networks` reads the
cache; `scan` collects fresh observations. `channels` requires a full-scan
snapshot, so run `scan` again after a channel-filtered scan if needed.
See the [CLI contract](docs/cli.md) for options, limits, and editing behavior.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) for issue reporting, development
conventions, validation, and pull request guidance. GitHub issue forms are
available for bug reports and feature requests.

Project participation follows the [Code of Conduct](CODE_OF_CONDUCT.md).
See the [changelog](CHANGELOG.md) for user-facing changes and current limits.

## Security

Read [SECURITY.md](SECURITY.md) for supported builds, vulnerability reporting,
disclosure handling, and the firmware's trust boundaries. Do not post
vulnerability details or sensitive diagnostic data in public issues.

## License

Project code is licensed under the Apache License, Version 2.0. See
[LICENSE.md](LICENSE.md) for the full terms.

Third-party dependencies, generated or adapted upstream code, and Espressif
wireless driver binaries retain their applicable licenses and notices. The
project's license does not replace those terms. Preserve the required upstream
copyright, license, and attribution notices when redistributing them.
