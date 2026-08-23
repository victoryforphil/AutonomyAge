use vs_data_store::{database::view::DataView, topics::TopicKey};

use crate::{broker::time::BrokerTime, task::{
    config::{BrokerCommanderFlags, BrokerTaskConfig},
    subscription::BrokerTaskSubscription,
    trigger::BrokerTaskTrigger,
    BrokerTask,
}};

pub struct TaskPrinter {
    pub subscribe_topic: TopicKey,
}

impl TaskPrinter {
    pub fn new(subscribe_topic: TopicKey) -> Self {
        Self { subscribe_topic }
    }
}

impl BrokerTask for TaskPrinter {
    fn init(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn get_config(&self) -> BrokerTaskConfig {
        BrokerTaskConfig::new("TaskPrinter")
            .with_trigger(BrokerTaskTrigger::Always)
            .with_subscription(BrokerTaskSubscription::new_latest(&self.subscribe_topic))
            .with_flag(BrokerCommanderFlags::NonBlocking)
    }

    fn on_execute(
        &mut self,
        inputs: &vs_data_store::database::view::DataView,
        _timing: &BrokerTime,
    ) -> Result<DataView, anyhow::Error> {
        if let Ok(values) = inputs.get_latest_map(&self.subscribe_topic) {
            // if values is empty, print empty message

            if values.is_empty() {
                return Ok(DataView::new());
            }

            println!("+------------------------+------------------------+");
            println!("| Topic                 | Value                  |");
            println!("+------------------------+------------------------+");
            for (topic, value) in values {
                println!("| {:<22} | {:<20?} |", topic, value);
            }
            println!("+------------------------+------------------------+");
        } else {
            println!(
                "TaskPrinter: No data available for topic '{}'",
                self.subscribe_topic
            );
        }
        Ok(DataView::new())
    }
}
