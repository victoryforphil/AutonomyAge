use serde::{Deserialize, Serialize};
use vs_data_store::topics::{TopicKeyHandle, TopicKeyProvider};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionMode {
    Latest,
    NewValues,
}

/// A subscription for a broker task
/// - topic_query: The topic key to subscribe to. Will be used to filter the messages that the task will receive.
/// - mode: The mode of the subscription. Either pull all latest values or only new values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerTaskSubscription {
    pub topic_query: TopicKeyHandle,
    pub mode: SubscriptionMode,
}

impl BrokerTaskSubscription {
    pub fn new_latest(topic_query: &dyn TopicKeyProvider) -> Self {
        Self {
            topic_query: topic_query.handle(),
            mode: SubscriptionMode::Latest,
        }
    }

    pub fn new_updates_only(topic_query: &dyn TopicKeyProvider) -> Self {
        Self {
            topic_query: topic_query.handle(),
            mode: SubscriptionMode::NewValues,
        }
    }
}
