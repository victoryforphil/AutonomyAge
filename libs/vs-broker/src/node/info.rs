use crate::node::NodeID;
use log::debug;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerNodeInfo {
    pub node_id: NodeID,
    pub name: String,
}

impl BrokerNodeInfo {
    pub fn new(name: &str) -> Self {
        let id = rand::random();
        debug!("BrokerNodeInfo // New Node ID: {}, name: {}", id, name);
        Self {
            node_id: id,
            name: name.to_string(),
        }
    }

    pub fn new_with_id(node_id: NodeID, name: &str) -> Self {
        Self {
            node_id,
            name: name.to_string(),
        }
    }
}
