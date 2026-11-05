//! Bounded status protocol. Only the application task reads analyzer state.
use analyzer_core::{
    config::{REQUEST_QUEUE_DEPTH, RESPONSE_QUEUE_DEPTH},
    state::StatusSnapshot,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

pub struct StatusRequest {
    pub id: u32,
}

pub struct StatusReply {
    pub request_id: u32,
    pub status: StatusSnapshot,
}

pub static REQUESTS: Channel<CriticalSectionRawMutex, StatusRequest, REQUEST_QUEUE_DEPTH> =
    Channel::new();
pub static REPLIES: Channel<CriticalSectionRawMutex, StatusReply, RESPONSE_QUEUE_DEPTH> =
    Channel::new();
