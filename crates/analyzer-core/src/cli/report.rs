// SPDX-License-Identifier: Apache-2.0

//! Reports use caller-owned fixed-capacity buffers or another fmt::Write sink.
use core::fmt::{self, Write};

use super::terminal::{
    AnsiColor, MAX_TERMINAL_WIDTH, MIN_TERMINAL_WIDTH, TerminalLayout, TerminalSettings,
    write_colored,
};

use crate::{
    analysis::{
        channels::summarize,
        history::{NetworkHistory, RssiSample},
    },
    config::{CLI_LINE_BYTES, MAX_ACCESS_POINTS},
    model::{AccessPoint, Bssid, ScanCoverage, ScanSnapshot, Security},
    state::{RadioState, ScannerState, StatusSnapshot},
};

pub const HELP_TEXT: &str = concat!(
    "Commands:\r\n",
    "  help                              Show available commands\r\n",
    "  status                            Show device and scanner status\r\n",
    "  scan [--channel N] [--passive]     Collect a fresh network snapshot\r\n",
    "  networks                          Show cached results and their age\r\n",
    "  watch [--interval 10s] [--count 3] Repeat scans without overlap\r\n",
    "  stop                              Stop further watch scans\r\n",
    "  history <BSSID>                   Show retained RSSI observations\r\n",
    "  channels                          Summarize AP presence by channel\r\n",
    "  terminal                          Show display settings\r\n",
    "  terminal color <on|off>           Set ANSI colors\r\n",
    "  terminal width <40..160>          Set the terminal column count\r\n",
    "  terminal layout <auto|table|compact>  Choose the scan layout\r\n",
    "\r\n",
    "Enter submits a command. Backspace edits the current line.\r\n",
    "Ctrl+U clears a line. Ctrl+X stops watch; a running scan finishes.\r\n",
    "Commands use ASCII. Example BSSID: 02:11:22:33:44:55.\r\n",
    "Configured scan channels: 1..11.\r\n",
    "Watch defaults to every 10s until stopped; count includes failed attempts.\r\n",
    "Use stop in espflash; its Ctrl+C exits the monitor.\r\n",
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
        "Command capacity: {CLI_LINE_BYTES} bytes\r\nConfigured scan channels: 1..11\r\n"
    )
}

const CHANNEL_WIDTH: usize = 3;
const RSSI_NUMBER_WIDTH: usize = 6;
const RSSI_WIDTH: usize = RSSI_NUMBER_WIDTH + " dBm".len();
const SECURITY_WIDTH: usize = "WPA2/WPA3-Personal".len();
const BSSID_WIDTH: usize = 17;
const FIXED_TABLE_WIDTH: usize = BSSID_WIDTH + CHANNEL_WIDTH + RSSI_WIDTH + SECURITY_WIDTH + 8;
const SSID_HEADING: &str = "SSID (escaped)";

fn table_ssid_width(settings: TerminalSettings) -> Option<usize> {
    let width = settings.width.clamp(MIN_TERMINAL_WIDTH, MAX_TERMINAL_WIDTH);
    if settings.layout == TerminalLayout::Compact || width < FIXED_TABLE_WIDTH + SSID_HEADING.len()
    {
        None
    } else {
        Some(width - FIXED_TABLE_WIDTH)
    }
}

pub fn write_network_header(
    out: &mut impl Write,
    scan: &ScanSnapshot,
    now_ms: u64,
    settings: TerminalSettings,
) -> fmt::Result {
    let age = now_ms.saturating_sub(scan.completed_at_ms);
    if settings.width >= 80 {
        write!(
            out,
            "Scan #{}; age: {age} ms; duration: {} ms\r\n",
            scan.id, scan.duration_ms
        )?;
    } else {
        write!(
            out,
            "Scan #{}\r\nAge: {age} ms\r\nDuration: {} ms\r\n",
            scan.id, scan.duration_ms
        )?;
    }
    out.write_str("Coverage: ")?;
    write_coverage(out, scan.coverage)?;
    out.write_str("\r\nConfigured scan channels: 1..11\r\n")?;
    if !scan.access_points.is_empty()
        && let Some(ssid_width) = table_ssid_width(settings)
    {
        write_colored(
            out,
            settings.color,
            AnsiColor::Cyan,
            format_args!(
                "{SSID_HEADING:<ssid_width$}  {:<BSSID_WIDTH$}  {:>CHANNEL_WIDTH$}  {:>RSSI_WIDTH$}  SECURITY",
                "BSSID", "CH", "RSSI"
            ),
        )?;
        out.write_str("\r\n")?;
    }
    Ok(())
}

pub fn write_access_point(
    out: &mut impl Write,
    ap: &AccessPoint,
    settings: TerminalSettings,
) -> fmt::Result {
    // A 32-byte SSID takes at most 128 ASCII escape bytes plus two quotes.
    let mut ssid = heapless::String::<130>::new();
    write_ap_ssid(&mut ssid, ap)?;
    if let Some(ssid_width) = table_ssid_width(settings) {
        let (first, mut remaining) = split_ssid(&ssid, ssid_width);
        write!(
            out,
            "{first:<ssid_width$}  {}  {:>CHANNEL_WIDTH$}  ",
            ap.bssid, ap.channel
        )?;
        write_rssi(out, settings.color, ap.rssi_dbm, RSSI_NUMBER_WIDTH)?;
        write!(out, "  {}\r\n", security_name(ap.security))?;
        while !remaining.is_empty() {
            let (line, rest) = split_ssid(remaining, ssid_width);
            write!(out, "{line}\r\n")?;
            remaining = rest;
        }
    } else {
        let width = settings.width.clamp(MIN_TERMINAL_WIDTH, MAX_TERMINAL_WIDTH);
        write_colored(out, settings.color, AnsiColor::Cyan, "SSID (escaped):")?;
        out.write_str("\r\n")?;
        let mut remaining = ssid.as_str();
        while !remaining.is_empty() {
            let (line, rest) = split_ssid(remaining, width);
            write!(out, "{line}\r\n")?;
            remaining = rest;
        }
        write!(out, "BSSID: {}\r\nCH: {}\r\nRSSI: ", ap.bssid, ap.channel)?;
        write_rssi(out, settings.color, ap.rssi_dbm, 0)?;
        write!(out, "\r\nSECURITY: {}\r\n\r\n", security_name(ap.security))?;
    }
    Ok(())
}

/// Split rendered ASCII names only between complete escaped bytes.
fn split_ssid(text: &str, width: usize) -> (&str, &str) {
    let bytes = text.as_bytes();
    let mut end = 0;
    while end < bytes.len() {
        let length = if bytes[end] == b'\\' {
            if bytes.get(end + 1) == Some(&b'x') {
                4
            } else {
                2
            }
        } else {
            1
        };
        if end + length > width {
            break;
        }
        end += length;
    }
    text.split_at(end)
}

pub fn write_network_footer(
    out: &mut impl Write,
    scan: &ScanSnapshot,
    settings: TerminalSettings,
) -> fmt::Result {
    if scan.access_points.is_empty() {
        out.write_str("No access points observed in this scan.\r\n")?;
    }
    write!(
        out,
        "Returned: {} AP{}; strongest RSSI first.\r\n",
        scan.access_points.len(),
        if scan.access_points.len() == 1 {
            ""
        } else {
            "s"
        }
    )?;
    write_incomplete_warning(out, scan, settings.color)
}

pub fn write_channels(
    out: &mut impl Write,
    scan: &ScanSnapshot,
    now_ms: u64,
    settings: TerminalSettings,
) -> fmt::Result {
    let age = now_ms.saturating_sub(scan.completed_at_ms);
    let Ok(channels) = summarize(scan) else {
        write_colored(
            out,
            settings.color,
            AnsiColor::Red,
            "Error: channel summary requires a full scan.",
        )?;
        out.write_str("\r\nRun scan without --channel.\r\nCoverage: ")?;
        write_coverage(out, scan.coverage)?;
        return write!(out, "\r\nAge: {age} ms\r\n");
    };
    write!(
        out,
        "Channel summary from scan #{}\r\nAge: {age} ms\r\n",
        scan.id
    )?;
    write_colored(
        out,
        settings.color,
        AnsiColor::Cyan,
        "CHANNEL  OBSERVED APS  STRONGEST RSSI",
    )?;
    out.write_str("\r\n")?;
    for (index, channel) in channels.iter().enumerate() {
        if let Some(rssi) = channel.strongest_rssi_dbm {
            write!(out, "{:>7}  {:>12}  ", index + 1, channel.observed_aps)?;
            write_rssi(out, settings.color, rssi, 10)?;
            out.write_str("\r\n")?;
        }
    }
    if scan.access_points.is_empty() {
        out.write_str("No APs observed; no channel ranking can be inferred.\r\n")?;
    }
    write_incomplete_warning(out, scan, settings.color)?;
    out.write_str("AP presence indicates potential contention; airtime and traffic utilization are not measured.\r\n")
}

pub fn write_history_header(
    out: &mut impl Write,
    network: &NetworkHistory,
    now_ms: u64,
) -> fmt::Result {
    out.write_str("BSSID: ")?;
    write_bssid(out, network.access_point.bssid)?;
    out.write_str("\r\nSSID: ")?;
    write_ap_ssid(out, &network.access_point)?;
    write!(
        out,
        "\r\nLast channel: {}; security: {}\r\nFirst seen age: {} ms; last seen age: {} ms\r\nEligible missed scans: {}\r\n",
        network.access_point.channel,
        security_name(network.access_point.security),
        now_ms.saturating_sub(network.first_seen_ms),
        now_ms.saturating_sub(network.last_seen_ms),
        network.missed_scans
    )?;

    let mut count = 0u32;
    let mut sum = 0i32;
    let mut minimum = i16::MAX;
    let mut maximum = i16::MIN;
    for sample in network.history.samples() {
        count += 1;
        sum += i32::from(sample.rssi_dbm);
        minimum = minimum.min(sample.rssi_dbm);
        maximum = maximum.max(sample.rssi_dbm);
    }
    if count == 0 {
        out.write_str("Retained RSSI samples: 0\r\n")?;
    } else {
        write!(
            out,
            "Retained RSSI samples: {count}\r\nMinimum: {minimum} dBm; average: {} dBm; maximum: {maximum} dBm\r\n",
            sum / count as i32
        )?;
    }
    write!(out, "{:>13}  {:>RSSI_WIDTH$}\r\n", "SAMPLE AGE", "RSSI")
}

pub fn write_history_sample(
    out: &mut impl Write,
    sample: &RssiSample,
    now_ms: u64,
    settings: TerminalSettings,
) -> fmt::Result {
    write!(out, "{:>10} ms  ", now_ms.saturating_sub(sample.at_ms))?;
    write_rssi(out, settings.color, sample.rssi_dbm, RSSI_NUMBER_WIDTH)?;
    out.write_str("\r\n")
}

pub fn write_bssid(out: &mut impl Write, bssid: Bssid) -> fmt::Result {
    write!(out, "{bssid}")
}

fn write_ap_ssid(out: &mut impl Write, ap: &AccessPoint) -> fmt::Result {
    if !ap.ssid_complete {
        return out.write_str("<SSID unavailable: invalid UTF-8>");
    }
    if ap.ssid.is_empty() {
        return out.write_str("<hidden>");
    }
    out.write_char('"')?;
    for &byte in &ap.ssid {
        match byte {
            b'"' => out.write_str("\\\"")?,
            b'\\' => out.write_str("\\\\")?,
            b'|' => out.write_str("\\x7c")?,
            b' '..=b'~' => out.write_char(char::from(byte))?,
            _ => write!(out, "\\x{byte:02x}")?,
        }
    }
    out.write_char('"')
}

fn write_rssi(out: &mut impl Write, color: bool, rssi: i16, width: usize) -> fmt::Result {
    let style = match rssi {
        -60.. => AnsiColor::Green,
        -75..=-61 => AnsiColor::Yellow,
        _ => AnsiColor::Red,
    };
    write_colored(out, color, style, format_args!("{rssi:>width$} dBm"))
}

fn write_coverage(out: &mut impl Write, coverage: ScanCoverage) -> fmt::Result {
    match coverage {
        ScanCoverage::AllConfiguredChannels => out.write_str("all configured channels"),
        ScanCoverage::SingleChannel(channel) => write!(out, "channel {channel}"),
    }
}

fn write_incomplete_warning(out: &mut impl Write, scan: &ScanSnapshot, color: bool) -> fmt::Result {
    if scan.possibly_incomplete {
        write_colored(
            out,
            color,
            AnsiColor::Yellow,
            "Warning: result limit reached; list may be incomplete.",
        )?;
        out.write_str("\r\n")?;
    }
    Ok(())
}

fn security_name(security: Security) -> &'static str {
    match security {
        Security::Open => "Open",
        Security::Wep => "WEP",
        Security::WpaPersonal => "WPA-Personal",
        Security::Wpa2Personal => "WPA2-Personal",
        Security::WpaWpa2Personal => "WPA/WPA2-Personal",
        Security::Wpa3Personal => "WPA3-Personal",
        Security::Wpa2Wpa3Personal => "WPA2/WPA3-Personal",
        Security::Enterprise => "Enterprise",
        Security::Unknown => "Unknown",
    }
}
