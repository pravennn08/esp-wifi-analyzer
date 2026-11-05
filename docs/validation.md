# Validation plan

## Automated gates

- Format the workspace and lint the host core and firmware with warnings denied.
- Host contract tests: invalid/duplicate arguments, bounded history eviction,
  and refusal to summarize a filtered scan as full channel coverage.
- CLI tests: CRLF deduplication, Backspace/Delete, overflow quarantine, recovery
  after invalid control/non-ASCII bytes and UART errors, Ctrl+X/Ctrl+C stop and Ctrl+U clear,
  blank/invalid commands, and status formatting at maximum counter values.
- Command tests: scan flags, strict BSSID syntax, watch defaults/ranges/duplicate
  flags, and stop argument validation.
- History/watch tests: bounded per-AP retention, deterministic eviction, duplicate
  observations, missing-scan eligibility, finite watch completion, and skipped
  scheduling slots instead of catch-up bursts.
- Report tests: safe SSID escaping, unavailable-name markers, cache age/coverage,
  empty observations, incomplete-result warnings, filtered-scan channel refusal,
  signed RSSI statistics, and worst-case chunk-size budgets.
- Link an actual Xtensa firmware ELF, not only `cargo check`.
- `Host checks` runs host formatting/tests/Clippy on Linux and combined
  host/firmware checks plus release linking on Windows. Validate workflow YAML
  with actionlint when changing it. Local success is not a remote CI result.

## Hardware evidence

The user confirmed the original foundation boots on COM3: ESP32 revision 3.1,
40 MHz crystal, and 4 MB flash. Flash verification completed successfully and
the radio initialized. This evidence predates the interactive CLI changes.

## CLI hardware acceptance (pending)

1. Close the old monitor and flash the CLI image on COM3.
2. At 115200 baud, verify a readable banner and a single `wifi>` prompt.
3. Enter `help`, then `status`; verify radio state, rising uptime, and sensible
   heap values. Before the first scan, successful scan count should be zero and
   last scan duration `none`.
4. Edit `helx` using Backspace then `p`; Enter should execute help.
5. Enter `status extra`, an unknown command, and `history invalid`; each should
   produce a clear error and return to the prompt without resetting the board.
6. Paste over 128 command bytes, then Enter; ensure the whole line is rejected
   and a subsequent help command works. Use Ctrl+U to clear a partial line.
7. With a terminal configured to transmit CRLF, verify commands execute once.
8. Repeat help/status for several minutes; confirm no resets, prompt corruption,
   or progressive heap loss. If testing UART faults, verify discard/recovery and
   the UART receive-error counter using a controlled serial test setup.
9. Paste `watch` followed immediately by `stop`; verify the queued watch does
   not start later. A subsequent new watch command should still work.

## Scan, history, and watch acceptance (pending)

- Confirm chip revision and board flash size with espflash board-info.
- Boot and verify UART0 output at 115200 without resets.
- Run `scan` with a known AP; compare SSID, BSSID, channel, and RSSI with that
  AP's configuration. Verify strongest-RSSI ordering among returned observations.
- Run `scan --channel 6`, `scan --passive`, and both flags together. Reject
  `scan --channel 12` as outside the configured 1..11 set.
- Run `networks` without rescanning; verify its age increases. Verify empty cache
  behavior after reset, then an empty successful scan where practical.
- Run `channels` after a full scan; verify counts against that snapshot and
  the AP-presence limitation. After a filtered scan, require a new full scan.
- Run `watch --interval 10s --count 3`; verify exactly three attempts and no
  overlap. Exercise a scan longer than the interval to check skipped slots.
- Run unlimited watch and stop it using `stop` and Ctrl+X, including while a scan
  is active. Confirm that scan finishes and no further attempt starts. Use a
  terminal that forwards Ctrl+C to test its stop path; espflash consumes it.
- Inspect `history <BSSID>` after repeated scans. Verify sample ages, min/average/
  max, eligible misses, replacement after 32 tracked APs, and reboot clearing.
- Exercise capped scans and controlled driver failures; retain the last successful
  cache, do not count capped observations as proof of absence, and report errors.
- Ensure an overlong command and terminal escape bytes cannot corrupt the CLI.
- Verify history behavior under filtered, incomplete, and missed scans.
- Record heap low-water mark, stack/future allocations, scan durations, and
  uptime during a multi-hour repeated-scan run.

Build success does not establish radio behavior or board compatibility.
Interactive CLI and scan checks remain pending until performed on the actual board.
