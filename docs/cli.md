# CLI contract and implementation stages

The firmware implements the command surface below over UART0 at 115200 baud.
Commands collect or inspect real driver observations; the firmware does not
generate demonstration network records. Hardware acceptance checks remain
pending for this implementation.

## Available now

```text
ESP32 Wi-Fi Analyzer 0.1.0
Target: ESP32 / UART0 at 115200 baud
Type 'help' for commands.
wifi> help
Commands:
  help                              Show available commands
  status                            Show device and scanner status
  scan [--channel N] [--passive]     Collect a fresh network snapshot
  networks                          Show cached results and their age
  watch [--interval 10s] [--count 3] Repeat scans without overlap
  stop                              Stop further watch scans
  history <BSSID>                   Show retained RSSI observations
  channels                          Summarize AP presence by channel
```

`status` requests a snapshot from the application task. It reports firmware
version, target, uptime in seconds, radio initialization state, scanner state,
successful/failed scan counts, last scan duration, cached AP count, allocator
used/free bytes, configured channels, command capacity, and UART receive errors.
Before the first successful scan, cached AP count is zero and the last scan
duration is `none`. Radio `initialized` does not mean connected to an AP.

Application requests and report chunks pass through bounded queues. A busy
request is rejected explicitly. Reports stream bounded chunks rather than
placing a whole 32-network table in one buffer. The console redraws the prompt
and preserves a line being edited while asynchronous watch reports arrive.

## v0.1 command surface

| Command | Behavior |
|---|---|
| `help` | Describe available commands and editing |
| `scan` | Collect a new all-channel snapshot, strongest RSSI first |
| `scan --channel 6` | Scan one configured/allowed channel |
| `scan --passive` | Select passive scanning |
| `networks` | Show cached snapshot with age and coverage |
| `watch --interval 10s --count 3` | Repeat scans without overlap |
| `stop` | Stop future watch repetitions after any current scan finishes |
| `history <BSSID>` | Show retained RSSI observations |
| `channels` | Summarize AP presence from a full scan |
| `status` | State, uptime, scan counters, heap, duration, capacities |

The configured scan set is channels 1..11. An all-channel scan visits those
channels individually; it does not imply every channel supported by the chip.
The adapter rejects channels outside that configured set even if the parser
recognizes their numeric syntax. The firmware does not infer a location or
claim that the driver's default region was reconfigured.
`--ssid` and `watch --bssid` can follow after the initial command set.

## Cache, watch, and history semantics

- `scan` uses active scanning by default; `--passive` selects passive scanning.
  The two flags can be combined: `scan --channel 6 --passive`.
- `networks` shows the latest successful snapshot without performing a scan.
  Every snapshot identifies its age, duration, coverage, and completeness.
  A failed scan preserves the previous successful snapshot.
- `watch` defaults to a 10-second interval and runs until stopped. The interval
  is 1..3600 seconds, supplied as `10s` or `10`; count is 1..1000 attempts.
  Failures count as attempts. The first scan starts immediately. Scans never
  overlap, and missed schedule slots are skipped instead of queued.
- `stop`, Ctrl+X, or a forwarded Ctrl+C prevents further watch scans. Any
  in-progress driver scan finishes safely before stopping. Ctrl+U only clears
  the edited line. espflash consumes Ctrl+C to exit its monitor, so use `stop`
  or Ctrl+X there.
- `history` uses a strict six-byte colon-separated BSSID. It retains up to 16
  RSSI samples for each of 32 APs in RAM; reboot clears history. Minimum,
  average, maximum, sample ages, last channel/security, and eligible missed
  scans are reported. The signed average is rounded toward zero.
- History identifies APs by BSSID, not by SSID. When full, the least recently
  observed entry is replaced deterministically. Only complete scans covering
  an AP's channel count it missing; capped scans cannot establish absence.
- `channels` requires the latest snapshot to cover all configured channels.
  After a single-channel scan, run `scan` before requesting a channel summary.
  It reports observed AP counts and strongest RSSI, without claiming airtime
  utilization or recommending an objectively best channel.

## Serial behavior

- UART0 at 115200, 8N1 through the board's USB-to-UART bridge.
- Accept CR, CRLF, and LF without executing CRLF twice; support Backspace/Delete.
- Ctrl+U clears the current line. Ctrl+X and forwarded Ctrl+C request watch stop;
  espflash reserves Ctrl+C locally to exit its monitor.
- Reject an overlong line as a whole and discard until newline. Never execute
  a truncated command prefix.
- The current command grammar is ASCII. Non-ASCII and unsupported control/escape
  input reject the entire line and are never echoed. Tabs normalize to spaces.
- UART receive errors discard the current line until Enter or an explicit clear.
- Arrow-key editing and command history navigation are not implemented.
- Busy, invalid syntax, unsupported channel, and driver errors are distinct.
- No IP connection, router password, or TCP stack is required for serial scans.

## Example scan output

```text
wifi> scan
Scanning...
Scan #1; age: 0 ms; duration: 2140 ms
Coverage: all configured channels
Configured scan channels: 1..11
SSID (escaped) | BSSID | CH | RSSI | SECURITY
"Lab" | 02:11:22:33:44:01 | 6 | -42 dBm | WPA2-Personal
Returned: 1 AP; strongest RSSI first.
wifi>
```

At the scan capacity, print "result limit reached; list may be incomplete".
Sort only the returned observations; do not claim to have collected the 32
strongest APs in the environment unless the driver guarantees that behavior.
The displayed network above is illustrative, not an observed board result.
SSID control/non-ASCII bytes are escaped as ASCII. A hidden empty name appears
as `<hidden>`; a name the driver's public API cannot expose completely appears
as `<SSID unavailable: invalid UTF-8>` rather than displaying a misleading prefix.
