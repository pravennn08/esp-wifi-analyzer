//! Missing samples are not encoded as a fake RSSI value.
use crate::{
    config::{HISTORY_SAMPLES, MAX_ACCESS_POINTS},
    model::{AccessPoint, Bssid, ScanCoverage, ScanSnapshot},
};
use heapless::{Deque, Vec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RssiSample {
    pub at_ms: u64,
    pub rssi_dbm: i16,
}

#[derive(Clone, Debug, Default)]
pub struct RssiHistory {
    samples: Deque<RssiSample, HISTORY_SAMPLES>,
}

#[derive(Clone, Debug)]
pub struct NetworkHistory {
    pub access_point: AccessPoint,
    pub history: RssiHistory,
    pub first_seen_ms: u64,
    pub last_seen_ms: u64,
    /// Consecutive eligible scans where this AP was not observed.
    pub missed_scans: u32,
}

#[derive(Default)]
pub struct HistoryStore {
    entries: Vec<NetworkHistory, MAX_ACCESS_POINTS>,
    evictions: u32,
    last_observation: Option<(u32, u64)>,
}

impl HistoryStore {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn evictions(&self) -> u32 {
        self.evictions
    }

    pub fn get(&self, bssid: Bssid) -> Option<&NetworkHistory> {
        self.entries
            .iter()
            .find(|entry| entry.access_point.bssid == bssid)
    }

    pub fn observe(&mut self, scan: &ScanSnapshot) {
        let observation = (scan.id, scan.completed_at_ms);
        if self.last_observation == Some(observation) {
            return;
        }
        self.last_observation = Some(observation);

        for entry in &mut self.entries {
            let strongest = scan
                .access_points
                .iter()
                .filter(|ap| ap.bssid == entry.access_point.bssid)
                .max_by_key(|ap| ap.rssi_dbm);
            if let Some(ap) = strongest {
                entry.access_point = ap.clone();
                entry.last_seen_ms = scan.completed_at_ms;
                entry.missed_scans = 0;
                entry.history.push(RssiSample {
                    at_ms: scan.completed_at_ms,
                    rssi_dbm: ap.rssi_dbm,
                });
            } else if !scan.possibly_incomplete {
                let eligible = match scan.coverage {
                    ScanCoverage::AllConfiguredChannels => true,
                    ScanCoverage::SingleChannel(channel) => channel == entry.access_point.channel,
                };
                if eligible {
                    entry.missed_scans = entry.missed_scans.saturating_add(1);
                }
            }
        }

        for (index, candidate) in scan.access_points.iter().enumerate() {
            if self.get(candidate.bssid).is_some()
                || scan.access_points[..index]
                    .iter()
                    .any(|ap| ap.bssid == candidate.bssid)
            {
                continue;
            }
            // A duplicated driver observation yields one sample, using its strongest RSSI.
            let ap = scan
                .access_points
                .iter()
                .filter(|ap| ap.bssid == candidate.bssid)
                .max_by_key(|ap| ap.rssi_dbm)
                .unwrap_or(candidate);
            let mut history = RssiHistory::default();
            history.push(RssiSample {
                at_ms: scan.completed_at_ms,
                rssi_dbm: ap.rssi_dbm,
            });
            let entry = NetworkHistory {
                access_point: ap.clone(),
                history,
                first_seen_ms: scan.completed_at_ms,
                last_seen_ms: scan.completed_at_ms,
                missed_scans: 0,
            };
            if self.entries.is_full() {
                // Protect APs observed in this scan, even if timestamps tie.
                let oldest = self
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, entry)| {
                        !scan
                            .access_points
                            .iter()
                            .any(|ap| ap.bssid == entry.access_point.bssid)
                    })
                    .min_by_key(|(index, entry)| (entry.last_seen_ms, *index))
                    .map(|(index, _)| index);
                if let Some(oldest) = oldest {
                    self.entries[oldest] = entry;
                    self.evictions = self.evictions.saturating_add(1);
                }
            } else {
                let _ = self.entries.push(entry);
            }
        }
    }
}

impl RssiHistory {
    pub fn push(&mut self, sample: RssiSample) {
        if self.samples.is_full() {
            self.samples.pop_front();
        }
        let _ = self.samples.push_back(sample);
    }

    pub fn samples(&self) -> impl Iterator<Item = &RssiSample> {
        self.samples.iter()
    }
}
