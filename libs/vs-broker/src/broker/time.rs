use log::info;
use serde::{Deserialize, Serialize};
use tracing::trace;
use vs_wtf::{Timepoint, Timespan};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerTime{
    pub time_monotonic: Timepoint,
    pub time_delta: Timespan,
    pub time_last_monotonic: Option<Timepoint>,
}

impl Default for BrokerTime {
    fn default() -> Self {
        Self {
            time_monotonic: Timepoint::zero(),
            time_delta: Timespan::zero(),
            time_last_monotonic: None,
        }
    }
}


impl BrokerTime {
    pub fn update(&mut self, delta_time: Timespan) {
        self.time_last_monotonic = Some(self.time_monotonic.clone());
        self.time_monotonic = self.time_monotonic.clone() + delta_time.clone();
        self.time_delta = delta_time.clone();
        trace!("BrokerTime // Updated to {} with delta {}", self.time_monotonic.secs(), delta_time.ms());
    }
}