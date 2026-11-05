# Validation plan

## Automated gates

- Format the workspace and lint the host core and firmware with warnings denied.
- Host contract tests: invalid/duplicate arguments, bounded history eviction,
  and refusal to summarize a filtered scan as full channel coverage.
- CLI tests: CRLF deduplication, Backspace/Delete, overflow quarantine, recovery
  after invalid control/non-ASCII bytes and UART errors, Ctrl+C/Ctrl+U clear,
  blank/invalid commands, and status formatting at maximum counter values.
- Link an actual Xtensa firmware ELF, not only `cargo check`.

## Hardware evidence

The user confirmed the original foundation boots on COM3: ESP32 revision 3.1,
40 MHz crystal, and 4 MB flash. Flash verification completed successfully and
the radio initialized. This evidence predates the interactive CLI changes.

## CLI hardware acceptance (pending)

1. Close the old monitor and flash the CLI image on COM3.
2. At 115200 baud, verify a readable banner and a single `wifi>` prompt.
3. Enter `help`, then `status`; verify radio state, rising uptime, and sensible
   heap values. Scan counts should be zero and last scan duration `none`.
4. Edit `helx` using Backspace then `p`; Enter should execute help.
5. Enter `status extra`, an unknown command, and `scan`; each should produce a
   clear error and return to the prompt without resetting the board.
6. Paste over 128 command bytes, then Enter; ensure the whole line is rejected
   and a subsequent help command works. Use Ctrl+U to clear a partial line.
7. With a terminal configured to transmit CRLF, verify commands execute once.
8. Repeat help/status for several minutes; confirm no resets, prompt corruption,
   or progressive heap loss. If testing UART faults, verify discard/recovery and
   the UART receive-error counter using a controlled serial test setup.

## Remaining hardware acceptance for v0.1

- Confirm chip revision and board flash size with espflash board-info.
- Boot and verify UART0 output at 115200 without resets.
- Run one scan with no nearby APs and one with a known AP.
- Exercise capped scans, repeated scans, driver failures, and Ctrl+C.
- Ensure an overlong command and terminal escape bytes cannot corrupt the CLI.
- Verify history behavior under filtered, incomplete, and missed scans.
- Record heap low-water mark, stack/future allocations, scan durations, and
  uptime during a multi-hour repeated-scan run.

Build success does not establish radio behavior or board compatibility.
Interactive CLI and scan checks remain pending until performed on the actual board.
