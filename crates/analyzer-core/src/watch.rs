// SPDX-License-Identifier: Apache-2.0

//! Monotonic scan scheduling, independent of executors and driver futures.
use crate::model::WatchOptions;

/// Sequence comparison for the bounded console queue, including u32 wraparound.
/// Requests cannot remain outstanding for half the sequence-number space.
pub fn request_precedes(request_id: u32, cutoff: u32) -> bool {
    let distance = cutoff.wrapping_sub(request_id);
    distance != 0 && distance < (1 << 31)
}

#[derive(Clone, Copy, Debug)]
pub struct WatchSchedule {
    options: WatchOptions,
    next_due_ms: u64,
    completed_attempts: u32,
}

impl WatchSchedule {
    pub fn new(options: WatchOptions, now_ms: u64) -> Self {
        Self {
            options,
            next_due_ms: now_ms,
            completed_attempts: 0,
        }
    }

    pub fn due(&self, now_ms: u64) -> bool {
        !self.finished() && now_ms >= self.next_due_ms
    }

    pub fn record_attempt(&mut self, started_at_ms: u64, completed_at_ms: u64) {
        if self.finished() {
            return;
        }
        self.completed_attempts = self.completed_attempts.saturating_add(1);
        if self.finished() {
            self.next_due_ms = u64::MAX;
            return;
        }
        let interval_ms = u64::from(self.options.interval_secs)
            .saturating_mul(1_000)
            .max(1);
        let deadline = started_at_ms.saturating_add(interval_ms);
        let completed_at_ms = completed_at_ms.max(started_at_ms);
        let skipped = completed_at_ms
            .saturating_sub(deadline)
            .div_ceil(interval_ms);
        self.next_due_ms = deadline.saturating_add(skipped.saturating_mul(interval_ms));
    }

    pub fn finished(&self) -> bool {
        self.options
            .count
            .is_some_and(|count| self.completed_attempts >= count)
    }

    pub fn next_due_ms(&self) -> u64 {
        self.next_due_ms
    }

    pub fn completed_attempts(&self) -> u32 {
        self.completed_attempts
    }
}
