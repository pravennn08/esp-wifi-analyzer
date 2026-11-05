//! Bounded protocols between the console, state owner, and radio worker.
use analyzer_core::{
    cli::Command,
    config::{REQUEST_QUEUE_DEPTH, RESPONSE_QUEUE_DEPTH, STATUS_REPORT_BYTES},
    error::ScanError,
    model::{ScanOptions, ScanSnapshot},
    state::StatusSnapshot,
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};
use heapless::String;

pub struct Request {
    pub id: u32,
    pub command: Command,
}

pub enum ReplyKind {
    Text(String<STATUS_REPORT_BYTES>),
    Status(StatusSnapshot),
}

pub struct Reply {
    pub request_id: Option<u32>,
    pub kind: ReplyKind,
    pub complete: bool,
}

pub struct ScanJob {
    pub id: u32,
    pub options: ScanOptions,
}
pub struct ScanResult {
    pub result: Result<ScanSnapshot, ScanError>,
    pub started_at_ms: u64,
    pub completed_at_ms: u64,
}

pub static REQUESTS: Channel<CriticalSectionRawMutex, Request, REQUEST_QUEUE_DEPTH> =
    Channel::new();
pub static REPLIES: Channel<CriticalSectionRawMutex, Reply, RESPONSE_QUEUE_DEPTH> = Channel::new();
pub static STOP: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub static SCAN_JOBS: Channel<CriticalSectionRawMutex, ScanJob, 1> = Channel::new();
pub static SCAN_RESULTS: Channel<CriticalSectionRawMutex, ScanResult, 1> = Channel::new();
