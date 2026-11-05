use heapless::Vec;

use crate::config::MAX_ACCESS_POINTS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bssid(pub [u8; 6]);

/// SSIDs are arbitrary bytes, not necessarily valid UTF-8.
pub type Ssid = Vec<u8, 32>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Security {
    Open,
    Wep,
    WpaPersonal,
    Wpa2Personal,
    WpaWpa2Personal,
    Wpa3Personal,
    Wpa2Wpa3Personal,
    Enterprise,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessPoint {
    pub bssid: Bssid,
    pub ssid: Ssid,
    pub channel: u8,
    pub rssi_dbm: i16,
    pub security: Security,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScanOptions {
    pub channel: Option<u8>,
    pub passive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanCoverage {
    AllConfiguredChannels,
    SingleChannel(u8),
}

/// One successful scan, published as a unit. Times use monotonic milliseconds.
#[derive(Clone, Debug)]
pub struct ScanSnapshot {
    pub id: u32,
    pub completed_at_ms: u64,
    pub duration_ms: u32,
    pub coverage: ScanCoverage,
    /// True means the driver/result capacity may have hidden additional APs.
    pub possibly_incomplete: bool,
    pub access_points: Vec<AccessPoint, MAX_ACCESS_POINTS>,
}
