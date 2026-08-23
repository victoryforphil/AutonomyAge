use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use victory_data_store::topics::TopicKeyHandle;
use victory_wtf::Timepoint;

use super::BrokerTaskID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokerTaskStatus {
    Idle,
    Queued,
    Executing,
    Waiting,
    Completed,
    Failed,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerTaskState {
    pub task_id: BrokerTaskID,
    pub status: BrokerTaskStatus,
    pub last_execution_time: Option<Timepoint>,
    pub last_topic_update: HashMap<TopicKeyHandle, Timepoint>,
}

impl BrokerTaskState {
    pub fn new(task_id: BrokerTaskID) -> Self {
        Self {
            task_id,
            status: BrokerTaskStatus::Idle,
            last_execution_time: None,
            last_topic_update: HashMap::new(),
        }
    }

    pub fn set_status(&mut self, status: BrokerTaskStatus) {
        self.status = status;
    }

    pub fn set_last_execution_time(&mut self, time: Timepoint) {
        self.last_execution_time = Some(time);
    }

    pub fn topic_updated(&mut self, topic: &TopicKeyHandle) {
        self.last_topic_update
            .insert(topic.clone(), self.last_execution_time.clone().unwrap());
    }
}
