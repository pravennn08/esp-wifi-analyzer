//! Deterministic strongest-first retention across the configured channel sweep.
use crate::model::{AccessPoint, ScanSnapshot};

/// Keep one observation per BSSID and retain the strongest bounded set.
pub fn retain(snapshot: &mut ScanSnapshot, candidate: AccessPoint) {
    let observations = &mut snapshot.access_points;
    if let Some(existing) = observations
        .iter_mut()
        .find(|ap| ap.bssid == candidate.bssid)
    {
        if candidate.rssi_dbm > existing.rssi_dbm {
            *existing = candidate;
        }
    } else if observations.is_full() {
        snapshot.possibly_incomplete = true;
        let weakest = observations.last_mut().expect("nonempty AP capacity");
        if order(&candidate, weakest).is_lt() {
            *weakest = candidate;
        }
    } else {
        let _ = observations.push(candidate);
    }
    observations.sort_unstable_by(order);
}

fn order(a: &AccessPoint, b: &AccessPoint) -> core::cmp::Ordering {
    b.rssi_dbm
        .cmp(&a.rssi_dbm)
        .then_with(|| a.bssid.0.cmp(&b.bssid.0))
}
