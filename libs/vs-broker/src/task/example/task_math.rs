use crate::{broker::time::BrokerTime, task::{config::BrokerTaskConfig, trigger::BrokerTaskTrigger, BrokerTask}};
use anyhow::{Error, Result};
use vs_data_store::{database::view::DataView, topics::TopicKey};

pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

pub struct TaskMath {
    pub input_topic_a: TopicKey,
    pub input_topic_b: TopicKey,
    pub output_topic: TopicKey,
    pub operation: Operation,
}

impl TaskMath {
    pub fn new(
        input_topic_a: TopicKey,
        input_topic_b: TopicKey,
        output_topic: TopicKey,
        operation: Operation,
    ) -> Self {
        Self {
            input_topic_a,
            input_topic_b,
            output_topic,
            operation,
        }
    }
}

impl BrokerTask for TaskMath {
    fn init(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn get_config(&self) -> BrokerTaskConfig {
        BrokerTaskConfig::new("TaskMath").with_trigger(BrokerTaskTrigger::Always)
    }

    fn on_execute(&mut self, inputs: &DataView, _timing: &BrokerTime) -> Result<DataView, Error> {
        let mut outputs = DataView::new();

        let value_a = inputs
            .get_latest::<_, u64>(&self.input_topic_a)
            .map_err(|_| anyhow::anyhow!("No value found for input_topic_a"))?;
        let value_b = inputs
            .get_latest::<_, u64>(&self.input_topic_b)
            .map_err(|_| anyhow::anyhow!("No value found for input_topic_b"))?;

        let result = match self.operation {
            Operation::Add => value_a + value_b,
            Operation::Sub => value_a - value_b,
            Operation::Mul => value_a * value_b,
            Operation::Div => {
                if value_b == 0 {
                    return Err(anyhow::anyhow!("Division by zero"));
                }
                value_a / value_b
            }
        };

        outputs.add_latest(&self.output_topic, result);
        Ok(outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_task_math_add() {
        let input_topic_a = TopicKey::from_str("input_a");
        let input_topic_b = TopicKey::from_str("input_b");
        let output_topic = TopicKey::from_str("output");
        let operation = Operation::Add;
        let mut task_math = TaskMath::new(
            input_topic_a.clone(),
            input_topic_b.clone(),
            output_topic.clone(),
            operation,
        );

        let mut inputs = DataView::new();
        inputs.add_latest(&input_topic_a, 10u64);
        inputs.add_latest(&input_topic_b, 5u64);

        let outputs = task_math.on_execute(&inputs, &BrokerTime::default()).unwrap();
        assert_eq!(outputs.get_latest::<_, u64>(&output_topic).unwrap(), 15u64);
    }

    #[test]
    fn test_task_math_divide_by_zero() {
        let input_topic_a = TopicKey::from_str("input_a");
        let input_topic_b = TopicKey::from_str("input_b");
        let output_topic = TopicKey::from_str("output");
        let operation = Operation::Div;
        let mut task_math = TaskMath::new(
            input_topic_a.clone(),
            input_topic_b.clone(),
            output_topic.clone(),
            operation,
        );

        let mut inputs = DataView::new();
        inputs.add_latest(&input_topic_a, 10u64);
        inputs.add_latest(&input_topic_b, 0u64);

        let result = task_math.on_execute(&inputs, &BrokerTime::default());
        assert!(result.is_err());
    }
}
