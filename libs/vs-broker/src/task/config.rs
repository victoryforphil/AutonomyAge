use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::adapters::{AdapterID, ConnectionID};

use super::{subscription::BrokerTaskSubscription, trigger::BrokerTaskTrigger, BrokerTaskID};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BrokerCommanderFlags {
    NonBlocking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerTaskConfig {
    pub task_id: BrokerTaskID,
    pub name: String,
    pub adapter_id: AdapterID,
    pub connection_id: ConnectionID,
    pub subscriptions: Vec<BrokerTaskSubscription>,
    pub trigger: BrokerTaskTrigger,
    pub flags: HashSet<BrokerCommanderFlags>,
}
impl Default for BrokerTaskConfig {
    fn default() -> Self {
        Self {
            task_id: 0,
            name: "default".to_string(),
            adapter_id: 0,
            connection_id: 0,
            subscriptions: vec![],
            trigger: BrokerTaskTrigger::Always,
            flags: HashSet::new(),
        }
    }
}

impl BrokerTaskConfig {
    pub fn new_with_id(task_id: BrokerTaskID, name: &str) -> Self {
        Self {
            task_id,
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn new(name: &str) -> Self {
        let id = rand::random();
        Self::new_with_id(id, name)
    }
    //TODO: Would be cool to hav a macro to auto make these set / with methods
    pub fn with_trigger(mut self, trigger: BrokerTaskTrigger) -> Self {
        self.trigger = trigger;
        self
    }
    pub fn set_trigger(mut self, trigger: BrokerTaskTrigger) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn with_subscription(mut self, subscription: BrokerTaskSubscription) -> Self {
        self.subscriptions.push(subscription);
        self
    }

    pub fn with_flag(mut self, flag: BrokerCommanderFlags) -> Self {
        self.flags.insert(flag);
        self
    }

    pub fn set_flag(mut self, flag: BrokerCommanderFlags) -> Self {
        self.flags.insert(flag);
        self
    }

    pub fn add_subscription(&mut self, subscription: BrokerTaskSubscription) {
        self.subscriptions.push(subscription);
    }
}
