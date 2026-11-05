// SPDX-License-Identifier: Apache-2.0

use crate::model::{ScanCoverage, ScanSnapshot};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChannelSummary {
    pub observed_aps: u8,
    pub strongest_rssi_dbm: Option<i16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummaryError {
    FullScanRequired,
}

/// AP presence only; this does not measure channel utilization or airtime.
pub fn summarize(snapshot: &ScanSnapshot) -> Result<[ChannelSummary; 14], SummaryError> {
    if snapshot.coverage != ScanCoverage::AllConfiguredChannels {
        return Err(SummaryError::FullScanRequired);
    }
    let mut channels = [ChannelSummary::default(); 14];
    for ap in &snapshot.access_points {
        if let Some(summary) = ap
            .channel
            .checked_sub(1)
            .and_then(|index| channels.get_mut(index as usize))
        {
            summary.observed_aps = summary.observed_aps.saturating_add(1);
            summary.strongest_rssi_dbm = Some(
                summary
                    .strongest_rssi_dbm
                    .map_or(ap.rssi_dbm, |old| old.max(ap.rssi_dbm)),
            );
        }
    }
    Ok(channels)
}
