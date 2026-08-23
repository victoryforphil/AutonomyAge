use std::sync::{Arc, Mutex};

use config::BrokerTaskConfig;
use vs_data_store::database::view::DataView;

use crate::broker::time::BrokerTime;

pub mod example;
pub mod state;
pub mod subscription;
pub mod trigger;

pub mod config;

pub type BrokerTaskID = u32;

pub type BrokerTaskHandle = Arc<Mutex<dyn BrokerTask>>;

pub trait BrokerTask: Send {
    fn init(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }
    fn get_config(&self) -> BrokerTaskConfig;
    fn on_execute(&mut self, inputs: &DataView, timing: &BrokerTime) -> Result<DataView, anyhow::Error>;
}
