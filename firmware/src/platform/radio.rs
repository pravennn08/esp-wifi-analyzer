// SPDX-License-Identifier: Apache-2.0

//! The radio worker owns the controller for the entire scan operation.
use analyzer_core::{
    analysis::scan::retain,
    config::MAX_ACCESS_POINTS,
    error::ScanError,
    model::{AccessPoint, Bssid, ScanCoverage, ScanOptions, ScanSnapshot, Security, Ssid},
};
use embassy_time::Instant;
use esp_hal::{peripherals::WIFI, time::Duration};
use esp_radio::wifi::{
    AuthenticationMethod, WifiController,
    ap::AccessPointInfo,
    scan::{ScanConfig, ScanTypeConfig},
};

pub struct Radio {
    controller: WifiController<'static>,
}

impl Radio {
    pub fn new(peripheral: WIFI<'static>) -> Result<Self, esp_radio::wifi::WifiError> {
        let controller = WifiController::new(peripheral, Default::default())?;
        Ok(Self { controller })
    }

    pub fn supports_channel(channel: u8) -> bool {
        (1..=11).contains(&channel)
    }

    pub async fn scan(&mut self, options: ScanOptions, id: u32) -> Result<ScanSnapshot, ScanError> {
        if options
            .channel
            .is_some_and(|channel| !Self::supports_channel(channel))
        {
            return Err(ScanError::UnsupportedChannel);
        }
        let started = Instant::now().as_millis();
        let mut snapshot = ScanSnapshot {
            id,
            completed_at_ms: 0,
            duration_ms: 0,
            coverage: options.channel.map_or(
                ScanCoverage::AllConfiguredChannels,
                ScanCoverage::SingleChannel,
            ),
            possibly_incomplete: false,
            access_points: heapless::Vec::new(),
        };
        let scan_type = if options.passive {
            ScanTypeConfig::Passive(Duration::from_millis(300))
        } else {
            ScanTypeConfig::Active {
                min: Duration::from_millis(100),
                max: Duration::from_millis(300),
            }
        };
        // The pinned driver's country API hardcodes its channel count. Sweep
        // our explicit 1..=11 set instead of inheriting a country-wide scan.
        let first = options.channel.unwrap_or(1);
        let last = options.channel.unwrap_or(11);
        for channel in first..=last {
            let config = ScanConfig::default()
                .with_channel(channel)
                .with_show_hidden(true)
                .with_max(MAX_ACCESS_POINTS + 1)
                .with_scan_type(scan_type);
            // Always await completion: only this worker accesses the controller.
            let records = self
                .controller
                .scan_async(&config)
                .await
                .map_err(|_| ScanError::Driver)?;
            if records.len() > MAX_ACCESS_POINTS {
                snapshot.possibly_incomplete = true;
            }
            for record in records {
                if record.channel != channel {
                    snapshot.possibly_incomplete = true;
                    continue;
                }
                retain(&mut snapshot, normalize(record)?);
            }
        }
        snapshot.completed_at_ms = Instant::now().as_millis();
        snapshot.duration_ms = snapshot
            .completed_at_ms
            .saturating_sub(started)
            .min(u32::MAX as u64) as u32;
        Ok(snapshot)
    }
}

fn normalize(record: AccessPointInfo) -> Result<AccessPoint, ScanError> {
    let text = record.ssid.as_str();
    // The pinned driver exposes only a valid UTF-8 prefix publicly. Preserve
    // that limitation explicitly instead of displaying a partial name as exact.
    let ssid_complete = text.len() == record.ssid.len();
    let ssid = Ssid::from_slice(text.as_bytes()).map_err(|_| ScanError::Driver)?;
    let security = match record.auth_method {
        Some(AuthenticationMethod::None) => Security::Open,
        Some(AuthenticationMethod::Wep) => Security::Wep,
        Some(AuthenticationMethod::Wpa) => Security::WpaPersonal,
        Some(AuthenticationMethod::Wpa2Personal) => Security::Wpa2Personal,
        Some(AuthenticationMethod::WpaWpa2Personal) => Security::WpaWpa2Personal,
        Some(
            AuthenticationMethod::Wpa3Personal
            | AuthenticationMethod::Wpa3ExtPsk
            | AuthenticationMethod::Wpa3ExtPskMixed,
        ) => Security::Wpa3Personal,
        Some(AuthenticationMethod::Wpa2Wpa3Personal) => Security::Wpa2Wpa3Personal,
        Some(
            AuthenticationMethod::Wpa2Enterprise
            | AuthenticationMethod::Wpa3Enterprise
            | AuthenticationMethod::Wpa2Wpa3Enterprise
            | AuthenticationMethod::WpaEnterprise
            | AuthenticationMethod::Wpa3EntSuiteB192Bit,
        ) => Security::Enterprise,
        _ => Security::Unknown,
    };
    Ok(AccessPoint {
        bssid: Bssid(record.bssid),
        ssid,
        ssid_complete,
        channel: record.channel,
        rssi_dbm: record.signal_strength as i16,
        security,
    })
}
