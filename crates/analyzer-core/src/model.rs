// SPDX-License-Identifier: Apache-2.0

use heapless::Vec;

use crate::config::MAX_ACCESS_POINTS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bssid(pub [u8; 6]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BssidParseError;

impl core::str::FromStr for Bssid {
    type Err = BssidParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = value.as_bytes();
        if bytes.len() != 17 {
            return Err(BssidParseError);
        }
        let mut address = [0; 6];
        for (index, octet) in address.iter_mut().enumerate() {
            let offset = index * 3;
            if index != 5 && bytes[offset + 2] != b':' {
                return Err(BssidParseError);
            }
            let hex = |byte: u8| match byte {
                b'0'..=b'9' => Ok(byte - b'0'),
                b'a'..=b'f' => Ok(byte - b'a' + 10),
                b'A'..=b'F' => Ok(byte - b'A' + 10),
                _ => Err(BssidParseError),
            };
            *octet = (hex(bytes[offset])? << 4) | hex(bytes[offset + 1])?;
        }
        Ok(Self(address))
    }
}

impl core::fmt::Display for Bssid {
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let [a, b, c, d, e, f] = self.0;
        write!(out, "{a:02x}:{b:02x}:{c:02x}:{d:02x}:{e:02x}:{f:02x}")
    }
}

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
    /// False when the radio API cannot expose all original SSID bytes.
    pub ssid_complete: bool,
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
pub struct WatchOptions {
    pub interval_secs: u32,
    /// A limit on scan attempts, including failed attempts; None runs until stopped.
    pub count: Option<u32>,
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
