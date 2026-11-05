# Contributing to ESP32 Wi-Fi Analyzer

Contributions can improve firmware, tests, hardware validation, documentation,
or development tooling. Search existing issues before starting. Discuss larger
features or architecture changes in an issue so the scope and hardware needs
are clear. Small fixes can go directly to a pull request.

Participation is covered by the [Code of Conduct](CODE_OF_CONDUCT.md).

## Project context

- [README](README.md): supported hardware, features, and quick start.
- [Architecture](docs/architecture.md): task ownership, memory, and errors.
- [Development setup](docs/development.md): standard Rust and ESP32 toolchains.
- [CLI contract](docs/cli.md): commands and serial behavior.
- [Validation](docs/validation.md): automated checks and hardware evidence.

The current target is a classic ESP32 with revision 3.0 or later. Basic hardware
acceptance has been exercised on revision 3.1 with 4 MB flash. Configured scan
channels are 1..11. Other chips or channel sets need explicit implementation
and validation; do not assume every ESP32-family board is supported.

## Issues and feature requests

Use the GitHub issue forms for a [bug report](https://github.com/pravennn08/esp-wifi-analyzer/issues/new?template=bug_report.yml)
or [feature request](https://github.com/pravennn08/esp-wifi-analyzer/issues/new?template=feature_request.yml).

For bugs, provide the commit hash, board/chip revision, tool versions, exact
commands, expected behavior, and a short relevant log. Firmware version alone
does not identify a build while several commits share the same version.

Redact passwords, private network names, BSSIDs, and device MAC addresses from
public logs. Preserve channel, RSSI, timing, and error fields when they are
needed to understand the behavior. Avoid posting an entire unrelated scan dump.

For features, explain the use case, proposed behavior, and acceptance criteria.
Call out known memory, flash, radio scheduling, or additional hardware costs.
Keep estimates and unknowns explicit.

## Prepare a change

1. Fork or clone the repository and create a branch for one focused change.
2. Follow the [development guide](docs/development.md) to install standard Rust
   and, for firmware work, the pinned ESP32 Xtensa tools.
3. Make the change in the appropriate crate. Add regression coverage for changed
   parsing, scheduling, history, or report behavior.
4. Update relevant documentation when commands, capacities, setup, or supported
   hardware change.
5. Add a concise entry under `Unreleased` in [CHANGELOG.md](CHANGELOG.md) for
   user-facing changes. Describe the resulting behavior and any compatibility
   limits; do not add a release date or version tag before an actual release.

## Code conventions

- Keep `analyzer-core` hardware-independent, `no_std`, allocation-free, and free
  of unsafe code. Keep HAL and radio-driver types inside `firmware`.
- Retain one owner for each peripheral and mutable application state. Use
  bounded messages across tasks; do not hold a mutex across an await.
- Specify buffer capacities and overflow behavior. Do not add unbounded
  collections, unchecked input, or heap allocation to command/report paths.
- Return typed errors for invalid input and recoverable failures. Do not panic
  on malformed commands or expected driver errors.
- Do not cancel an active radio scan when processing a stop request. Stop future
  repetitions and let the current scan finish safely.
- Treat network names as untrusted data. Preserve metadata limitations and escape
  terminal control bytes when rendering observations.
- Describe AP counts as observations or congestion indicators. Do not present
  them as measured airtime, throughput, packet loss, or a security audit.
- Use plain-text messages. Do not add emoji to source code, comments, build
  diagnostics, or CLI output.
- Follow rustfmt and Clippy. Keep dependency changes deliberate and include
  the updated `Cargo.lock` when dependencies change.

## Run checks

Host-only changes can be checked without an ESP32 toolchain:

```powershell
cargo +stable fmt --all -- --check
cargo +stable test -p analyzer-core --locked
cargo +stable clippy -p analyzer-core --all-targets --locked -- -D warnings
```

For firmware changes, also run the combined checks and release build:

```powershell
.\scripts\check.ps1
.\scripts\build.ps1 -Release
```

The combined script includes host checks, the ESP32 development build, and
firmware Clippy. GitHub Actions runs host checks on Linux and host/firmware
checks plus release linking on Windows. Do not claim remote CI passed based
only on local checks.

For documentation or issue-template changes, check links, spelling, and YAML
syntax as applicable. A firmware rebuild is unnecessary unless behavior or
build configuration also changes.

## Hardware validation

Close other serial monitors before flashing. Choose your board's actual port:

```powershell
.\scripts\flash.ps1 -Port COM3
```

Flashing replaces the application on that board. Use relevant cases in
[validation.md](docs/validation.md), including error and stop behavior for changes
that affect scanning or watch scheduling.

Record the board, chip revision, commit, commands, duration, and results.
Separate host-tested behavior from board-tested behavior and call out anything
not tested. A successful build does not establish radio behavior, long-run
stability, or support for another board.

## Pull requests

Explain the problem and the resulting behavior. Reference related issues,
describe how the change was validated, and mention remaining limitations.
For CLI changes, include a short before/after command example. For resource
changes, include measured or estimated RAM/flash effects and state which they are.

Keep generated build artifacts, local tool installations, credentials, and
unrelated formatting changes out of the pull request. Keep discussion focused
and respectful, and respond to review with the same validation evidence used
to support the original change.
