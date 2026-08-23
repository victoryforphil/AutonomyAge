use core::time;

use log::info;
use tokio::net::TcpStream;
use victory_wtf::Timepoint;

use super::{
    connection::{TcpBrokerConnection, TcpBrokerConnectionHandle},
    message::TcpBrokerMessage,
};
use crate::{adapters::{BrokerAdapter, BrokerAdapterError}, broker::time::BrokerTime};
use crate::task::config::BrokerTaskConfig;
use victory_data_store::{database::view::DataView, datapoints::Datapoint};

pub struct TcpBrokerClient {
    address: String,
    connection: TcpBrokerConnectionHandle,
    // Internal queues for managing tasks and responses
    new_tasks: Vec<BrokerTaskConfig>,
    execute_queue: Vec<(BrokerTaskConfig, BrokerTime)>,
    response_queue: Vec<BrokerTaskConfig>,
    inputs: Vec<Datapoint>,
    outputs: Vec<Datapoint>,
}

impl TcpBrokerClient {
    pub async fn new(address: &str) -> Result<Self, BrokerAdapterError> {
        info!("[Broker/TcpClient] Connecting to {}", address);
        let stream = TcpStream::connect(address)
            .await
            .map_err(|e| BrokerAdapterError::Generic(Box::new(e)))?;
        let connection = TcpBrokerConnection::new(stream).await;

        Ok(Self {
            address: address.to_string(),
            connection,
            new_tasks: Vec::new(),
            execute_queue: Vec::new(),
            response_queue: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        })
    }

    fn process_incoming_messages(&mut self) {
        let conn = self.connection.clone();
        let mut conn = conn.try_lock().unwrap();
        let connection_id = conn.connection_id;
        while let Ok(message) = conn.recv_rx.try_recv() {
            match message {
                TcpBrokerMessage::NewTask(mut task_config) => {
                    task_config.connection_id = connection_id;
                    self.new_tasks.push(task_config);
                }
                TcpBrokerMessage::ExecuteTask(mut task_config, time) => {
                    task_config.connection_id = connection_id;
                    self.execute_queue.push((task_config, time));
                }
                TcpBrokerMessage::TaskResponse(mut task_config) => {
                    task_config.connection_id = connection_id;
                    self.response_queue.push(task_config);
                }
                TcpBrokerMessage::Inputs(inputs) => {
                    self.inputs.extend(inputs);
                }
                TcpBrokerMessage::Outputs(outputs) => {
                    self.outputs.extend(outputs);
                }
            }
        }
    }
}

impl BrokerAdapter for TcpBrokerClient {
    fn get_new_tasks(&mut self) -> Result<Vec<BrokerTaskConfig>, BrokerAdapterError> {
        self.process_incoming_messages();
        Ok(self.new_tasks.drain(..).collect())
    }

    fn send_new_task(&mut self, task: &BrokerTaskConfig) -> Result<(), BrokerAdapterError> {
        let message = TcpBrokerMessage::NewTask(task.clone());
        let conn = self.connection.clone();
        let conn = conn.try_lock().unwrap();
        conn.send_tx
            .try_send(message)
            .map_err(|e| BrokerAdapterError::Generic(Box::new(e)))?;
        Ok(())
    }

    fn send_execute(
        &mut self,
        task: &BrokerTaskConfig,
        time: &BrokerTime
    ) -> Result<(), BrokerAdapterError> {
        let message = TcpBrokerMessage::ExecuteTask(task.clone(), time.clone());
        let conn = self.connection.clone();
        let conn = conn.try_lock().unwrap();
        conn.send_tx
            .try_send(message)
            .map_err(|e| BrokerAdapterError::Generic(Box::new(e)))?;
        Ok(())
    }

    fn recv_response(&mut self, _task: &BrokerTaskConfig) -> Result<(), BrokerAdapterError> {
        self.process_incoming_messages();
        if let Some(pos) = self.response_queue.iter().position(|task_config| task_config.task_id == _task.task_id) {
            self.response_queue.remove(pos);
            Ok(())
            } else {
                return Err(BrokerAdapterError::WaitingForTaskResponse)
        } 
    }

    fn recv_execute(&mut self) -> Result<Vec<(BrokerTaskConfig, BrokerTime)>, BrokerAdapterError> {
        self.process_incoming_messages();
        Ok(self.execute_queue.drain(..).collect())
    }

    fn send_response(
        &mut self,
        task: &BrokerTaskConfig
    ) -> Result<(), BrokerAdapterError> {
        let message = TcpBrokerMessage::TaskResponse(task.clone());
        let conn = self.connection.clone();
        let conn = conn.try_lock().unwrap();
        conn.send_tx
            .try_send(message)
            .map_err(|e| BrokerAdapterError::Generic(Box::new(e)))?;
        Ok(())
    }

    fn send_inputs(&mut self, inputs: &Vec<Datapoint>) -> Result<(), BrokerAdapterError> {
        let message = TcpBrokerMessage::Inputs(inputs.clone());
        let conn = self.connection.clone();
        let conn = conn.try_lock().unwrap();
        conn.send_tx
            .try_send(message)
            .map_err(|e| BrokerAdapterError::Generic(Box::new(e)))?;
        Ok(())
    }
    
    fn recv_inputs(&mut self) -> Result<Vec<Datapoint>, BrokerAdapterError> {
        self.process_incoming_messages();
        if self.inputs.is_empty() {
            Err(BrokerAdapterError::NoPendingInputs)
        } else {
            Ok(self.inputs.drain(..).collect())
        }
    }

    fn send_outputs(&mut self, outputs: &Vec<Datapoint>) -> Result<(), BrokerAdapterError> {
        let message = TcpBrokerMessage::Outputs(outputs.clone());
        let conn = self.connection.clone();
        let conn = conn.try_lock().unwrap();
        conn.send_tx
            .try_send(message)
            .map_err(|e| BrokerAdapterError::Generic(Box::new(e)))?;
        Ok(())
    }

    fn recv_outputs(&mut self) -> Result<Vec<Datapoint>, BrokerAdapterError> {
        self.process_incoming_messages();
        if self.outputs.is_empty() {
            Err(BrokerAdapterError::NoPendingOutputs)
        } else {
            Ok(self.outputs.drain(..).collect())
        }
    }
}
