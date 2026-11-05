# Setup report â€” 2026-11-05

Target directory: `C:\Users\Raven\Documents\Projects\hardware\esp-wifi-analyzer`
Target chip: classic ESP32 / WROOM-32, Xtensa, 2.4 GHz.

## Standard per-user installation

| Tool | Version | Location |
|---|---|---|
| Host Rust stable | 1.99.1 | %USERPROFILE%\.rustup |
| Xtensa Rust (`esp`) | 1.97.0.0 / rustc 1.97.0-nightly | %USERPROFILE%\.rustup\toolchains\esp |
| Xtensa GCC | esp-16.1.0_20260609 | Inside the esp toolchain |
| espup | 0.19.0 | %USERPROFILE%\.cargo\bin |
| espflash | 4.7.0 | %USERPROFILE%\.cargo\bin |
| esp-generate (optional) | 2.0.0 | %USERPROFILE%\.cargo\bin |

The user installed standard Rust after the initial project bootstrap. The
previously validated ESP32 compiler, linker, and Espressif CLI binaries were
migrated into that standard installation. Existing stable Rust was preserved.
Visual Studio 2022 Community C++ build tools provide the Windows host linker.

The project no longer installs or activates a private Rust environment. The
local installer and activation scripts were removed. Build/check/flash remain;
the build script discovers the installed esp toolchain using rustup and adds
its linker/Clang paths to the current process environment as needed. It does
not redirect Cargo/Rustup homes or write global registry settings.

## Foundation delivered

Two-crate Cargo workspace; no_std domain library; ESP HAL/RTOS/Embassy bootstrap;
radio ownership wrapper; bounded types, command parser, history ring, channel
summary; typed request/reply contract; build/check/flash scripts; host CI;
architecture, CLI, development, and validation documents; Cargo.lock.

## Validation

Validation using the standard per-user Rust installation:

- Formatting: passed.
- Host contract tests: 3 passed.
- Core Clippy with warnings denied: passed.
- All remaining PowerShell scripts: syntax checks passed.
- ESP32 development and release builds: compiled and linked successfully.
- Duplicate project-local tool directory: removed after successful builds.
- Release build repeated offline after tool removal: passed.
- rustup selects the esp compiler under %USERPROFILE%\.rustup\toolchains\esp.
- No local activation or installation script is needed.

The user subsequently flashed the foundation successfully on COM3 and confirmed
boot/radio initialization on an ESP32 revision 3.1 with 4 MB flash.
The help/status CLI milestone is now implemented, with 12 host tests passing
and development/release firmware builds linked successfully. Its board checks
are pending; no new image was flashed by the implementation task. Actual scan
execution remains the next milestone.
The current dependency set targets ESP32 revision 3.0 and later; verify the
board revision before flashing.
