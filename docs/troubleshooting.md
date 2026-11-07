# Troubleshooting

Run PowerShell commands from the repository root unless stated otherwise.
Type analyzer commands in the serial terminal at the board's `esp-wifi>` prompt.
For tool installation, see [development.md](development.md).

## Rust, Cargo, or espflash is not recognized

Open a new terminal after installation so it picks up your user PATH. Check:

```powershell
Get-Command rustc, cargo, rustup, espflash
rustc +stable --version
```

A standard Windows Rust installation places commands in
`%USERPROFILE%\.cargo\bin`. If you previously activated the retired project-local
tools, close that terminal; the project no longer needs an activation script.
Follow the development guide to install a missing tool rather than copying
executables or changing toolchain directories arbitrarily.

## The esp toolchain or Xtensa linker is missing

Standard Rust supports host tests but the classic ESP32 also needs the custom
Xtensa toolchain. Inspect the installed toolchains:

```powershell
rustup show
rustc +esp --version
```

Install the pinned versions in [development.md](development.md), then use
`scripts/build.ps1`. That script locates the installed linker through rustup;
calling Cargo directly may require configuring its toolchain paths yourself.
Keep the HAL/radio/runtime dependencies on the reviewed versions in Cargo.lock.

## The board does not appear or the COM port cannot be opened

Use a USB data cable and check Windows Device Manager under Ports (COM & LPT).
Depending on the board, its USB bridge may need a CP210x or CH340 driver from
the board or bridge manufacturer's official source.

```powershell
[System.IO.Ports.SerialPort]::GetPortNames()
espflash board-info --port COM3
```

Replace COM3 with the actual port. Close Arduino Serial Monitor, another
espflash monitor, or any program already using it. USB reconnection may change
the port assignment.

`board-info` reads the chip information; it does not flash application firmware.

## Connecting or flashing fails

Verify the port, cable, power, and that another serial client is not connected.
If the board does not enter download mode automatically, follow its documented
BOOT/EN procedure. On typical classic ESP32 development boards, holding BOOT
while resetting with EN enters download mode; release BOOT once the flashing
tool connects.

Confirm the board revision with `espflash board-info`. The current dependency
set targets classic ESP32 revision 3.0 or later. Other ESP32-family chips and
older revisions are not interchangeable with this build target.

## Boot logs appear but there is no new CLI prompt

Make sure the updated image was built and flashed, then reset the board. The
earlier foundation image printed a banner but had no interactive command input.

```powershell
.\scripts\flash.ps1 -Port COM3
```

Use 115200 baud, 8 data bits, no parity, 1 stop bit, and no hardware flow control.
Disable local echo because the device echoes commands. Press Enter once if the
terminal attached after the prompt was printed.

## Unknown commands, unsupported characters, or duplicated input

- Enter commands such as `help` at the analyzer's `esp-wifi>` prompt.
- Commands currently use ASCII. Arrow keys and bracketed-paste control
  sequences can produce an unsupported-character error. Type a short command
  manually or use a terminal configured to send plain text.
- CR, LF, and CRLF are supported. If characters appear twice, disable local echo.
- Backspace/Delete edits the line. Ctrl+U clears it.
- Commands longer than 128 bytes are discarded as a whole. Press Enter to finish
  discarding the line, or Ctrl+U to clear it, then enter a shorter command.

## Watch starts before stop, or the scanner reports Busy

The first watch scan starts immediately. Another scan or watch cannot start
while the scanner is occupied or watch is enabled.

```text
stop
status
```

Stopping prevents further repetitions; an already active scan may finish and
print its results. Wait for that result and confirm `Watch: off` and
`Scanner: idle` before starting another scan. Ctrl+X also requests stop.

In espflash, Ctrl+C exits the monitor and Ctrl+R resets the board. Exiting a
monitor alone does not necessarily stop firmware running on the board. A
different terminal must forward Ctrl+C for the device to interpret it as stop.

## Networks are missing or their RSSI changes

The radio samples channels in turn. Visibility varies with distance, antennas,
interference, scan type, and dwell time. An AP missed during one scan is not
proof that it disappeared. The configured channel set is 1..11; requests for
channels 12..14 are rejected, and other bands are not covered by this firmware.

Try `scan` and `scan --passive`, then compare their recorded coverage and age.
Empty SSIDs are shown as hidden. Names unavailable through the pinned driver's
UTF-8 interface are marked explicitly. At the result limit, the list may be
incomplete. RSSI is an observation, not a throughput or connection-quality test.

## Channels asks for a full scan or networks looks unchanged

`networks` reads the cache; it does not request another scan. Its age should grow.
Every successful scan replaces that cache, including a channel-filtered scan.

```text
scan
networks
channels
```

Run a full `scan` before `channels` if the latest snapshot covers only one
channel. AP counts describe observed presence, not measured airtime utilization.

## History is empty or older observations are gone

Use an exact BSSID returned by `scan`, for example
`history 02:11:22:33:44:55`. Multiple APs may share an SSID. History keeps up to
16 RSSI samples per AP and 32 tracked APs; older entries are evicted when needed.
It lives in RAM and resets on reboot. Filtered or incomplete scans do not
establish that an AP outside their eligible coverage was absent.

## The device reports UART errors, resets, or declining heap

Start with a known-good cable and stable USB power. Avoid pasting large command
streams, and watch the UART receive-error count. On a receive error the current
line is discarded; press Enter or clear it before entering another command.

For repeated failures, capture the commit, board revision, redacted boot/error
output, command sequence, uptime, and several `status` samples. A free-heap
reading is not a whole-system RAM or stack measurement. Use
[validation.md](validation.md) for soak and controlled-fault checks, then follow
[CONTRIBUTING.md](../CONTRIBUTING.md) for a reproducible issue report.

Suspected vulnerabilities should follow [SECURITY.md](../SECURITY.md) instead
of being disclosed in a public bug report.
