# Initial setup report — 2026-11-05

Target directory: `C:\Users\Raven\Documents\Projects\hardware\esp-wifi-analyzer`
Target chip: classic ESP32 / WROOM-32, Xtensa, 2.4 GHz.

## Installed locally

| Tool | Version |
|---|---|
| rustup | 1.29.3 |
| Host Rust stable | 1.99.1 |
| Xtensa Rust (`esp`) | 1.97.0.0 / rustc 1.97.0-nightly |
| Xtensa GCC | esp-16.1.0_20260609 |
| espup | 0.19.0 |
| esp-generate | 2.0.0 |
| espflash | 4.7.0 |

Tools are under ignored `.tools/`. Visual Studio 2022 Community C++ build tools
were already installed. No global Rust installation was found. The sandbox
rejected writes to the default Cargo directory and Windows link creation, so
installation uses local Cargo/Rustup homes and copied rustup proxies. espup's
final registry update was denied after toolchain extraction; `scripts/env.ps1`
provides the required per-process environment instead. The repeat setup run
and both compiler version checks succeeded.

## Foundation delivered

Two-crate Cargo workspace; no_std domain library; ESP HAL/RTOS/Embassy bootstrap;
radio ownership wrapper; bounded types, command parser, history ring, channel
summary; typed request/reply contract; build/check/setup/flash scripts; host CI;
architecture, CLI, development, and validation documents; Cargo.lock; Git main
branch initialized with no commit or remote.

## Verification

- Workspace formatting: passed.
- Host tests: 3 passed.
- Core Clippy, all targets, warnings denied: passed.
- ESP32 development and release firmware: compiled and linked successfully.
- All PowerShell scripts: syntax validated; setup rerun succeeded.
- Local tool and build directories: excluded by .gitignore.

Firmware has not been flashed or exercised on hardware. Interactive command
input and actual scan execution remain pending; the current image initializes
the radio and async application task and prints an honest foundation banner.
The current generator/dependency set targets ESP32 revision 3.0 and later;
verify the board revision before flashing.
