use serde::{Deserialize, Serialize};
use vs_wtf::Timespan;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokerTaskTrigger {
    /// Always trigger the task on each commander tick.
    Always,
    /// Trigger the task on a fixed rate.
    Rate(Timespan),
    // Futrue ideas:
    // OnChange(TopicKeyHandle),
    // OnValue(TopicKeyHandle, Value),
}
