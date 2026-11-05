//! Missing samples are not encoded as a fake RSSI value.
use crate::config::HISTORY_SAMPLES;
use heapless::Deque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RssiSample {
    pub at_ms: u64,
    pub rssi_dbm: i16,
}

#[derive(Default)]
pub struct RssiHistory {
    samples: Deque<RssiSample, HISTORY_SAMPLES>,
}

impl RssiHistory {
    pub fn push(&mut self, sample: RssiSample) {
        if self.samples.is_full() {
            self.samples.pop_front();
        }
        let _ = self.samples.push_back(sample);
    }

    pub fn samples(&self) -> impl Iterator<Item = &RssiSample> {
        self.samples.iter()
    }
}
