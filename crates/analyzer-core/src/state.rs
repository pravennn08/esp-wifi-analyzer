//! State owned by the application task; never shared as mutable globals.
use crate::model::ScanSnapshot;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScannerState {
    Idle,
    Scanning,
    Faulted,
}

pub struct AnalyzerState {
    pub scanner: ScannerState,
    pub successful_scans: u32,
    pub failed_scans: u32,
    pub latest: Option<ScanSnapshot>,
}

impl Default for AnalyzerState {
    fn default() -> Self {
        Self {
            scanner: ScannerState::Idle,
            successful_scans: 0,
            failed_scans: 0,
            latest: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioState {
    Initialized,
    Failed,
}

/// Small, immutable telemetry message copied to the console task.
#[derive(Clone, Copy, Debug)]
pub struct StatusSnapshot {
    pub uptime_ms: u64,
    pub radio: RadioState,
    pub scanner: ScannerState,
    pub successful_scans: u32,
    pub failed_scans: u32,
    pub cached_aps: usize,
    pub last_scan_duration_ms: Option<u32>,
    pub heap_total_bytes: usize,
    pub heap_used_bytes: usize,
}
