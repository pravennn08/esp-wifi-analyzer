# CLI contract and implementation stages

The current firmware implements `help` and `status` over UART0. The core parser
also recognizes `scan` syntax, but executing it returns an explicit unavailable
message. The remaining commands below are the v0.1 target specification.

## Available now

```text
ESP32 Wi-Fi Analyzer 0.1.0
Target: ESP32 / UART0 at 115200 baud
Type 'help' for commands.
wifi> help
Commands:
  help     Show available commands
  status   Show device, radio, scanner, and heap status
```

`status` requests a snapshot from the application task. It reports firmware
version, target, uptime in seconds, radio initialization state, scanner state,
successful/failed scan counts, last scan duration, cached AP count, allocator
used/free bytes, command capacity, and UART receive errors. Before scan execution
is implemented, scan counters and cached AP count remain zero and the last scan
duration is `none`. Radio `initialized` does not mean connected to an AP.

Only one status request per console is outstanding at a time. A repeated status
while one is pending returns Busy. Help and line editing remain local to the
console. Replies redraw and preserve any command currently being edited.

## v0.1 command surface

| Command | Behavior |
|---|---|
| `help` | Implemented: describe available commands and editing |
| `scan` | Collect a new all-channel snapshot, strongest RSSI first |
| `scan --channel 6` | Scan one configured/allowed channel |
| `scan --passive` | Select passive scanning |
| `networks` | Show cached snapshot with age and coverage |
| `watch --interval 10s --count 3` | Repeat scans without overlap |
| `history <BSSID>` | Show retained RSSI observations |
| `channels` | Summarize AP presence from a full scan |
| `status` | Implemented: state, uptime, scan counters, heap, duration, capacities |

Region configuration is a firmware concern. Parser channel-range checks are
syntactic only; the adapter must enforce the actual allowed channel set.
`--ssid` and `watch --bssid` can follow after the initial command set.

## Serial behavior

- UART0 at 115200, 8N1 through the board's USB-to-UART bridge.
- Accept CR, CRLF, and LF without executing CRLF twice; support Backspace/Delete.
- Ctrl+U clears the current line. Forwarded Ctrl+C also clears it; espflash
  reserves Ctrl+C locally to exit its monitor.
- Reject an overlong line as a whole and discard until newline. Never execute
  a truncated command prefix.
- The current command grammar is ASCII. Non-ASCII and unsupported control/escape
  input reject the entire line and are never echoed. Tabs normalize to spaces.
- UART receive errors discard the current line until Enter or an explicit clear.
- Arrow-key editing and command history navigation are not implemented.
- Busy, invalid syntax, unsupported channel, and driver errors are distinct.
- No IP connection, router password, or TCP stack is required for serial scans.

## Example future scan output

```text
wifi> scan
Scanning...
SSID      BSSID              CH  RSSI      SECURITY
Lab       02:11:22:33:44:01    6  -42 dBm  WPA2-Personal
Returned: 1 AP; duration: 2140 ms; coverage: all configured channels
wifi>
```

At the scan capacity, print "result limit reached; list may be incomplete".
Sort only the returned observations; do not claim to have collected the 32
strongest APs in the environment unless the driver guarantees that behavior.
