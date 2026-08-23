use crate::{
    buckets::BucketHandle,
    datapoints::Datapoint,
    primitives::{
        serde::{deserializer::PrimitiveDeserializer, serialize::to_map}, Primitives,
    },
    topics::{TopicKey, TopicKeyHandle, TopicKeyProvider},
};

use log::{debug, warn};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use victory_wtf::Timepoint;
use std::collections::HashMap;

use super::{Datastore, DatastoreError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataView {
    pub maps: HashMap<TopicKey, Datapoint>,
    bucket_cache: HashMap<TopicKeyHandle, Vec<BucketHandle>>,
    time: Timepoint,
}

impl Default for DataView {
    fn default() -> Self {
        Self::new()
    }
}

impl DataView {
    pub fn new() -> DataView {
        DataView {
            maps: HashMap::new(),
            bucket_cache: HashMap::new(),
            time: Timepoint::zero(),
        }
    }

    pub fn new_timed(time: Timepoint) -> DataView {
        DataView {
            maps: HashMap::new(),
            bucket_cache: HashMap::new(),
            time,
        }
    }

    pub fn add_query_from_view(
        mut self,
        view: &DataView,
        topic: &TopicKey,
    ) -> Result<DataView, DatastoreError> {
        for (key, datapoint) in view.maps.iter() {
            if key.key().is_child_of(topic) || key.key() == topic {
                self.maps.insert(key.clone(), datapoint.clone());
            }
        }
        Ok(self)
    }

    pub fn add_query_after_from_view(
        mut self,
        view: &DataView,
        topic: &TopicKey,
        time: &Timepoint,
    ) -> Result<DataView, DatastoreError> {
        for (key, datapoint) in view.maps.iter() {
            if (key.key().is_child_of(topic) || key.key() == topic) && datapoint.time >= *time {
                self.maps.insert(key.clone(), datapoint.clone());
            }
        }
        Ok(self)
    }

    pub fn add_query(
        mut self,
        datastore: &mut Datastore,
        topic: &TopicKey,
    ) -> Result<DataView, DatastoreError> {
        let buckets = datastore.get_buckets_matching_cached(topic)?;
        for bucket in buckets {
            let bucket = bucket.read().unwrap();

            if let Some( value) = bucket.get_latest_datapoint() {
                let key = value.topic.key().clone();
                
                self.maps.insert(key, value.clone());
            }
        }

        Ok(self)
    }

    pub fn add_query_after(
        mut self,
        datastore: &mut Datastore,
        topic: &TopicKey,
        time: &Timepoint,
    ) -> Result<DataView, DatastoreError> {
        let buckets = datastore.get_buckets_matching_cached(topic)?;
        for bucket in buckets {
            let bucket = bucket.read().unwrap();
            for datapoint in bucket.get_data_points_after(time) {
                let key = datapoint.topic.key().clone();
                self.maps.insert(key, datapoint.clone());
            }
        }
        Ok(self)
    }

    pub fn add_query_after_per(
        mut self,
        datastore: &mut Datastore,
        topic: &TopicKey,
        fallback_time: Timepoint,
        time_per_topic: &HashMap<TopicKeyHandle, Timepoint>,
    ) -> Result<DataView, DatastoreError> {
        let buckets = datastore.get_buckets_matching_cached(topic)?;
        for bucket in buckets {
            let bucket = bucket.read().unwrap();
            let time = time_per_topic.get(&bucket.topic.handle()).unwrap_or(&fallback_time);
            
            for datapoint in bucket.get_data_points_after(time) {
                let key = datapoint.topic.key().clone();
                self.maps.insert(key, datapoint.clone());
            }
        }
        Ok(self)
    }
    pub fn remove_query<T: TopicKeyProvider>(&mut self, topic: &T) {
        self.maps = self.maps
            .iter()
            .filter_map(|(k, v)| {
                if !k.key().is_child_of(topic.key()) && k.key() != topic.key() {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
    }
    pub fn get_latest_map<T: TopicKeyProvider>(
        &self,
        topic: &T,
    ) -> Result<HashMap<TopicKey, Datapoint>, DatastoreError> {
        let map = self
            .maps
            .iter()
            .filter_map(|(k, v)| {
                if k.key().is_child_of(topic.key()) || k.key() == topic.key() {
                    Some((k.key().clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<TopicKey, Datapoint>>();
        Ok(map)
    }

    pub fn get_latest<T: TopicKeyProvider, S: DeserializeOwned>(
        &self,
        topic: &T,
    ) -> Result<S, DatastoreError> {
        let value_map = self
            .maps
            .iter()
            .filter_map(|(k, v)| {
                if k.key().is_child_of(topic.key()) {
                    let key = k.key().remove_prefix(topic.key().clone()).unwrap();
                    Some((key.handle(), v.value.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<TopicKeyHandle, Primitives>>();

        // Deserialize the value map into the struct
        let mut deserializer = PrimitiveDeserializer::new(&value_map);
        let result = S::deserialize(&mut deserializer)
            .map_err(|e| DatastoreError::Generic(format!("Error deserializing struct: {:?}", e)));

        result
    }

    pub fn add_latest<T: TopicKeyProvider, S: Serialize>(
        &mut self,
        topic: &T,
        value: S,
    ) -> Result<(), DatastoreError> {
        let topic_key = topic.key().clone();
        let value_map = to_map(&value).unwrap();
        for (key, primitive_value) in value_map {
            let full_key = key.add_prefix(topic_key.clone());
            let datapoint = Datapoint {
                topic: full_key.handle(),
                time: self.time.clone(),
                value: primitive_value,
            };
            self.maps.insert(full_key, datapoint);
        }
        Ok(())
    }
    
    pub fn get_datapoint<T: TopicKeyProvider>(
        &self,
        topic: &T,
    ) -> Option<&Datapoint> {
        self.maps.get(topic.key())
    }

    pub fn get_value<T: TopicKeyProvider>(
        &self,
        topic: &T,
    ) -> Option<&Primitives> {
        self.maps.get(topic.key()).map(|dp| &dp.value)
    }

    pub fn get_time<T: TopicKeyProvider>(
        &self,
        topic: &T,
    ) -> Option<&Timepoint> {
        self.maps.get(topic.key()).map(|dp| &dp.time)
    }
    pub fn add_datapoint(&mut self, datapoint: Datapoint) -> Result<(), DatastoreError> {
        //TODO: Use topic handle instead of key
        self.maps.insert(datapoint.topic.key().clone(), datapoint);
        Ok(())
    }

    pub fn get_all_datapoints(&self) -> Vec<Datapoint> {
        self.maps.values().cloned().collect()
    }

    pub fn get_datapoints_after<T: TopicKeyProvider>(
        &self,
        topic: &T,
        time: &Timepoint,
    ) -> Vec<&Datapoint> {
        if let Some(datapoint) = self.maps.get(topic.key()) {
            if datapoint.time >= *time {
                return vec![datapoint];
            }
        }
        Vec::new()
    }

    pub fn get_primitives_after<T: TopicKeyProvider>(
        &self,
        topic: &T,
        time: &Timepoint,
    ) -> Vec<&Primitives> {
        if let Some(datapoint) = self.maps.get(topic.key()) {
            if datapoint.time >= *time {
                return vec![&datapoint.value];
            }
        }
        Vec::new()
    }

    pub fn get_struct_after<T: TopicKeyProvider, S: DeserializeOwned>(
        &self,
        topic: &T,
        time: &Timepoint,
    ) -> Result<Option<S>, DatastoreError> {
        let value_map = self
            .maps
            .iter()
            .filter_map(|(k, v)| {
                if k.key().is_child_of(topic.key()) && v.time >= *time {
                    let key = k.key().remove_prefix(topic.key().clone()).unwrap();
                    Some((key.handle(), v.value.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<TopicKeyHandle, Primitives>>();

        if value_map.is_empty() {
            return Ok(None);
        }

        let mut deserializer = PrimitiveDeserializer::new(&value_map);
        let result = S::deserialize(&mut deserializer)
            .map(Some)
            .map_err(|e| DatastoreError::Generic(format!("Error deserializing struct: {:?}", e)));

        result
    }

}

#[cfg(test)]
mod tests {

    use serde::{Deserialize, Serialize};
    use victory_wtf::Timepoint;

    use crate::{
        database::{view::DataView, Datastore},
        topics::TopicKey,
    };

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
        let mut view = DataView::new();
        view.time = Timepoint::zero();
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
        let time = Timepoint::zero();
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


}
