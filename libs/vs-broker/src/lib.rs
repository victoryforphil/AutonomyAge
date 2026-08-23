pub mod adapters;
pub mod big_state;
pub mod broker;
pub mod commander;
pub mod node;
pub mod task;

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use vs_data_store::{primitives::Primitives, topics::TopicKey};
    use vs_wtf::Timespan;

    use crate::{
        adapters::channel::ChannelBrokerAdapter,
        broker::Broker,
        commander::linear::LinearBrokerCommander,
        node::{info::BrokerNodeInfo, BrokerNode},
        task::example::task_ticker::TaskTicker,
    };
    use test_env_log::test;
    #[test]
    fn int_single_task_single_node() {
        let (adapter_a, adapter_b) = ChannelBrokerAdapter::new_pair();
        let mut broker = Broker::new(LinearBrokerCommander::new());
        broker.add_adapter(adapter_a);

        let node_info = BrokerNodeInfo::new("test_node");
        let mut node = BrokerNode::new(node_info, adapter_b);

        let topic_a = TopicKey::from_str("test/a");
        let task_a = TaskTicker::new(topic_a.clone());
        node.add_task(Arc::new(Mutex::new(task_a))).unwrap();

        let broker_handle = Arc::new(Mutex::new(broker));
        let broker = broker_handle.clone();
        // Launch broker thread
        let broker_thread = std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let runtime = tokio::runtime::Runtime::new().unwrap();

            while start.elapsed().as_secs() < 2 {
                {
                    let mut broker = broker.lock().unwrap();
                    runtime
                        .block_on(broker.tick(Timespan::new_ms(50.0)))
                        .unwrap();
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        let node_handle = Arc::new(Mutex::new(node));

        // Launch node thread
        let node_thread = std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let node = node_handle.clone();
            while start.elapsed().as_secs() < 5 {
                {
                    let mut node = node.lock().unwrap();
                    node.tick().unwrap();
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        // Wait for both threads to complete
        broker_thread.join().unwrap();
        node_thread.join().unwrap();

        // Test the datastore on broker to see if the topic was updated
        let broker = broker_handle.lock().unwrap();
        let data = broker.datastore.lock().unwrap();
        let value = data.get_latest_datapoints(&topic_a).unwrap();

        assert_eq!(value.len(), 1);
        // Check the key
        assert_eq!(value[0].topic.display_name(), topic_a.display_name());
        // Check the value
        let value = match value[0].value {
            Primitives::Integer(n) => n,
            _ => panic!("Expected a number"),
        };
        assert!(value > 1);
    }
}
