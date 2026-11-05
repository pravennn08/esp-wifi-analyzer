use analyzer_core::{
    analysis::history::{HistoryStore, RssiSample},
    cli::report::{
        HELP_TEXT, write_access_point, write_channels, write_history_header, write_history_sample,
        write_network_footer, write_network_header,
    },
    config::{MAX_ACCESS_POINTS, STATUS_REPORT_BYTES},
    model::{AccessPoint, Bssid, ScanCoverage, ScanSnapshot, Security},
};
use heapless::{String, Vec};

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
    write_access_point(&mut out, &ap).unwrap();
    assert!(out.contains("Lab\\x1b[2J\\x0a\\xff\\x7c\\\"\\\\"));
    assert!(out.contains("02:00:00:00:00:01 | 6 | -42 dBm | WPA2-Personal"));
    assert!(!out.as_bytes().contains(&0x1b));
    assert!(out.is_ascii());
}

#[test]
fn hidden_and_driver_truncated_names_are_distinct() {
    let mut ap = access_point(1, 1, -60, b"");
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    write_access_point(&mut out, &ap).unwrap();
    assert!(out.starts_with("<hidden>"));
    ap.ssid = Vec::from_slice(b"prefix").unwrap();
    ap.ssid_complete = false;
    out.clear();
    write_access_point(&mut out, &ap).unwrap();
    assert!(out.starts_with("<SSID unavailable: invalid UTF-8>"));
    assert!(!out.contains("prefix"));
}

#[test]
fn cached_scan_metadata_and_empty_results_are_explicit() {
    let scan = scan(ScanCoverage::AllConfiguredChannels, &[]);
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    write_network_header(&mut out, &scan, 1_500).unwrap();
    write_network_footer(&mut out, &scan).unwrap();
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
    write_channels(&mut out, &scan, 2_000).unwrap();
    assert!(out.contains("requires a full scan"));
    assert!(out.contains("channel 6; age: 1000 ms"));
    assert!(!out.contains("CHANNEL | OBSERVED APS"));
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
    write_channels(&mut out, &scan, 2_000).unwrap();
    assert!(out.contains("1 | 2 | -45 dBm"));
    assert!(out.contains("6 | 1 | -65 dBm"));
    assert!(out.contains("list may be incomplete"));
    assert!(out.contains("airtime and traffic utilization are not measured"));
    out.clear();
    write_network_footer(&mut out, &scan).unwrap();
    assert!(out.contains("list may be incomplete"));
}

#[test]
fn empty_channel_scan_does_not_claim_a_channel_is_best() {
    let scan = scan(ScanCoverage::AllConfiguredChannels, &[]);
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    write_channels(&mut out, &scan, 0).unwrap();
    assert!(out.contains("age: 0 ms"));
    assert!(out.contains("no channel ranking can be inferred"));
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
    )
    .unwrap();
    assert_eq!(out.as_str(), "0 ms | -50 dBm\r\n");
}

#[test]
fn worst_case_rows_and_channel_report_fit_stream_chunk_budget() {
    let ap = access_point(255, 14, i16::MIN, &[0xff; 32]);
    let mut out = String::<STATUS_REPORT_BYTES>::new();
    write_access_point(&mut out, &ap).unwrap();
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
    write_channels(&mut out, &scan, u64::MAX).unwrap();
    assert!(out.len() < STATUS_REPORT_BYTES);
    assert!(HELP_TEXT.len() < STATUS_REPORT_BYTES);
}
