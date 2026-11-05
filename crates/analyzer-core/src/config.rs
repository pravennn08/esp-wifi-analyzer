// SPDX-License-Identifier: Apache-2.0

//! Application capacities; radio-driver allocations are budgeted separately.
pub const MAX_ACCESS_POINTS: usize = 32;
pub const HISTORY_SAMPLES: usize = 16;
pub const CLI_LINE_BYTES: usize = 128;
pub const REQUEST_QUEUE_DEPTH: usize = 4;
pub const RESPONSE_QUEUE_DEPTH: usize = 2;

pub const STATUS_REPORT_BYTES: usize = 1024;
pub const DEFAULT_WATCH_INTERVAL_SECS: u32 = 10;
pub const MAX_WATCH_INTERVAL_SECS: u32 = 3600;
pub const MAX_WATCH_COUNT: u32 = 1000;
