# Development environment

## Standard Windows Rust installation

Install Rust using the official installer: https://rust-lang.org/tools/install/
Select the default x86_64-pc-windows-msvc stable toolchain. Visual Studio C++
build tools are required for Windows host linking and are installed on this
machine. Open a fresh terminal after installation so it sees Cargo's PATH.

The project uses your regular per-user installation:

```text
%USERPROFILE%\.cargo\bin       cargo, rustc, rustup, and installed CLI tools
%USERPROFILE%\.rustup          stable and custom esp toolchains
```

No project-local Rust installation or manual activation is required. The
build/check/flash scripts use standard commands on PATH; they never set
CARGO_HOME or RUSTUP_HOME. If an old shell still has those variables pointing
into the former local installation, close it and open a fresh terminal.

## ESP32 tools on a fresh machine

Classic ESP32 uses Xtensa and needs the extra esp toolchain in addition to
ordinary stable Rust. The tools have already been migrated on this machine.
For a fresh installation, use these pinned versions:

```powershell
rustup component add rustfmt clippy --toolchain stable
cargo +stable install espup --version 0.19.0 --locked
espup install --targets esp32 --toolchain-version 1.97.0.0 --crosstool-toolchain-version 16.1.0_20260609
cargo +stable install espflash --version 4.7.0 --locked
```

Espressif also provides official prebuilt Windows binaries for espup and
espflash as an alternative to compiling the CLI tools with cargo install:

- https://github.com/esp-rs/espup/releases/tag/v0.19.0
- https://github.com/esp-rs/espflash/releases/tag/v4.7.0
- https://docs.espressif.com/projects/rust/book/getting-started/toolchain.html

esp-generate 2.0.0 created the upstream foundation; it is optional for normal
development because this workspace already exists.

Verify the installation from a fresh terminal:

```powershell
rustc +stable --version
rustc +esp --version
rustup show
espflash --version
```

Tested versions: stable Rust 1.99.1; Xtensa Rust 1.97.0.0;
GCC esp-16.1.0_20260609; espup 0.19.0; espflash 4.7.0.

## Build and check

```powershell
cargo +stable test -p analyzer-core --locked
cargo +stable fmt --all
cargo +stable clippy -p analyzer-core --all-targets --locked -- -D warnings
.\scripts\build.ps1
.\scripts\build.ps1 -Release
.\scripts\check.ps1
```

The root selects stable so tests run natively. The firmware build script uses
cargo +esp and xtensa-esp32-none-elf, building core/alloc from source. It locates
the ESP32 GCC/Clang directories using rustup's installed esp compiler path and
adds them to the current process environment. No manual activation or persistent
registry modification is needed to build. The target linker flags are scoped
to Xtensa so they do not affect host tests.

The combined check script runs formatting, host tests, core Clippy, the
firmware build, and firmware Clippy. The GitHub Actions workflow has a Linux
host job and a Windows ESP32 job. The latter installs the pinned Xtensa
compiler/linker, runs the combined checks, and links the release image.
Both jobs use Cargo.lock; neither job flashes a physical board.

Dependencies start from esp-generate 2.0.0's ESP32 + Wi-Fi + Embassy template:
esp-hal ~1.2.2, esp-radio =1.0.0-beta.1, esp-rtos 0.4.0, esp-alloc 0.11.0.
Keep Cargo.lock in version control. A HAL or radio upgrade needs an explicit
compatibility review and firmware build. The project uses esp-hal and esp-rtos
with Embassy; it does not use esp-idf-hal or FreeRTOS.

## Planned delivery order

1. Foundation: tooling, compileable boot, ownership and capacity contracts.
2. Interactive UART CLI (implemented): help/status, bounded input, robust line handling.
3. Scan adapter (implemented): real driver scans, full snapshots, escaped reports.
4. Cached reports/channels (implemented): age, coverage, and incompleteness warnings.
5. Watch/history (implemented): safe stop ordering and deterministic history eviction.
6. Hardware soak tests, memory measurements, and v0.1 release notes.

No hardware flashing is part of tooling setup. Check the chip revision and
serial port before running the explicit flash script.

## Interactive CLI on the board

Close the old serial monitor, then build/flash the new image:

```powershell
.\scripts\flash.ps1 -Port COM3
```

Once the `esp-wifi>` prompt appears, try commands in the monitor:

```text
help
status
scan
networks
channels
scan --channel 6
scan --passive
watch --interval 10s --count 3
history 02:11:22:33:44:55
stop
```

Replace the example BSSID with one returned by your scan. Full scans sweep
configured channels 1..11; channel 12..14 requests are rejected. Passive dwell
is 300 ms per channel; active dwell is 100..300 ms. The first watch scan starts
immediately. Count includes failed attempts; missed deadlines are skipped.
Only one scan runs at a time. Status and cached queries remain usable during
a scan. A filtered scan replaces the cached snapshot, so `channels` asks for
a new full scan after `scan --channel 6`.

The device performs its own echo; disable local echo in other serial clients.
If using another terminal, select 115200 baud, 8 data bits, no parity, 1 stop
bit, and no hardware flow control. CR, LF, and CRLF are supported.

The espflash monitor reserves Ctrl+C to exit and Ctrl+R to reset the board.
Use `stop` or Ctrl+X to stop future watch scans while the current scan finishes.
Forwarded Ctrl+C also requests stop; it exits espflash instead. Ctrl+U only
clears the line. Arrow-key editing and Unicode command arguments are not
supported. RSSI `history` is supported. See docs/validation.md for the
manual CLI acceptance checks. Automated tests do not replace these board checks.
