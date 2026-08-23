use std::collections::HashMap;

use info::BrokerNodeInfo;
use log::{debug, info};
use victory_data_store::database::view::DataView;
use victory_wtf::Timepoint;

use crate::{
    adapters::{BrokerAdapterError, BrokerAdapterHandle}, broker::time::BrokerTime, task::{config::BrokerTaskConfig, subscription::SubscriptionMode, BrokerTaskHandle, BrokerTaskID}
};

pub mod info;

pub type NodeID = u32;

pub struct BrokerNode {
    pub info: BrokerNodeInfo,
    pub adapter: BrokerAdapterHandle,
    pub view: DataView,
    pub task_handles: HashMap<BrokerTaskID, BrokerTaskHandle>,
    pub task_configs: HashMap<BrokerTaskID, BrokerTaskConfig>,
}

impl BrokerNode {
    pub fn new(info: BrokerNodeInfo, adapter: BrokerAdapterHandle) -> Self {
        Self {
            info,
            adapter,
            view: DataView::new(),
            task_handles: HashMap::new(),
            task_configs: HashMap::new()
        }
    }

    pub fn init(&mut self) -> Result<(), anyhow::Error> {
        for task_handle in self.task_handles.values_mut() {
            task_handle.lock().unwrap().init()?;
        }
        Ok(())
    }

    pub fn add_task(&mut self, task_handle: BrokerTaskHandle) -> Result<(), anyhow::Error> {
        let task_config = task_handle.lock().unwrap().get_config();
        info!(
            "Node {:?} - Adding task {:?} to node {:?}",
            self.info.node_id, task_config.task_id, self.info.node_id
        );
        self.task_handles.insert(task_config.task_id, task_handle);
        self.task_configs
            .insert(task_config.task_id, task_config.clone());

        self.adapter.try_lock().unwrap().send_new_task(&task_config)?;
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), anyhow::Error> {
        // 1. Check for any execute requests from the adapter
        let adapter = self.adapter.clone();
        let mut adapter = adapter.try_lock().unwrap();

        // 1. Get any inputs from the adapter and add them to the view
        let mut n_updates = 0;
        while let Ok(inputs) = adapter.recv_inputs() {
            if inputs.len() == 0 {
                break;
            }
            n_updates += inputs.len();
           
            for datapoint in inputs {
                self.view.add_datapoint(datapoint)?;
            }
        }

        if n_updates > 0 {
            debug!(
                "Node {:?} - Received {:?} new updates",
                self.info.name, n_updates
            );
        }

        // 3. Execute the tasks
        let execute_requests = adapter.recv_execute()?;
        let task_configs = self.task_configs.clone();
        // 2. Execute the tasks
        // TODO: Parallelize this so tasks can execute in parallel
        for (task_config, time) in execute_requests.iter() {
            
            // Get our copy of the task_config and inputs
            let task_config = task_configs.get(&task_config.task_id).unwrap();
            let inputs = self.get_inputs(&task_config, &time.time_last_monotonic.clone().unwrap_or_default())?;
            debug!(
                "Node {:?} // Executing task {:?} with {:?} total inputs",
                self.info.name, task_config.name, inputs.maps.keys().len()
            );


            // Execute the task
            let task_handle = self
                .task_handles
                .get_mut(&task_config.task_id)
                .expect("Task not found");
            let results = task_handle.lock().unwrap().on_execute(&inputs, &time)?;

            // 4. Send the results back to the adapter using send outputs
            let outputs = results.get_all_datapoints();
            let mut total_sent = 0;
            for chunk in outputs.chunks(32) {
                total_sent += chunk.len();
           
                adapter.send_outputs(&chunk.to_vec())?;
            }
            debug!(
                "Node {:?} - Sent {:?} outputs for task {:?}",
                self.info.name, total_sent, task_config.name
            );

            // 5. Send the response back to the adapter using send response
            adapter.send_response(task_config)?;
        }

        Ok(())
    }

    fn get_inputs(&mut self, task: &BrokerTaskConfig, prev_time: &Timepoint) -> Result<DataView, BrokerAdapterError> {
        let mut inputs = DataView::new();
        
        for subscription in &task.subscriptions {
            match subscription.mode {
                SubscriptionMode::Latest => {
                    // Get all latest values matching the topic query
                    inputs = inputs
                        .add_query_from_view(&self.view, &subscription.topic_query)
                        .unwrap();
                }
                SubscriptionMode::NewValues => {
                    // Get only values after last execution
                    inputs = inputs
                        .add_query_after_from_view(&self.view, &subscription.topic_query, &prev_time)
                        .unwrap();

                    // Remove all datapoints that match the topic query
                   // self.view.remove_query(&subscription.topic_query);
                }
            }
        }

        Ok(inputs)
    }
}
