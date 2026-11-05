//! Reports use caller-owned fixed-capacity buffers or another fmt::Write sink.
use core::fmt::{self, Write};

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

pub fn write_network_header(out: &mut impl Write, scan: &ScanSnapshot, now_ms: u64) -> fmt::Result {
    write!(
        out,
        "Scan #{}; age: {} ms; duration: {} ms\r\nCoverage: ",
        scan.id,
        now_ms.saturating_sub(scan.completed_at_ms),
        scan.duration_ms
    )?;
    write_coverage(out, scan.coverage)?;
    out.write_str(
        "\r\nConfigured scan channels: 1..11\r\nSSID (escaped) | BSSID | CH | RSSI | SECURITY\r\n",
    )
}

pub fn write_access_point(out: &mut impl Write, ap: &AccessPoint) -> fmt::Result {
    write_ap_ssid(out, ap)?;
    out.write_str(" | ")?;
    write_bssid(out, ap.bssid)?;
    write!(
        out,
        " | {} | {} dBm | {}\r\n",
        ap.channel,
        ap.rssi_dbm,
        security_name(ap.security)
    )
}

pub fn write_network_footer(out: &mut impl Write, scan: &ScanSnapshot) -> fmt::Result {
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
    write_incomplete_warning(out, scan)
}

pub fn write_channels(out: &mut impl Write, scan: &ScanSnapshot, now_ms: u64) -> fmt::Result {
    let Ok(channels) = summarize(scan) else {
        out.write_str("Error: channel summary requires a full scan. Run scan without --channel.\r\nLatest snapshot coverage: ")?;
        write_coverage(out, scan.coverage)?;
        return write!(
            out,
            "; age: {} ms.\r\n",
            now_ms.saturating_sub(scan.completed_at_ms)
        );
    };
    write!(
        out,
        "Channel summary from scan #{}; age: {} ms\r\n",
        scan.id,
        now_ms.saturating_sub(scan.completed_at_ms)
    )?;
    out.write_str("CHANNEL | OBSERVED APS | STRONGEST RSSI\r\n")?;
    for (index, channel) in channels.iter().enumerate() {
        if let Some(rssi) = channel.strongest_rssi_dbm {
            write!(
                out,
                "{} | {} | {} dBm\r\n",
                index + 1,
                channel.observed_aps,
                rssi
            )?;
        }
    }
    if scan.access_points.is_empty() {
        out.write_str("No APs observed; no channel ranking can be inferred.\r\n")?;
    }
    write_incomplete_warning(out, scan)?;
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
    out.write_str("SAMPLE AGE | RSSI\r\n")
}

pub fn write_history_sample(out: &mut impl Write, sample: &RssiSample, now_ms: u64) -> fmt::Result {
    write!(
        out,
        "{} ms | {} dBm\r\n",
        now_ms.saturating_sub(sample.at_ms),
        sample.rssi_dbm
    )
}

pub fn write_bssid(out: &mut impl Write, bssid: Bssid) -> fmt::Result {
    let [a, b, c, d, e, f] = bssid.0;
    write!(out, "{a:02x}:{b:02x}:{c:02x}:{d:02x}:{e:02x}:{f:02x}")
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

fn write_coverage(out: &mut impl Write, coverage: ScanCoverage) -> fmt::Result {
    match coverage {
        ScanCoverage::AllConfiguredChannels => out.write_str("all configured channels"),
        ScanCoverage::SingleChannel(channel) => write!(out, "channel {channel}"),
    }
}

fn write_incomplete_warning(out: &mut impl Write, scan: &ScanSnapshot) -> fmt::Result {
    if scan.possibly_incomplete {
        out.write_str("Warning: result limit reached; list may be incomplete.\r\n")?;
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
