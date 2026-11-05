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
