pub type AdapterID = u32;
pub type ConnectionID = u32;



use std::sync::Arc;

use tokio::sync::Mutex;
use vs_data_store::{database::view::DataView, datapoints::Datapoint};
use vs_wtf::Timepoint;

use crate::{broker::time::BrokerTime, task::config::BrokerTaskConfig};

pub mod channel;
pub mod mock;
pub mod tcp;
pub type BrokerAdapterHandle = Arc<Mutex<dyn BrokerAdapter>>;

#[derive(thiserror::Error, Debug)]
pub enum BrokerAdapterError {
    #[error(transparent)]
    Generic(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// Waiting for task response
    #[error("Waiting for task response")]
    WaitingForTaskResponse,

    /// No Pending Inputs
    #[error("No pending inputs")]
    NoPendingInputs,

    /// No Pending Outputs
    #[error("No pending outputs")]
    NoPendingOutputs,
}

pub trait BrokerAdapter: Send {
    fn get_new_tasks(&mut self) -> Result<Vec<BrokerTaskConfig>, BrokerAdapterError>;
    fn send_new_task(&mut self, task: &BrokerTaskConfig) -> Result<(), BrokerAdapterError>;

    fn send_inputs(&mut self, inputs: &Vec<Datapoint>) -> Result<(), BrokerAdapterError>;
    fn recv_inputs(&mut self) -> Result<Vec<Datapoint>, BrokerAdapterError>;

    fn send_outputs(&mut self, outputs: &Vec<Datapoint>) -> Result<(), BrokerAdapterError>;
    fn recv_outputs(&mut self) -> Result<Vec<Datapoint>, BrokerAdapterError>;

    fn send_execute(
        &mut self,
        task: &BrokerTaskConfig,
        time: &BrokerTime,
    ) -> Result<(), BrokerAdapterError>;
    fn recv_execute(&mut self) -> Result<Vec<(BrokerTaskConfig, BrokerTime)>, BrokerAdapterError>;

    fn send_response(
        &mut self,
        task: &BrokerTaskConfig
    ) -> Result<(), BrokerAdapterError>;
    fn recv_response(&mut self, task: &BrokerTaskConfig) -> Result<(), BrokerAdapterError>;
}
