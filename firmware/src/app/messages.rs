//! Planned bounded command protocol for the console and application tasks.
//! These types are defined now; queues are added with the interactive CLI.
#![allow(dead_code)]
use analyzer_core::{cli::Command, error::ScanError};

pub struct Request {
    pub id: u32,
    pub command: Command,
}

pub enum Reply {
    Accepted { request_id: u32 },
    Complete { request_id: u32, scan_id: u32 },
    Failed { request_id: u32, error: ScanError },
}
