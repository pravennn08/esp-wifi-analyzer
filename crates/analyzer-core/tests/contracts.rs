use analyzer_core::{
    analysis::{
        channels::{SummaryError, summarize},
        history::{RssiHistory, RssiSample},
    },
    cli::{Command, parse},
    config::HISTORY_SAMPLES,
    model::{ScanCoverage, ScanOptions, ScanSnapshot},
};

#[test]
fn scan_options_are_validated_before_radio_access() {
    assert_eq!(
        parse("scan --channel 6 --passive"),
        Ok(Command::Scan(ScanOptions {
            channel: Some(6),
            passive: true,
        }))
    );
    for line in [
        "scan --channel",
        "scan --channel 0",
        "scan --channel 255",
        "scan --channel 6 --channel 1",
        "status extra",
        "scan --unknown",
    ] {
        assert!(parse(line).is_err(), "accepted invalid command: {line}");
    }
}

#[test]
fn history_retains_only_the_newest_bounded_samples() {
    let mut history = RssiHistory::default();
    for at_ms in 0..=(HISTORY_SAMPLES as u64) {
        history.push(RssiSample {
            at_ms,
            rssi_dbm: -50,
        });
    }
    assert_eq!(history.samples().count(), HISTORY_SAMPLES);
    assert_eq!(history.samples().next().unwrap().at_ms, 1);
}

#[test]
fn partial_scan_cannot_be_misreported_as_full_channel_coverage() {
    let scan = ScanSnapshot {
        id: 1,
        completed_at_ms: 1,
        duration_ms: 1,
        coverage: ScanCoverage::SingleChannel(6),
        possibly_incomplete: false,
        access_points: heapless::Vec::new(),
    };
    assert_eq!(summarize(&scan), Err(SummaryError::FullScanRequired));
}
