# CLI contract and implementation stages

This document is a target specification. The current firmware is boot-only;
the core parser currently accepts `help`, `status`, and `scan` options.

## v0.1 command surface

| Command | Behavior |
|---|---|
| `help` | Describe commands and limits |
| `scan` | Collect a new all-channel snapshot, strongest RSSI first |
| `scan --channel 6` | Scan one configured/allowed channel |
| `scan --passive` | Select passive scanning |
| `networks` | Show cached snapshot with age and coverage |
| `watch --interval 10s --count 3` | Repeat scans without overlap |
| `history <BSSID>` | Show retained RSSI observations |
| `channels` | Summarize AP presence from a full scan |
| `status` | State, uptime, scan counters, duration, capacities |

Region configuration is a firmware concern. Parser channel-range checks are
syntactic only; the adapter must enforce the actual allowed channel set.
`--ssid` and `watch --bssid` can follow after the initial command set.

## Serial behavior

- UART0 at 115200, 8N1 through the board's USB-to-UART bridge.
- Accept CRLF and LF without executing twice; support backspace and Ctrl+C.
- Reject an overlong line as a whole and discard until newline. Never execute
  a truncated command prefix.
- Escape network-supplied terminal control bytes. Accept commands as UTF-8;
  return an error for invalid input, never panic.
- Busy, invalid syntax, unsupported channel, and driver errors are distinct.
- No IP connection, router password, or TCP stack is required for serial scans.

## Example intended output

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
