// SPDX-License-Identifier: Apache-2.0

use analyzer_core::{
    analysis::history::{HistoryStore, RssiSample},
    cli::{
        report::{
            HELP_TEXT, write_access_point, write_channels, write_history_header,
            write_history_sample, write_network_footer, write_network_header,
        },
        terminal::{MAX_TERMINAL_WIDTH, MIN_TERMINAL_WIDTH, TerminalLayout, TerminalSettings},
    },
    config::{MAX_ACCESS_POINTS, STATUS_REPORT_BYTES},
    model::{AccessPoint, Bssid, ScanCoverage, ScanSnapshot, Security},
};
use heapless::{String, Vec};

fn plain_settings() -> TerminalSettings {
    TerminalSettings {
        color: false,
        ..TerminalSettings::default()
    }
}

fn access_point(id: u8, channel: u8, rssi_dbm: i16, ssid: &[u8]) -> AccessPoint {
    AccessPoint {
        bssid: Bssid([2, 0, 0, 0, 0, id]),
        ssid: Vec::from_slice(ssid).unwrap(),
        ssid_complete: true,
        channel,
        rssi_dbm,
        security: Security::Wpa2Personal,
    }
}

fn scan(coverage: ScanCoverage, access_points: &[AccessPoint]) -> ScanSnapshot {
    ScanSnapshot {
        id: 7,
        completed_at_ms: 1_000,
        duration_ms: 200,
        coverage,
        possibly_incomplete: false,
        access_points: Vec::from_slice(access_points).unwrap(),
    }
}

#[test]
fn network_reports_escape_names_and_keep_terminal_controls_out() {
    let ap = access_point(1, 6, -42, b"Lab\x1b[2J\n\xff|\"\\");
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    let settings = TerminalSettings {
        width: MAX_TERMINAL_WIDTH,
        ..plain_settings()
    };
    write_access_point(&mut out, &ap, settings).unwrap();
    assert!(out.contains("Lab\\x1b[2J\\x0a\\xff\\x7c\\\"\\\\"));
    assert!(out.contains("02:00:00:00:00:01"));
    assert!(out.contains("-42 dBm  WPA2-Personal"));
    assert!(!out.contains('|'));
    assert!(!out.as_bytes().contains(&0x1b));
    assert!(out.is_ascii());
}

#[test]
fn hidden_and_driver_truncated_names_are_distinct() {
    let mut ap = access_point(1, 1, -60, b"");
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    let settings = TerminalSettings {
        width: MAX_TERMINAL_WIDTH,
        ..plain_settings()
    };
    write_access_point(&mut out, &ap, settings).unwrap();
    assert!(out.starts_with("<hidden>"));
    ap.ssid = Vec::from_slice(b"prefix").unwrap();
    ap.ssid_complete = false;
    out.clear();
    write_access_point(&mut out, &ap, settings).unwrap();
    assert!(out.starts_with("<SSID unavailable: invalid UTF-8>"));
    assert!(!out.contains("prefix"));
}

#[test]
fn cached_scan_metadata_and_empty_results_are_explicit() {
    let scan = scan(ScanCoverage::AllConfiguredChannels, &[]);
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    write_network_header(&mut out, &scan, 1_500, plain_settings()).unwrap();
    write_network_footer(&mut out, &scan, plain_settings()).unwrap();
    assert!(out.contains("Scan #7; age: 500 ms; duration: 200 ms"));
    assert!(out.contains("Coverage: all configured channels"));
    assert!(out.contains("No access points observed in this scan."));
    assert!(out.contains("Returned: 0 APs"));
}

#[test]
fn filtered_scan_never_becomes_a_full_channel_summary() {
    let scan = scan(
        ScanCoverage::SingleChannel(6),
        &[access_point(1, 6, -40, b"Lab")],
    );
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    write_channels(&mut out, &scan, 2_000, plain_settings()).unwrap();
    assert!(out.contains("requires a full scan"));
    assert!(out.contains("Coverage: channel 6\r\nAge: 1000 ms"));
    assert!(!out.contains("CHANNEL  OBSERVED APS"));
}

#[test]
fn channel_summary_counts_observations_and_explains_incomplete_results() {
    let mut scan = scan(
        ScanCoverage::AllConfiguredChannels,
        &[
            access_point(1, 1, -80, b"A"),
            access_point(2, 1, -45, b"B"),
            access_point(3, 6, -65, b"C"),
        ],
    );
    scan.possibly_incomplete = true;
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    write_channels(&mut out, &scan, 2_000, plain_settings()).unwrap();
    let rows: std::vec::Vec<_> = out.lines().skip(3).take(2).collect();
    assert!(rows[0].split_whitespace().eq(["1", "2", "-45", "dBm"]));
    assert!(rows[1].split_whitespace().eq(["6", "1", "-65", "dBm"]));
    assert!(out.contains("list may be incomplete"));
    assert!(out.contains("airtime and traffic utilization are not measured"));
    out.clear();
    write_network_footer(&mut out, &scan, plain_settings()).unwrap();
    assert!(out.contains("list may be incomplete"));
}

#[test]
fn empty_channel_scan_does_not_claim_a_channel_is_best() {
    let scan = scan(ScanCoverage::AllConfiguredChannels, &[]);
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    write_channels(&mut out, &scan, 0, plain_settings()).unwrap();
    assert!(out.contains("Age: 0 ms"));
    assert!(out.contains("No APs observed; no channel ranking can be inferred."));
}

#[test]
fn history_reports_signed_rssi_statistics_and_saturating_ages() {
    let mut store = HistoryStore::default();
    let ap = access_point(1, 6, -43, b"Lab");
    store.observe(&scan(
        ScanCoverage::AllConfiguredChannels,
        core::slice::from_ref(&ap),
    ));
    let mut second = scan(
        ScanCoverage::AllConfiguredChannels,
        &[AccessPoint {
            rssi_dbm: -50,
            ..ap.clone()
        }],
    );
    second.id = 8;
    second.completed_at_ms = 2_000;
    store.observe(&second);
    let network = store.get(ap.bssid).unwrap();
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    write_history_header(&mut out, network, 3_000).unwrap();
    assert!(out.contains("Retained RSSI samples: 2"));
    assert!(out.contains("Minimum: -50 dBm; average: -46 dBm; maximum: -43 dBm"));
    assert!(out.contains("last seen age: 1000 ms"));
    out.clear();
    write_history_sample(
        &mut out,
        &RssiSample {
            at_ms: 3_000,
            rssi_dbm: -50,
        },
        2_000,
        plain_settings(),
    )
    .unwrap();
    assert!(out.split_whitespace().eq(["0", "ms", "-50", "dBm"]));
}

#[test]
fn worst_case_rows_and_channel_report_fit_stream_chunk_budget() {
    let ap = access_point(255, 14, i16::MIN, &[0xff; 32]);
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    let settings = TerminalSettings {
        color: true,
        width: MAX_TERMINAL_WIDTH,
        ..plain_settings()
    };
    write_access_point(&mut out, &ap, settings).unwrap();
    assert!(out.len() < STATUS_REPORT_BYTES);
    let mut scan = scan(ScanCoverage::AllConfiguredChannels, &[]);
    scan.id = u32::MAX;
    scan.possibly_incomplete = true;
    for id in 0..MAX_ACCESS_POINTS {
        scan.access_points
            .push(access_point(id as u8, (id % 14 + 1) as u8, i16::MIN, b"X"))
            .unwrap();
    }
    out.clear();
    write_channels(&mut out, &scan, u64::MAX, settings).unwrap();
    assert!(out.len() < STATUS_REPORT_BYTES);
    assert!(
        HELP_TEXT
            .lines()
            .all(|line| line.len() + 2 < STATUS_REPORT_BYTES)
    );
}

fn without_color(text: &str) -> std::string::String {
    ["\x1b[0m", "\x1b[31m", "\x1b[32m", "\x1b[33m", "\x1b[36m"]
        .into_iter()
        .fold(text.to_owned(), |plain, code| plain.replace(code, ""))
}

#[test]
fn scan_table_aligns_headers_and_rows_with_or_without_color() {
    let ap = access_point(1, 11, -60, b"Lab");
    let scan = scan(
        ScanCoverage::AllConfiguredChannels,
        core::slice::from_ref(&ap),
    );
    let mut plain_header = String::<STATUS_REPORT_BYTES>::new();
    let mut plain_row = String::<STATUS_REPORT_BYTES>::new();
    let settings = plain_settings();
    write_network_header(&mut plain_header, &scan, 1_500, settings).unwrap();
    write_access_point(&mut plain_row, &ap, settings).unwrap();

    let heading = plain_header.lines().last().unwrap();
    let row = plain_row.lines().next().unwrap();
    assert_eq!(heading.find("SSID"), row.find("\"Lab\""));
    assert_eq!(heading.find("CH"), row.find("11"));
    assert_eq!(heading.find("RSSI"), row.find(" dBm"));
    assert_eq!(heading.find("SECURITY"), row.find("WPA2-Personal"));
    assert_eq!(heading.find("BSSID"), row.find("02:00:00:00:00:01"));
    assert!(!heading.contains('|'));
    assert!(!row.contains('|'));

    let mut colored_header = String::<STATUS_REPORT_BYTES>::new();
    let mut colored_row = String::<STATUS_REPORT_BYTES>::new();
    let settings = TerminalSettings {
        color: true,
        ..settings
    };
    write_network_header(&mut colored_header, &scan, 1_500, settings).unwrap();
    write_access_point(&mut colored_row, &ap, settings).unwrap();
    assert!(colored_header.contains("\x1b[36m"));
    assert!(colored_row.contains("\x1b[32m   -60 dBm\x1b[0m"));
    assert_eq!(without_color(&colored_header), plain_header.as_str());
    assert_eq!(without_color(&colored_row), plain_row.as_str());
}

#[test]
fn rssi_color_thresholds_and_reset_are_consistent() {
    let settings = TerminalSettings {
        color: true,
        ..plain_settings()
    };
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    for (rssi, color) in [(-60, 32), (-61, 33), (-75, 33), (-76, 31)] {
        out.clear();
        write_history_sample(
            &mut out,
            &RssiSample {
                at_ms: 0,
                rssi_dbm: rssi,
            },
            0,
            settings,
        )
        .unwrap();
        assert!(out.contains(&format!("\x1b[{color}m   {rssi} dBm\x1b[0m\r\n")));
    }
}

#[test]
fn all_terminal_widths_keep_rows_bounded_and_preserve_network_identity() {
    let ap = AccessPoint {
        security: Security::Wpa2Wpa3Personal,
        ..access_point(255, 255, i16::MIN, &[0xff; 32])
    };
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    for width in MIN_TERMINAL_WIDTH..=MAX_TERMINAL_WIDTH {
        for layout in [
            TerminalLayout::Auto,
            TerminalLayout::Table,
            TerminalLayout::Compact,
        ] {
            let settings = TerminalSettings {
                color: true,
                width,
                layout,
            };
            out.clear();
            write_access_point(&mut out, &ap, settings).unwrap();
            let plain = without_color(&out);
            assert!(
                plain.lines().all(|line| line.len() <= width),
                "report exceeds {width} columns in {layout:?}: {plain}"
            );
            assert!(plain.contains("02:00:00:00:00:ff"));
            assert!(plain.contains("WPA2/WPA3-Personal"));
            assert!(plain.contains("-32768 dBm"));
            assert!(!plain.contains('|'));
            let expected = format!("\"{}\"", "\\xff".repeat(32));
            let reconstructed = if plain.starts_with("SSID (escaped):") {
                plain
                    .lines()
                    .skip(1)
                    .take_while(|line| !line.starts_with("BSSID:"))
                    .collect::<std::string::String>()
            } else {
                let mut lines = plain.lines();
                let first = lines.next().unwrap();
                let end = first.find("02:00:00:00:00:ff").unwrap();
                let mut value = first[..end].trim_end().to_owned();
                for line in lines {
                    value.push_str(line);
                }
                value
            };
            assert_eq!(
                reconstructed, expected,
                "lost SSID data at width {width} in {layout:?}"
            );
        }
    }
}

#[test]
fn auto_layout_uses_compact_rows_in_narrow_terminals() {
    let ap = access_point(1, 6, -72, b"Lab");
    let scan = scan(
        ScanCoverage::AllConfiguredChannels,
        core::slice::from_ref(&ap),
    );
    let settings = TerminalSettings {
        width: MIN_TERMINAL_WIDTH,
        ..plain_settings()
    };
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    write_network_header(&mut out, &scan, 1_500, settings).unwrap();
    assert!(!out.contains(" # "));
    out.clear();
    write_access_point(&mut out, &ap, settings).unwrap();
    assert_eq!(
        out.as_str(),
        "SSID (escaped):\r\n\"Lab\"\r\nBSSID: 02:00:00:00:00:01\r\nCH: 6\r\nRSSI: -72 dBm\r\nSECURITY: WPA2-Personal\r\n\r\n"
    );
}
