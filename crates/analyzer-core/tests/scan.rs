// SPDX-License-Identifier: Apache-2.0

use analyzer_core::{
    analysis::scan::retain,
    config::MAX_ACCESS_POINTS,
    model::{AccessPoint, Bssid, ScanCoverage, ScanSnapshot, Security},
};

fn snapshot() -> ScanSnapshot {
    ScanSnapshot {
        id: 1,
        completed_at_ms: 0,
        duration_ms: 0,
        coverage: ScanCoverage::AllConfiguredChannels,
        possibly_incomplete: false,
        access_points: heapless::Vec::new(),
    }
}

fn ap(id: u8, rssi_dbm: i16) -> AccessPoint {
    AccessPoint {
        bssid: Bssid([2, 0, 0, 0, 0, id]),
        ssid: heapless::Vec::new(),
        ssid_complete: true,
        channel: 1,
        rssi_dbm,
        security: Security::Open,
    }
}

#[test]
fn aggregate_retains_strongest_records_across_channels_and_marks_cap() {
    let mut scan = snapshot();
    for id in 0..MAX_ACCESS_POINTS as u8 {
        retain(&mut scan, ap(id, -90));
    }
    retain(&mut scan, ap(200, -30));
    retain(&mut scan, ap(201, -100));
    assert_eq!(scan.access_points.len(), MAX_ACCESS_POINTS);
    assert!(scan.possibly_incomplete);
    assert_eq!(scan.access_points[0].bssid, Bssid([2, 0, 0, 0, 0, 200]));
    assert!(!scan.access_points.iter().any(|ap| ap.bssid.0[5] == 201));
    assert!(!scan.access_points.iter().any(|ap| ap.bssid.0[5] == 31));
}

#[test]
fn duplicate_bssids_yield_one_sample_and_ties_have_stable_order() {
    let mut scan = snapshot();
    retain(&mut scan, ap(2, -60));
    retain(&mut scan, ap(1, -60));
    retain(&mut scan, ap(2, -40));
    retain(&mut scan, ap(2, -70));
    assert_eq!(scan.access_points.len(), 2);
    assert_eq!(scan.access_points[0].bssid.0[5], 2);
    assert_eq!(scan.access_points[0].rssi_dbm, -40);
    assert!(!scan.possibly_incomplete);
    retain(&mut scan, ap(1, -40));
    assert_eq!(scan.access_points[0].bssid.0[5], 1);
}
