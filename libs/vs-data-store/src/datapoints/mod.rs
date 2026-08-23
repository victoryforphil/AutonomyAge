use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};
use vs_wtf::Timepoint;

use crate::{
    primitives::{serde::serialize::to_map, Primitives},
    topics::{TopicKeyHandle, TopicKeyProvider},
};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

pub struct Datapoint {
    pub topic: TopicKeyHandle,
    pub time: Timepoint,
    pub value: Primitives,
}

pub type DatapointMap = BTreeMap<TopicKeyHandle, Datapoint>;

pub type DatapointHandle = Arc<RwLock<Datapoint>>;

impl Datapoint {
    pub fn new<T: TopicKeyProvider>(topic: &T, time: Timepoint, value: Primitives) -> Datapoint {
        Datapoint {
            topic: topic.handle().clone(),
            time,
            value,
        }
    }

}

#[cfg(test)]
mod tests {
    use vs_wtf::Timepoint;

    use crate::{
        datapoints::Datapoint,
        primitives::Primitives,
        topics::{TopicKey, TopicKeyProvider},
    };
    #[test]
    fn test_datapoint() {
        let topic = TopicKey::from_str("test_topic");
        let now = Timepoint::now();
        let time = now;
        let value = Primitives::Integer(42);
        let datapoint = Datapoint::new(&topic, time.clone(), value);
        assert_eq!(datapoint.topic.key(), &TopicKey::from_str("test_topic"));
        assert_eq!(datapoint.value, 42.into());
    }
}
