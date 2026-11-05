# Changelog

This file records user-facing changes to ESP32 Wi-Fi Analyzer. Changes for the
next release are collected under `Unreleased`. Release versions correspond to
Git tags using the `vMAJOR.MINOR.PATCH` format.

## [Unreleased]

## [0.1.0] - 2026-11-05

Initial MVP release for the classic ESP32 / WROOM-32.

### Added

- Rust `no_std` firmware for the classic ESP32 using `esp-hal`, `esp-radio`,
  `esp-rtos`, and Embassy, with a separate hardware-independent analyzer core.
- Interactive UART0 CLI at 115200 baud with `help` and `status`.
- `scan` for fresh network observations, `scan --channel N` for one configured
  channel, and `scan --passive` for passive discovery. Channel and passive flags
  can be combined.
- Network reports with SSID, BSSID, channel, RSSI, advertised security,
  scan duration, coverage, observation age, and incomplete-result warnings.
- `networks` for the last successful cached snapshot without rescanning.
- `watch --interval 10s --count 3` for scheduled scans without overlap; omitted
  count runs continuously until stopped. Missed scheduling slots are skipped.
- `stop`, Ctrl+X, and forwarded Ctrl+C to stop future watch attempts while the
  current radio scan finishes safely.
- `history <BSSID>` with bounded RSSI samples, min/average/max, observation ages,
  eligible missed-scan counts, and deterministic eviction of old AP records.
- `channels` for AP-presence summaries from a full scan, with explicit refusal
  to treat a filtered scan as full channel coverage.
- Status telemetry for uptime, scanner/radio state, successful and failed scan
  counts, cached APs, heap usage, UART errors, watch state, and history capacity.
- Bounded command input with CR/LF/CRLF handling, Backspace/Delete editing,
  Ctrl+U line clearing, and errors for invalid or oversized input.
- Windows build, check, and explicit flash scripts using the standard Rust
  installation and the ESP32 Xtensa toolchain.
- GitHub Actions jobs for Linux host checks and Windows ESP32 firmware checks
  and release linking.
- Architecture, development, CLI, and hardware-validation documentation.
- GitHub bug-report and feature-request forms, contributor guidelines, and a
  project code of conduct.
- Root security policy covering maintained builds, vulnerability reporting,
  disclosure handling, and firmware trust boundaries.
- Apache-2.0 project licensing with the official license text in `LICENSE.md`,
  inherited Cargo package metadata, SPDX source headers, and contribution terms.
- Pull request template, consistent Git line-ending attributes, troubleshooting
  guide, and monthly dependency-update proposals with explicit ESP stack review.

### Changed

- Replaced the initial project-local Rust installation with the standard
  per-user Cargo/Rustup installation; removed the local setup and activation
  scripts. The build script discovers the ESP32 linker automatically.
- Separated console I/O, application state/scheduling, and radio operations into
  tasks connected by bounded queues. Cached queries remain available during scans.

### Fixed

- An urgent stop now invalidates an older queued watch request so it cannot
  start after the stop is processed.
- Failed scans preserve the last successful snapshot. Filtered or incomplete
  scans do not establish that unrelated APs have disappeared.
- Overlong commands are discarded as a whole instead of executing a truncated
  prefix; CRLF does not submit a command twice.
- Network names are escaped before terminal output. Names truncated by the
  pinned driver's public UTF-8 API receive an explicit unavailable marker.
- Formatting and Clippy findings that blocked the CLI integration checks.
- Removed emoji from generated project build diagnostics.

### Validation

- Thirty-three host tests cover parsing, line editing, scheduling, history,
  strongest-first bounded scan retention, report limits, and stop ordering.
- Host and firmware linting, development/release firmware builds, and workflow
  validation passed locally during integration.
- The user verified boot, help/status, watch/stop, active/channel/passive scans,
  cached results, and channel summaries on an ESP32 revision 3.1 with 4 MB flash.
- Recorded scan durations include 1346 ms for a full active scan, 306 ms for a
  channel-6 scan, and 3340 ms for a full passive scan. These are observations
  from one setup, not performance guarantees.

### Known limitations

- Scans cover configured channels 1..11 on the classic ESP32; support for other
  chips and channel sets has not been established.
- Each snapshot retains up to 32 APs. RAM history tracks up to 32 APs with 16
  samples each and resets on reboot.
- Channel summaries describe observed AP presence, not measured airtime,
  throughput, packet loss, or a security audit.
- Long-duration soak, crowded/capped environments, and controlled fault recovery
  need additional hardware evidence. See [validation](docs/validation.md).
- HTTP dashboards, persistent history, and raw packet analysis are not included.

[Unreleased]: https://github.com/pravennn08/esp-wifi-analyzer/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/pravennn08/esp-wifi-analyzer/releases/tag/v0.1.0
