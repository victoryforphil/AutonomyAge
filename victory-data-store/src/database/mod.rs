use crate::{
    buckets::{Bucket, BucketHandle},
    datapoints::Datapoint,
    primitives::{
        serde::{deserializer::PrimitiveDeserializer, serialize::to_map},
        Primitives,
    },
    topics::{TopicKey, TopicKeyHandle, TopicKeyProvider},
};
use listener::DataStoreListener;
use log::{debug, trace, warn};
use retention::RetentionPolicy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use thiserror::Error;
use tracing::{debug_span, info_span, instrument};
use victory_wtf::Timepoint;
use view::DataView;

pub type DatastoreHandle = Arc<Mutex<Datastore>>;

pub mod listener;
pub mod retention;
pub mod view;
#[derive(Debug, Clone)]
pub struct Datastore {
    buckets: HashMap<TopicKeyHandle, BucketHandle>,
    listeners: HashMap<TopicKeyHandle, Vec<Arc<Mutex<dyn DataStoreListener>>>>,
    pub retention: RetentionPolicy,
    /// Cache of topic searches and their resulting buckets
    query_cache: HashMap<TopicKeyHandle, Vec<BucketHandle>>,
}

#[derive(Error, Debug)]
pub enum DatastoreError {
    #[error("Generic Datastore Error: {0}")]
    Generic(String),
    #[error("Bucket not found for topic {0}")]
    BucketNotFound(TopicKey),
}

impl Default for Datastore {
    fn default() -> Self {
        Self::new()
    }
}

impl Datastore {
    #[instrument(skip_all)]
    pub fn new() -> Datastore {
        Datastore {
            listeners: HashMap::new(),
            buckets: HashMap::new(),
            retention: RetentionPolicy::default(),
            query_cache: HashMap::new(),
        }
    }

    #[instrument(skip_all)]
    pub fn handle(self) -> DatastoreHandle {
        Arc::new(Mutex::new(self))
    }

    #[instrument(skip_all)]
    pub fn set_retention(&mut self, retention: RetentionPolicy) {
        self.retention = retention;
    }

    #[instrument(skip_all)]
    pub fn clear_query_cache(&mut self) {
       self.query_cache.clear();
    }

    #[instrument(skip_all)]
    pub fn create_bucket<T: TopicKeyProvider>(&mut self, topic: &T) {
        if !self.buckets.contains_key(&topic.handle()) {
         //   let _span = debug_span!("creating_bucket").entered();
            let bucket = Bucket::new(topic);
            bucket
                .write()
                .unwrap()
                .set_retention(self.retention.clone());
            self.buckets.insert(topic.handle().clone(), bucket);
            self.clear_query_cache();
        }
    }

    #[instrument(skip_all)]
    pub fn get_or_create_bucket<T: TopicKeyProvider>(&mut self, topic: &T) -> BucketHandle {
        self.create_bucket(topic);
        self.get_bucket(topic).unwrap()
    }

    #[instrument(skip_all)]
    pub fn get_bucket<T: TopicKeyProvider>(
        &self,
        topic: &T,
    ) -> Result<BucketHandle, DatastoreError> {
        self.buckets
            .get(&topic.handle())
            .cloned()
            .ok_or_else(|| DatastoreError::BucketNotFound(topic.key().clone()))
    }

    #[instrument(skip_all)]
    pub fn get_buckets_matching_cached<T: TopicKeyProvider>(
        &mut self,
        parent_topic: &T,
    ) -> Result<Vec<BucketHandle>, DatastoreError> {
        // Check cache first
        if let Some(cached_buckets) = self.query_cache.get(&parent_topic.handle()) {
            return Ok(cached_buckets.clone());
        }

        // If not in cache, get buckets and cache result
        let buckets = self.get_buckets_matching(parent_topic)?;
        self.query_cache
            .insert(parent_topic.handle(), buckets.clone());
        Ok(buckets)
    }

    #[instrument(skip_all)]
    pub fn get_buckets_matching<T: TopicKeyProvider>(
        &self,
        parent_topic: &T,
    ) -> Result<Vec<BucketHandle>, DatastoreError> {
        Ok(self
            .buckets
            .iter()
            .filter_map(|(k, v)| {
                if k.key().is_child_of(parent_topic.key()) {
                    // trace!("Bucket {:?} matches topic {:?}", v, parent_topic.key());
                    Some(v.clone())
                } else if k.key() == parent_topic.key() {
                    // trace!("Bucket {:?} matches topic {:?}", v, parent_topic.key());
                    Some(v.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<BucketHandle>>())
    }

    #[instrument(skip_all)]
    pub fn get_struct<T, S>(&self, topic: &T) -> Result<S, DatastoreError>
    where
        T: TopicKeyProvider,
        S: DeserializeOwned,
    {
        // Get all the buckets that match the topic
        let buckets = self.get_buckets_matching(topic)?;

        let mut value_map: HashMap<TopicKeyHandle, Primitives> = HashMap::new();
        for bucket in buckets {
            let bucket = bucket.read().unwrap();

            if let Some(value) = bucket.get_latest_datapoint() {
                trace!(
                    "Added value to view: {:?} -> {:?}",
                    value.topic.key(),
                    value.value
                );
                let key = value
                    .topic
                    .key()
                    .remove_prefix(topic.key().clone())
                    .unwrap();

                value_map.insert(key.handle(), value.value.clone());
            }
        }
        // Deserialize the value map into the struct
        let mut deserializer = PrimitiveDeserializer::new(&value_map);
        let result = match Deserialize::deserialize(&mut deserializer) {
            Ok(s) => Ok(s),
            Err(e) => Err(DatastoreError::Generic(format!(
                "Error deserializing struct: {:?}",
                e
            ))),
        };

        match result {
            Ok(s) => Ok(s),
            Err(e) => Err(DatastoreError::Generic(format!(
                "Error deserializing struct: {:?}",
                e
            ))),
        }
    }

    #[instrument(skip_all)]
    pub fn add_struct<T: TopicKeyProvider, S: Serialize>(
        &mut self,
        topic: &T,
        time: Timepoint,
        value: S,
    ) -> Result<(), DatastoreError> {
        let topic = topic.handle();
        let value_map = to_map(&value)
            .map_err(|e| DatastoreError::Generic(format!("Error serializing struct: {:?}", e)))?;

        let mut datapoints = Vec::new();
        for (key, value) in value_map {
            let full_key = key.add_prefix(topic.key().to_owned());
            let datapoint = Datapoint::new(&full_key, time.clone(), value);
            // Remove the leading slash
            datapoints.push(datapoint);
        }
        self.add_datapoints(datapoints);
        Ok(())
    }

    #[instrument(skip_all)]
    pub fn add_primitive<T: TopicKeyProvider>(
        &mut self,
        topic: &T,
        time: Timepoint,
        value: Primitives,
    ) -> Result<usize, String> {
        let topic = topic.handle();
        let time = time.clone();
        self.create_bucket(&topic);

        let bucket = self.buckets.get_mut(&topic).unwrap();
        bucket.write().unwrap().add_primitive(time, value)
    }

    /// Add datapoints without notifying listeners, usually used when receiving remote datapoints
    /// that we want to store without triggering any local listeners
    #[instrument(skip_all)]
    pub fn add_datapoints_silent(&mut self, datapoints: Vec<Datapoint>) {
        let mut updated_datapoints = Vec::new();
        for datapoint in datapoints {
            let inserted_dp = datapoint.clone();
            if let Ok(n) =
                self.add_primitive(&inserted_dp.topic, inserted_dp.time, inserted_dp.value)
            {
                if n > 0 {
                    updated_datapoints.push(datapoint.clone());
                }
            }
        }
        if !updated_datapoints.is_empty() {
            self.notify_raw_datapoints(updated_datapoints);
        }
    }

    #[instrument(skip_all)]
    pub fn add_datapoints(&mut self, datapoints: Vec<Datapoint>) {
        let mut updated_datapoints = Vec::new();
        for datapoint in datapoints.clone() {
            let inserted_dp = datapoint.clone();
            if let Ok(n) =
                self.add_primitive(&inserted_dp.topic, inserted_dp.time, inserted_dp.value)
            {
                if n > 0 {
                    updated_datapoints.push(datapoint.clone());
                }
            }
        }
        if !updated_datapoints.is_empty() {
            self.notify_datapoints(updated_datapoints.clone());
            self.notify_raw_datapoints(updated_datapoints);
        }
    }

    #[instrument(skip_all)]
    pub fn get_latest_primitive<T: TopicKeyProvider>(&self, topic: &T) -> Option<Primitives> {
        let topic = topic.handle();
        self.buckets
            .get(&topic)
            .and_then(|b| b.read().unwrap().get_latest_value().cloned())
    }

    #[instrument(skip_all)]
    pub fn get_latest_datapoints<T: TopicKeyProvider>(
        &self,
        topic_query: &T,
    ) -> Result<Vec<Datapoint>, DatastoreError> {
        let buckets = self.get_buckets_matching(topic_query)?;
        let datapoints = buckets
            .iter()
            .filter_map(|b| b.read().unwrap().get_latest_datapoint().cloned())
            .collect();
        Ok(datapoints)
    }

    #[instrument(skip_all)]
    pub fn get_latest_primitives<T: TopicKeyProvider>(
        &self,
        topics: HashSet<T>,
    ) -> HashMap<TopicKey, Primitives> {
        topics
            .iter()
            .filter_map(|t| self.get_latest_primitive(t).map(|p| (t.key().clone(), p)))
            .collect()
    }

    #[instrument(skip_all)]
    pub fn get_datapoints<T: TopicKeyProvider>(
        &self,
        topic: &T,
    ) -> Result<Vec<Datapoint>, DatastoreError> {
        let topic = topic.handle();
        let buckets = self.get_buckets_matching(&topic)?;
        let mut datapoints = Vec::new();
        for bucket in buckets {
            let bucket = bucket.read().unwrap();
            datapoints.extend(bucket.get_datapoints());
        }

        Ok(datapoints)
    }

    #[instrument(skip_all)]
    pub fn get_all_keys(&self) -> Vec<TopicKeyHandle> {
        self.buckets.keys().cloned().collect()
    }

    #[instrument(skip_all)]
    pub fn get_all_display_names(&self) -> HashMap<TopicKeyHandle, String> {
        self.buckets
            .keys()
            .map(|k| (k.clone(), k.key().display_name()))
            .collect()
    }

    #[instrument(skip_all)]
    pub fn get_updated_keys<T: TopicKeyProvider>(
        &self,
        topic: &T,
        time: &Timepoint,
    ) -> Result<Vec<TopicKeyHandle>, DatastoreError> {
        let topic = topic.handle();
        let bucket = self.get_bucket(&topic)?;
        let bucket = bucket.read().unwrap();
        let new_values = bucket.get_data_points_after(&time.clone());
        Ok(new_values.iter().map(|v| v.topic.handle()).collect())
    }

    #[instrument(skip_all)]
    pub fn apply_view(&mut self, view: DataView) -> Result<(), DatastoreError> {
        for (topic, datapoint) in view.maps {
            self.add_datapoint(datapoint)?;
        }
        Ok(())
    }

    pub fn add_datapoint(&mut self, datapoint: Datapoint) -> Result<(), DatastoreError> {
        self.create_bucket(&datapoint.topic);
        let bucket = self.buckets.get_mut(&datapoint.topic).unwrap();
        bucket.write().unwrap().add_datapoint(datapoint.clone()).unwrap();
        self.notify_datapoints(vec![datapoint]);
        Ok(())
    }

    #[instrument(skip_all)]
    pub fn get_datapoints_after<T: TopicKeyProvider>(
        &self,
        topic: &T,
        time: &Timepoint,
    ) -> Result<Vec<Datapoint>, DatastoreError> {
        let topic = topic.handle();
        let buckets = self.get_buckets_matching(&topic)?;
        let mut datapoints = Vec::new();
        for bucket in buckets {
            let bucket = bucket.read().unwrap();
            datapoints.extend(bucket.get_data_points_after(time).into_iter().cloned());
        }
        Ok(datapoints)
    }

    #[instrument(skip_all)]
    pub fn get_primitives_after<T: TopicKeyProvider>(
        &self,
        topic: &T,
        time: &Timepoint,
    ) -> Result<Vec<Primitives>, DatastoreError> {
        let datapoints = self.get_datapoints_after(topic, time)?;
        Ok(datapoints.into_iter().map(|dp| dp.value).collect())
    }

    #[instrument(skip_all)]
    pub fn get_struct_after<T: TopicKeyProvider, S: DeserializeOwned>(
        &self,
        topic: &T,
        time: &Timepoint,
    ) -> Result<Option<S>, DatastoreError> {
        let buckets = self.get_buckets_matching(topic)?;
        let mut value_map = HashMap::new();

        for bucket in buckets {
            let bucket = bucket.read().unwrap();
            if let Some(datapoint) = bucket.get_data_points_after(time).first() {
                let key = datapoint
                    .topic
                    .key()
                    .remove_prefix(topic.key().clone())
                    .unwrap();
                value_map.insert(key.handle(), datapoint.value.clone());
            }
        }

        if value_map.is_empty() {
            return Ok(None);
        }

        let mut deserializer = PrimitiveDeserializer::new(&value_map);
        S::deserialize(&mut deserializer)
            .map(Some)
            .map_err(|e| DatastoreError::Generic(format!("Error deserializing struct: {:?}", e)))
    }
}

// ----------------------------
// Listener Implementations
// ----------------------------
impl Datastore {
    #[instrument(skip(self, listener))]
    pub fn add_listener(
        &mut self,
        topic_query: &TopicKey,
        listener: Arc<Mutex<dyn DataStoreListener>>,
    ) -> Result<(), DatastoreError> {
        debug!(
            "[DB/add_listener] Adding listener for topic: {:?}",
            topic_query
        );
        self.listeners
            .entry(topic_query.clone().handle())
            .or_default()
            .push(listener.clone());
        Ok(())
    }

    #[instrument(skip_all)]
    pub fn notify_datapoints(&mut self, datapoints: Vec<Datapoint>) {
        for (filter, listeners) in self.listeners.iter_mut() {
            for datapoint in datapoints.iter() {
                if datapoint.topic.key().matches(filter) {
                    for listener in listeners.iter_mut() {
                        listener.lock().unwrap().on_datapoint(datapoint);
                        listener.lock().unwrap().on_raw_datapoint(datapoint);
                    }
                }
            }
        }
    }

    #[instrument(skip_all)]
    pub fn notify_raw_datapoints(&mut self, datapoints: Vec<Datapoint>) {
        for (filter, listeners) in self.listeners.iter_mut() {
            for datapoint in datapoints.iter() {
                if datapoint.topic.key().matches(filter) {
                    for listener in listeners.iter_mut() {
                        listener.lock().unwrap().on_raw_datapoint(datapoint);
                    }
                }
            }
        }
    }

    #[instrument(skip_all)]
    pub fn notify_bucket_updates(&mut self, _buckets: Vec<BucketHandle>) {}
}

#[cfg(test)]
mod tests {

    use crate::database::*;

    #[test]
    pub fn test_datastore_creation() {
        let datastore = Datastore::new();
        assert_eq!(datastore.buckets.len(), 0);
    }

    #[test]
    pub fn test_datastore_create_bucket() {
        let mut datastore = Datastore::new();
        let topic = TopicKey::from_str("test/topic");
        datastore.create_bucket(&topic);
        assert_eq!(datastore.buckets.len(), 1);
        assert!(datastore.buckets.contains_key(&topic.handle()));
    }

    #[test]
    pub fn test_datastore_get_bucket() {
        let mut datastore = Datastore::new();
        let topic = TopicKey::from_str("test/topic");

        let bucket_failed = datastore.get_bucket(&topic);
        assert!(bucket_failed.is_err());

        datastore.create_bucket(&topic);

        let bucket = datastore.get_bucket(&topic);
        assert!(bucket.is_ok());
        assert_eq!(bucket.unwrap().read().unwrap().topic, topic.handle());
    }

    #[test]
    pub fn test_datastore_get_buckets_matching() {
        let mut datastore = Datastore::new();

        let topic_parent = TopicKey::from_str("test/topic");
        let topic_child_a = TopicKey::from_str("test/topic/b");
        let topic_child_b = TopicKey::from_str("test/topic/a");
        let topic_other = TopicKey::from_str("other/topic");
        let topic_other2 = TopicKey::from_str("test/other");
        let topic_child_a1 = TopicKey::from_str("test/topic/a/1");

        datastore.create_bucket(&topic_parent);
        datastore.create_bucket(&topic_child_a);
        datastore.create_bucket(&topic_child_b);
        datastore.create_bucket(&topic_other);
        datastore.create_bucket(&topic_other2);
        datastore.create_bucket(&topic_child_a1);

        let buckets = datastore.get_buckets_matching(&topic_parent).unwrap();
        assert_eq!(buckets.len(), 4);
        for bucket in &buckets {
            let bucket = bucket.read().unwrap();
            if bucket.topic.key() == &topic_parent {
                continue;
            }
            assert!(
                bucket.topic.is_child_of(&topic_parent),
                "Bucket {:?} is not a child of {:?}",
                bucket.topic,
                topic_parent
            );
        }

        let keys = buckets
            .iter()
            .map(|b| b.read().unwrap().topic.key().clone())
            .collect::<Vec<TopicKey>>();
        assert!(keys.contains(&topic_child_a));
        assert!(keys.contains(&topic_child_b));
        assert!(keys.contains(&topic_child_a1));
        assert!(!keys.contains(&topic_other));
        assert!(!keys.contains(&topic_other2));
        assert!(keys.contains(&topic_parent));
    }

    #[test]
    pub fn test_datastore_add_primitive() {
        let mut datastore = Datastore::new();
        let topic: TopicKey = "test/topic".into();
        let time = Timepoint::now();
        datastore.add_primitive(&topic, time.clone(), 42.into());

        let bucket = datastore.get_bucket(&topic).unwrap();
        let bucket = bucket.read().unwrap();
        let datapoints = bucket.get_datapoints();
        assert_eq!(datapoints.len(), 1);
        assert_eq!(datapoints[0].time, time.clone());
        assert_eq!(datapoints[0].value, 42.into());
    }

    #[test]
    pub fn test_datastore_get_set_struct() {
        sensible_env_logger::safe_init!();
        #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
        struct TestStruct {
            a: i32,
            b: String,
        }

        let mut datastore = Datastore::new();
        let topic: TopicKey = "/test/topic".into();
        let time = Timepoint::now();
        let test_struct = TestStruct {
            a: 42,
            b: "test".to_string(),
        };

        datastore
            .add_struct(&topic, time.clone(), test_struct.clone())
            .unwrap();

        // Log datastore keys

        let result: TestStruct = datastore.get_struct(&topic).unwrap();
        assert_eq!(result, test_struct);
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestStructA {
        a: i32,
        b: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestStructB {
        c: i32,
        d: String,
    }
    #[test]
    pub fn test_dataview_add_latest() {
        let topic: TopicKey = "/test/topic".into();

        let test_struct = TestStructA {
            a: 42,
            b: "test".to_string(),
        };
        let mut view = DataView::new_timed(Timepoint::zero());
        view.add_latest(&topic, test_struct.clone()).unwrap();
        let result: TestStructA = view.get_latest(&topic).unwrap();
        assert_eq!(result, test_struct);
    }
    #[test]
    pub fn test_datastore_view() {
        sensible_env_logger::safe_init!();

        let mut datastore = Datastore::new();
        let topic_a: TopicKey = "/test/a".into();
        let topic_b: TopicKey = "/test/b".into();
        let time = Timepoint::now();
        let test_struct_a = TestStructA {
            a: 42,
            b: "test".to_string(),
        };
        let test_struct_b = TestStructB {
            c: 42,
            d: "test".to_string(),
        };
        datastore
            .add_struct(&topic_a, time.clone(), test_struct_a.clone())
            .unwrap();
        datastore
            .add_struct(&topic_b, time.clone(), test_struct_b.clone())
            .unwrap();

        let view = DataView::new()
            .add_query(&mut datastore, &topic_a)
            .unwrap()
            .add_query(&mut datastore, &topic_b)
            .unwrap();

        let result: TestStructA = view.get_latest(&topic_a).unwrap();

        assert_eq!(result, test_struct_a);

        let result: TestStructB = view.get_latest(&topic_b).unwrap();
        assert_eq!(result, test_struct_b);

        let mut new_datastore = Datastore::new();
        new_datastore.apply_view(view).unwrap();

        let result: TestStructA = new_datastore.get_struct(&topic_a).unwrap();
        assert_eq!(result, test_struct_a);

        let result: TestStructB = new_datastore.get_struct(&topic_b).unwrap();
        assert_eq!(result, test_struct_b);
    }

    #[test]
    pub fn test_datastore_get_after() {
        let mut datastore = Datastore::new();
        let topic: TopicKey = "/test/topic".into();
        let time_before = Timepoint::new_secs(1.0);
        let data_time = Timepoint::new_secs(1.5);
        let time_after = Timepoint::new_secs(2.0);

        let test_struct = TestStructA {
            a: 42,
            b: "test".to_string(),
        };

        datastore
            .add_struct(&topic, data_time.clone(), test_struct.clone())
            .unwrap();

        // Test getting data after an earlier time (should return data)
        let result: Option<TestStructA> = datastore.get_struct_after(&topic, &time_before).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), test_struct);

        // Test getting data after a later time (should return None)
        let result: Option<TestStructA> = datastore.get_struct_after(&topic, &time_after).unwrap();
        assert!(result.is_none());
    }
}
