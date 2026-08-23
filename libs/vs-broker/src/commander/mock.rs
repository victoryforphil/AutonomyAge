use log::{debug, info};

use crate::task::config::BrokerTaskConfig;

use super::BrokerCommander;

/// Mock implementation of the BrokerCommander trait
/// Will simply instantly return the tasks added to it
pub struct MockBrokerCommander {
    tasks: Vec<BrokerTaskConfig>,
}

impl Default for MockBrokerCommander {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBrokerCommander {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }
}

impl BrokerCommander for MockBrokerCommander {
    fn add_task(&mut self, task: BrokerTaskConfig) -> Result<(), super::BrokerCommanderError> {
        info!(
            "MockCommander // Add Task // New task added: {:?}",
            task.task_id
        );
        self.tasks.push(task);
        Ok(())
    }

    fn get_next_tasks(&mut self) -> Result<Vec<BrokerTaskConfig>, super::BrokerCommanderError> {
        // Drain only the next 1 task from the tasks vector
        if self.tasks.is_empty() {
            debug!("MockCommander // Next Task // No tasks to get");
            return Ok(vec![]);
        }
        let drained_tasks = self.tasks.drain(..1).collect();
        Ok(drained_tasks)
    }

    fn remove_task(
        &mut self,
        task_id: crate::task::BrokerTaskID,
    ) -> Result<(), super::BrokerCommanderError> {
        self.tasks.retain(|task| task.task_id != task_id);
        Ok(())
    }
}

// -
// -
// -

#[cfg(test)]
mod mock_commander_tests {
    use super::*;

    /// Test the add_task method
    /// 1. Add a task to the commander
    /// 2. Check that the task was added to the tasks vector
    #[test]
    fn test_add_task() {
        let mut commander = MockBrokerCommander::new();
        let task = BrokerTaskConfig::new_with_id(0, "test");
        commander.add_task(task).unwrap();
    }

    /// Test the get_next_tasks method
    /// 1. Add a task to the commander
    /// 2. Call get_next_tasks
    /// 3. Check that the returned vector has one task
    #[test]
    fn test_get_next_tasks() {
        let mut commander = MockBrokerCommander::new();
        let task = BrokerTaskConfig::new_with_id(0, "test");
        commander.add_task(task).unwrap();
        let next_tasks = commander.get_next_tasks().unwrap();
        assert_eq!(next_tasks.len(), 1);
        assert_eq!(next_tasks[0].task_id, 0);
    }
}
