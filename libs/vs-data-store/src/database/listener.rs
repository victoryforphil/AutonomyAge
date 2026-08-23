use crate::{buckets::BucketHandle, datapoints::Datapoint, topics::TopicKey};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

pub trait DataStoreListener: Send {
    fn on_datapoint(&mut self, datapoint: &Datapoint);
    fn on_raw_datapoint(&mut self, _datapoint: &Datapoint) {}
    fn on_bucket_update(&mut self, bucket: &BucketHandle);
    fn get_filter(&self) -> Option<TopicKey> {
        None
    }
}
impl Debug for dyn DataStoreListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataStoreListener")
    }
}

pub struct MockDataStoreListener {
    filter: TopicKey,
    pub updates: Vec<Datapoint>,
}

impl Default for MockDataStoreListener {
    fn default() -> Self {
        MockDataStoreListener {
            filter: TopicKey::empty(),
            updates: Vec::new(),
        }
    }
}

impl MockDataStoreListener {
    pub fn new(filter: TopicKey) -> Self {
        MockDataStoreListener {
            filter,
            updates: Vec::new(),
        }
    }

    pub fn as_handle(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }
}

impl Debug for MockDataStoreListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockDataStoreListener")
    }
}

impl DataStoreListener for MockDataStoreListener {
    fn on_datapoint(&mut self, datapoint: &Datapoint) {
        self.updates.push(datapoint.clone());
    }

    fn get_filter(&self) -> Option<TopicKey> {
        Some(self.filter.clone())
    }

    fn on_bucket_update(&mut self, _bucket: &BucketHandle) {}
}

#[cfg(test)]
mod tests {
    use vs_wtf::Timepoint;

    use crate::{database::Datastore, topics::TopicKeyProvider};

    use super::*;

    #[test]
    pub fn test_datastore_add_listener() {
        let mut datastore = Datastore::new();
        let topic_a = TopicKey::from_str("test/topic/a");
        let topic_b = TopicKey::from_str("test/topic/b");
        datastore.create_bucket(&topic_a);
        datastore.create_bucket(&topic_b);
        let filter = TopicKey::from_str("test/topic");
        let listener = MockDataStoreListener::new(filter.clone());
        let listener = listener.as_handle();

        datastore.add_listener(&filter, listener.clone()).unwrap();

        // Write value to bucket a and b
        datastore.add_datapoints(vec![
            Datapoint {
                topic: topic_a.handle(),
                time: Timepoint::now(),
                value: 42.into(),
            },
            Datapoint {
                topic: topic_b.handle(),
                time: Timepoint::now(),
                value: 42.into(),
            },
        ]);

        let updates = listener.lock().unwrap().updates.clone();
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].topic.key(), &topic_a);
        assert_eq!(updates[1].topic.key(), &topic_b);
    }
}
