# Validation plan

## Automated gates

- Format the workspace and lint the host core with warnings denied.
- Host contract tests: invalid/duplicate arguments, bounded history eviction,
  and refusal to summarize a filtered scan as full channel coverage.
- Link an actual Xtensa firmware ELF, not only `cargo check`.

## Hardware acceptance for v0.1 (pending)

- Confirm chip revision and board flash size with espflash board-info.
- Boot and verify UART0 output at 115200 without resets.
- Run one scan with no nearby APs and one with a known AP.
- Exercise capped scans, repeated scans, driver failures, and Ctrl+C.
- Ensure an overlong command and terminal escape bytes cannot corrupt the CLI.
- Verify history behavior under filtered, incomplete, and missed scans.
- Record heap low-water mark, stack/future allocations, scan durations, and
  uptime during a multi-hour repeated-scan run.

Build success does not establish radio behavior or board compatibility.
Hardware checks remain pending until performed on the actual board.
