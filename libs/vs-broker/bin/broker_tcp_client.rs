use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use clap::Parser;
use log::info;

use vs_broker::{
    adapters::tcp::tcp_client::TcpBrokerClient,
    node::{info::BrokerNodeInfo, BrokerNode},
    task::example::{task_printer::TaskPrinter, task_ticker::TaskTicker},
};
use vs_data_store::topics::TopicKey;

#[derive(Parser)]
struct Args {
    /// IP address to connect to
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// Port to connect to
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let args = Args::parse();
    let connect_addr = format!("{}:{}", args.address, args.port);

    info!("Broker Test TCP Client // Connecting to {}", connect_addr);

    let client: TcpBrokerClient = TcpBrokerClient::new(&connect_addr).await.unwrap();

    let topic = TopicKey::from_str("test/topic/a");
    let task_a = TaskTicker::new(topic.clone());
    let task_print = TaskPrinter::new(topic.clone());

    let mut node = BrokerNode::new(
        BrokerNodeInfo::new("Broker Test TCP Client"),
        Arc::new(tokio::sync::Mutex::new(client)),
    );
    node.add_task(Arc::new(Mutex::new(task_a))).unwrap();
    node.add_task(Arc::new(Mutex::new(task_print))).unwrap();

    loop {
        node.tick().unwrap();
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
