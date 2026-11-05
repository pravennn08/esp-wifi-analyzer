//! Reports use caller-owned fixed-capacity buffers or another fmt::Write sink.
use core::fmt::{self, Write};

use crate::{
    config::{CLI_LINE_BYTES, MAX_ACCESS_POINTS},
    state::{RadioState, ScannerState, StatusSnapshot},
};

pub const HELP_TEXT: &str = concat!(
    "Commands:\r\n",
    "  help     Show available commands\r\n",
    "  status   Show device, radio, scanner, and heap status\r\n",
    "\r\n",
    "Enter submits a command. Backspace edits the current line.\r\n",
    "Ctrl+U clears a line. Ctrl+C clears it if forwarded by the terminal.\r\n",
    "Commands use ASCII. Scan execution is not implemented yet.\r\n",
);

pub fn write_status(out: &mut impl Write, version: &str, status: &StatusSnapshot) -> fmt::Result {
    let radio = match status.radio {
        RadioState::Initialized => "initialized",
        RadioState::Failed => "initialization failed",
    };
    let scanner = match status.scanner {
        ScannerState::Idle => "idle",
        ScannerState::Scanning => "scanning",
        ScannerState::Faulted => "faulted",
    };
    write!(
        out,
        "Firmware: {version}\r\nTarget: ESP32\r\nUptime: {} s\r\n",
        status.uptime_ms / 1_000
    )?;
    write!(out, "Radio: {radio}\r\nScanner: {scanner}\r\n")?;
    write!(
        out,
        "Successful scans: {}\r\nFailed scans: {}\r\n",
        status.successful_scans, status.failed_scans
    )?;
    match status.last_scan_duration_ms {
        Some(ms) => write!(out, "Last scan duration: {ms} ms\r\n")?,
        None => out.write_str("Last scan duration: none\r\n")?,
    }
    write!(
        out,
        "Cached APs: {} / {MAX_ACCESS_POINTS}\r\n",
        status.cached_aps
    )?;
    write!(
        out,
        "Heap used: {} bytes\r\nHeap free: {} bytes\r\n",
        status.heap_used_bytes,
        status
            .heap_total_bytes
            .saturating_sub(status.heap_used_bytes)
    )?;
    write!(
        out,
        "Command capacity: {CLI_LINE_BYTES} bytes\r\nScan execution: not implemented\r\n"
    )
}
