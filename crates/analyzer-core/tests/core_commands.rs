use analyzer_core::{
    analysis::history::HistoryStore,
    cli::{Command, InputEvent, LineEditor, parse},
    config::{HISTORY_SAMPLES, MAX_ACCESS_POINTS},
    model::{AccessPoint, Bssid, ScanCoverage, ScanOptions, ScanSnapshot, Security, WatchOptions},
    watch::WatchSchedule,
};

fn ap(id: u8, channel: u8, rssi_dbm: i16) -> AccessPoint {
    AccessPoint {
        bssid: Bssid([2, 0, 0, 0, 0, id]),
        ssid: heapless::Vec::new(),
        ssid_complete: true,
        channel,
        rssi_dbm,
        security: Security::Wpa2Personal,
    }
}

fn scan(id: u32, at: u64, coverage: ScanCoverage, aps: &[AccessPoint]) -> ScanSnapshot {
    ScanSnapshot {
        id,
        completed_at_ms: at,
        duration_ms: 100,
        coverage,
        possibly_incomplete: false,
        access_points: heapless::Vec::from_slice(aps).unwrap(),
    }
}

#[test]
fn command_grammar_supports_the_full_mvp_surface() {
    assert_eq!(parse("networks"), Ok(Command::Networks));
    assert_eq!(parse("channels"), Ok(Command::Channels));
    assert_eq!(parse("stop"), Ok(Command::Stop));
    assert_eq!(
        parse("history 02:AB:cd:00:01:ff"),
        Ok(Command::History(Bssid([2, 0xab, 0xcd, 0, 1, 0xff])))
    );
    assert_eq!(
        parse("scan --passive --channel 6"),
        Ok(Command::Scan(ScanOptions {
            channel: Some(6),
            passive: true
        }))
    );
    assert_eq!(
        parse("watch"),
        Ok(Command::Watch(WatchOptions {
            interval_secs: 10,
            count: None
        }))
    );
    assert_eq!(
        parse("watch --interval 10s --count 3"),
        Ok(Command::Watch(WatchOptions {
            interval_secs: 10,
            count: Some(3)
        }))
    );
    assert_eq!(
        parse("watch --count 1000 --interval 3600"),
        Ok(Command::Watch(WatchOptions {
            interval_secs: 3600,
            count: Some(1000)
        }))
    );
}

#[test]
fn commands_reject_ambiguous_flags_and_invalid_watch_bounds() {
    for input in [
        "networks extra",
        "channels 6",
        "stop now",
        "history",
        "history 02:00:00:00:00:01 extra",
        "watch --interval",
        "watch --count",
        "watch --count 0",
        "watch --count 1001",
        "watch --count -1",
        "watch --count +1",
        "watch --count 1s",
        "watch --count 4294967296",
        "watch --interval 0s",
        "watch --interval 3601s",
        "watch --interval -10s",
        "watch --interval +1",
        "watch --interval 1ms",
        "watch --interval s",
        "watch --count 2 --count 3",
        "watch --interval 1 --interval 2",
        "watch --passive",
        "scan --channel +6",
        "scan --passive --passive",
    ] {
        assert!(parse(input).is_err(), "accepted invalid input: {input}");
    }
}

#[test]
fn bssid_is_strictly_parsed_and_formatted_canonically() {
    let address: Bssid = "02:AB:cd:00:01:ff".parse().unwrap();
    assert_eq!(address.to_string(), "02:ab:cd:00:01:ff");
    for value in [
        "2:00:00:00:00:01",
        "02-00-00-00-00-01",
        "020000000001",
        "02:00:00:00:00:01:",
        "02:00:00:00:00:gg",
        "02:00:00:00:00:0",
        "02:00:00:00:00:001",
        " 02:00:00:00:00:01",
        "02:00:00:00:00:01 ",
    ] {
        assert!(
            value.parse::<Bssid>().is_err(),
            "accepted malformed BSSID: {value}"
        );
    }
}

#[test]
fn watch_uses_start_cadence_without_overlap_or_catchup_bursts() {
    let mut watch = WatchSchedule::new(
        WatchOptions {
            interval_secs: 10,
            count: None,
        },
        1_000,
    );
    assert!(!watch.due(999));
    assert!(watch.due(1_000));
    watch.record_attempt(1_000, 3_000);
    assert_eq!(watch.next_due_ms(), 11_000);
    assert!(!watch.due(10_999));
    assert!(watch.due(11_000));
    // A scan taking 25 seconds skips the 21s and 31s deadlines.
    watch.record_attempt(11_000, 36_000);
    assert_eq!(watch.next_due_ms(), 41_000);
    assert!(!watch.due(36_000));
    assert!(watch.due(41_000));
    // Finishing exactly at the next deadline may start the next nonoverlapping scan.
    watch.record_attempt(41_000, 51_000);
    assert_eq!(watch.next_due_ms(), 51_000);
    assert!(watch.due(51_000));
}

#[test]
fn watch_count_limits_attempts_including_failures() {
    let mut watch = WatchSchedule::new(
        WatchOptions {
            interval_secs: 1,
            count: Some(2),
        },
        0,
    );
    // The caller records every completed attempt whether the driver succeeds or fails.
    watch.record_attempt(0, 50);
    assert_eq!(watch.completed_attempts(), 1);
    assert!(!watch.finished());
    watch.record_attempt(1_000, 1_050);
    assert_eq!(watch.completed_attempts(), 2);
    assert!(watch.finished());
    assert!(!watch.due(u64::MAX));
    watch.record_attempt(2_000, 2_050);
    assert_eq!(watch.completed_attempts(), 2);
}

#[test]
fn history_preserves_first_seen_and_keeps_only_recent_real_samples() {
    let mut history = HistoryStore::default();
    for id in 0..=(HISTORY_SAMPLES as u32) {
        history.observe(&scan(
            id,
            u64::from(id) * 1_000,
            ScanCoverage::AllConfiguredChannels,
            &[ap(1, 6, -40 - id as i16)],
        ));
    }
    let entry = history.get(ap(1, 6, 0).bssid).unwrap();
    assert_eq!(entry.first_seen_ms, 0);
    assert_eq!(entry.last_seen_ms, HISTORY_SAMPLES as u64 * 1_000);
    assert_eq!(entry.history.samples().count(), HISTORY_SAMPLES);
    assert_eq!(entry.history.samples().next().unwrap().at_ms, 1_000);
    assert_eq!(entry.missed_scans, 0);
}

#[test]
fn history_missing_counts_require_eligible_complete_coverage() {
    let mut history = HistoryStore::default();
    let first = ap(1, 6, -50);
    history.observe(&scan(
        1,
        1_000,
        ScanCoverage::AllConfiguredChannels,
        core::slice::from_ref(&first),
    ));
    history.observe(&scan(2, 2_000, ScanCoverage::SingleChannel(1), &[]));
    assert_eq!(history.get(first.bssid).unwrap().missed_scans, 0);
    let mut capped = scan(3, 3_000, ScanCoverage::AllConfiguredChannels, &[]);
    capped.possibly_incomplete = true;
    history.observe(&capped);
    assert_eq!(history.get(first.bssid).unwrap().missed_scans, 0);
    history.observe(&scan(4, 4_000, ScanCoverage::SingleChannel(6), &[]));
    assert_eq!(history.get(first.bssid).unwrap().missed_scans, 1);
    history.observe(&scan(5, 5_000, ScanCoverage::AllConfiguredChannels, &[]));
    assert_eq!(history.get(first.bssid).unwrap().missed_scans, 2);
    history.observe(&scan(
        6,
        6_000,
        ScanCoverage::AllConfiguredChannels,
        core::slice::from_ref(&first),
    ));
    let entry = history.get(first.bssid).unwrap();
    assert_eq!(entry.missed_scans, 0);
    assert_eq!(entry.history.samples().count(), 2);
}

#[test]
fn history_deduplicates_rows_and_replayed_scan_observations() {
    let mut history = HistoryStore::default();
    let snapshot = scan(
        1,
        1_000,
        ScanCoverage::AllConfiguredChannels,
        &[ap(1, 6, -70), ap(1, 6, -40)],
    );
    history.observe(&snapshot);
    history.observe(&snapshot);
    assert_eq!(history.len(), 1);
    let entry = history.get(ap(1, 6, 0).bssid).unwrap();
    assert_eq!(entry.access_point.rssi_dbm, -40);
    assert_eq!(entry.history.samples().count(), 1);
}

#[test]
fn history_evicts_oldest_unobserved_ap_without_losing_current_observations() {
    let mut history = HistoryStore::default();
    let aps: Vec<_> = (0..MAX_ACCESS_POINTS as u8)
        .map(|id| ap(id, 1, -50))
        .collect();
    history.observe(&scan(1, 1_000, ScanCoverage::AllConfiguredChannels, &aps));
    assert_eq!(history.len(), MAX_ACCESS_POINTS);
    // Equal timestamps and the new AP appearing first must not evict AP 0's history.
    history.observe(&scan(
        2,
        1_000,
        ScanCoverage::AllConfiguredChannels,
        &[ap(99, 6, -30), ap(0, 1, -40)],
    ));
    assert_eq!(history.len(), MAX_ACCESS_POINTS);
    assert_eq!(history.evictions(), 1);
    assert!(history.get(ap(1, 1, 0).bssid).is_none());
    assert_eq!(
        history
            .get(ap(0, 1, 0).bssid)
            .unwrap()
            .history
            .samples()
            .count(),
        2
    );
    assert!(history.get(ap(99, 6, 0).bssid).is_some());
}

#[test]
fn forwarded_control_c_and_control_x_request_stop_but_control_u_only_clears() {
    for byte in [3, 24] {
        let mut editor = LineEditor::<4>::default();
        for character in b"overflow" {
            editor.push(*character);
        }
        assert_eq!(editor.push(byte), InputEvent::StopRequested);
        assert!(editor.line().is_empty());
        for character in b"help" {
            editor.push(*character);
        }
        assert_eq!(editor.push(b'\n'), InputEvent::Command(Command::Help));
    }
    let mut editor = LineEditor::<4>::default();
    assert_eq!(editor.push(21), InputEvent::Cancelled);
}

#[test]
fn urgent_stop_rejects_already_queued_watch_but_allows_new_watch() {
    use analyzer_core::watch::request_precedes;
    // watch request 7 was queued; stop arrives before it reaches the app.
    assert!(request_precedes(7, 8));
    assert!(!request_precedes(8, 8));
    assert!(!request_precedes(9, 8));
    // The same ordering must hold when request numbering wraps.
    assert!(request_precedes(u32::MAX, 0));
    assert!(request_precedes(u32::MAX - 1, 1));
    assert!(!request_precedes(0, u32::MAX));
}
