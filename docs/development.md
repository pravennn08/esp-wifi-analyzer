# Development environment

## Tools

Project-local installation under `.tools/`; activate with:

```powershell
. .\scripts\env.ps1
```

Installed tools: rustup / Rust stable 1.99.1 (host tests), Xtensa Rust
1.97.0.0 (esp toolchain), GCC esp-16.1.0_20260609, espup 0.19.0 (Xtensa
toolchain), esp-generate 2.0.0 (upstream foundation), and espflash 4.7.0
(board info, flashing, serial monitoring). Visual Studio C++ build tools are
required for Windows host linking and were found on this machine.

The root selects stable so tests run natively. Firmware builds explicitly use
`cargo +esp` and `xtensa-esp32-none-elf`, building core/alloc from source. Target
linker flags are target-scoped and do not break host tests.

Dependencies start from the esp-generate 2.0.0 ESP32 + Wi-Fi + Embassy template:
esp-hal ~1.2.2, esp-radio =1.0.0-beta.1, esp-rtos 0.4.0, esp-alloc 0.11.0.
Keep Cargo.lock in version control. A minor/major HAL or radio update requires
an explicit compatibility review and firmware build. The initial setup is not
a migration to esp-idf-hal and does not introduce FreeRTOS.

## Commands

```powershell
cargo +stable test -p analyzer-core --locked
cargo +stable fmt --all
cargo +stable clippy -p analyzer-core --all-targets --locked -- -D warnings
cargo +esp esp-build --locked
.\scripts\build.ps1 -Release
```

Use `scripts/check.ps1` for the combined gate. Host-only CI can run the first
three checks with ordinary stable Rust; embedded CI needs the Xtensa toolchain.

## Planned delivery order

1. Foundation: tooling, compileable boot, ownership and capacity contracts.
2. Interactive UART CLI: help/status, bounded input, robust line handling.
3. Scan adapter: driver conversion, full snapshots, escaped report formatting.
4. Cached reports and channel summaries with coverage/completeness handling.
5. Watch/history with cancellation policy and deterministic eviction.
6. Hardware soak tests, memory measurements, and v0.1 release notes.

No hardware flashing is part of installation. Check chip revision and serial
port before running the explicit flash script.

On this machine espup installed the compiler/linker but its final Windows
registry update was denied by the sandbox. `scripts/env.ps1` supplies the
required PATH and LIBCLANG_PATH per shell. Re-running setup verifies the
installed tools and does not require that registry write.
