use vs_data_store::{database::view::DataView, datapoints::Datapoint};
use vs_wtf::Timepoint;

use crate::{broker::time::BrokerTime, task::config::BrokerTaskConfig};

use super::{BrokerAdapter, BrokerAdapterError};

#[derive(Default)]

pub struct MockBrokerAdapter {
    pub new_tasks: Vec<BrokerTaskConfig>,
    pub executed_tasks: Vec<(BrokerTaskConfig, BrokerTime)>,
    pub inputs: Vec<Datapoint>,
    pub outputs: Vec<Datapoint>,
}

impl MockBrokerAdapter {
    pub fn new() -> Self {
        Self {
            new_tasks: vec![],
            executed_tasks: vec![],
            inputs: vec![],
            outputs: vec![],
        }
    }
}

impl BrokerAdapter for MockBrokerAdapter {
    fn get_new_tasks(&mut self) -> Result<Vec<BrokerTaskConfig>, BrokerAdapterError> {
        Ok(self.new_tasks.drain(..).collect())
    }

    fn send_execute(
        &mut self,
        task: &BrokerTaskConfig,
        time: &BrokerTime,
    ) -> Result<(), BrokerAdapterError> {
        self.executed_tasks.push((task.clone(), time.clone())); 
        Ok(())
    }

    fn recv_response(&mut self, _task: &BrokerTaskConfig) -> Result<(), BrokerAdapterError> {
        Ok(())
    }

    fn send_new_task(&mut self, _task: &BrokerTaskConfig) -> Result<(), BrokerAdapterError> {
        Ok(())
    }

    fn recv_execute(&mut self) -> Result<Vec<(BrokerTaskConfig, BrokerTime)>, BrokerAdapterError> {
        Ok(vec![])
    }

    fn send_response(
        &mut self,
        _task: &BrokerTaskConfig
    ) -> Result<(), BrokerAdapterError> {
        Ok(())
    }
    
    fn send_inputs(&mut self, inputs: &Vec<Datapoint>) -> Result<(), BrokerAdapterError> {
        self.inputs = inputs.clone();
        Ok(())
    }
    
    fn recv_inputs(&mut self) -> Result<Vec<Datapoint>, BrokerAdapterError> {
        Ok(self.inputs.drain(..).collect())
    }

    fn send_outputs(&mut self, outputs: &Vec<Datapoint>) -> Result<(), BrokerAdapterError> {
        self.outputs = outputs.clone();
        Ok(())
    }

    fn recv_outputs(&mut self) -> Result<Vec<Datapoint>, BrokerAdapterError> {
        Ok(self.outputs.drain(..).collect())
    }
}

#[cfg(test)]
mod broker_adapter_tests {
    use vs_wtf::Timespan;

    use super::*;

    /// Test the get_new_tasks method
    /// 1. Add a new task to the adapter
    /// 2. Call get_new_tasks
    /// 3. Check that the task was returned
    #[test]
    fn test_mock_adapter_get_new_tasks() {
        let mut adapter = MockBrokerAdapter::new();
        let tasks = vec![BrokerTaskConfig::new_with_id(0, "test_task")];
        adapter.new_tasks = tasks.clone();
        let new_tasks = adapter.get_new_tasks().unwrap();
        assert_eq!(new_tasks.len(), 1);
        assert_eq!(new_tasks[0].task_id, tasks[0].task_id);
    }

    /// Test the execute_task method
    /// 1. Call the execute_task method with a task and inputs
    /// 2. Check that the task and inputs were added to the executed_tasks vector
    #[test]
    fn test_mock_adapter_execute_task() {
        let mut adapter = MockBrokerAdapter::new();
        let task = BrokerTaskConfig::new_with_id(0, "test_task");
        let time = BrokerTime {
            time_monotonic: Timepoint::now(),
            time_delta: Timespan::zero(),
            time_last_monotonic: None,
        };
        adapter.send_execute(&task, &time).unwrap();
        assert_eq!(adapter.executed_tasks.len(), 1);
        assert_eq!(adapter.executed_tasks[0].0.task_id, task.task_id);
    }

    /// Test the check_task_response method
    /// 1. Call the check_task_response method with a task
    /// 2. Check for an Ok result
    #[test]
    fn test_mock_adapter_check_task_response() {
        let mut adapter = MockBrokerAdapter::new();
        let task = BrokerTaskConfig::new_with_id(0, "test_task");
        let response = adapter.recv_response(&task);
        assert!(response.is_ok());
    }
}
