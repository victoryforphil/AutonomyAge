use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{info, warn};

use clap::Parser;
use victory_broker::{
    adapters::tcp::tcp_server::TcpBrokerServer, broker::Broker,
    commander::linear::LinearBrokerCommander,
};
use victory_data_store::topics::TopicKey;
use victory_wtf::Timespan;
#[derive(Parser)]
struct Args {
    /// IP address to bind to
    #[arg(short, long, default_value = "0.0.0.0")]
    address: String,

    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let args = Args::parse();
    let bind_addr = format!("{}:{}", args.address, args.port);

    info!("Broker Test TCP Server // Binding to {}", bind_addr);

    let server: TcpBrokerServer = TcpBrokerServer::new(&bind_addr).await.unwrap();
    let mut broker = Broker::new(LinearBrokerCommander::new());
    broker.add_adapter(Arc::new(tokio::sync::Mutex::new(server)));
    // Create channel adapter pair for local node
    let (adapter_a, adapter_b) =
        victory_broker::adapters::channel::ChannelBrokerAdapter::new_pair();
    broker.add_adapter(adapter_a);

    // Create local node
    let node_info = victory_broker::node::info::BrokerNodeInfo::new("local_node");
    let mut node = victory_broker::node::BrokerNode::new(node_info, adapter_b);

    // Create printer task to monitor all topics
    let printer_task =
        victory_broker::task::example::task_printer::TaskPrinter::new(TopicKey::from_str(""));
    node.add_task(Arc::new(Mutex::new(printer_task))).unwrap();

    // Spawn node thread
    let node_handle = Arc::new(Mutex::new(node));
    let node_thread = tokio::spawn(async move {
        loop {
            {
                let mut node = node_handle.lock().unwrap();
                if let Err(e) = node.tick() {
                    warn!("Node // Error: {:?}", e);
                }
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
            // Clear the console
            print!("\x1B[2J\x1B[1;1H");
        }
    });
    loop {
        match broker.tick(Timespan::new_ms(5.0)).await {
            Ok(_) => (),
            Err(e) => {
                warn!("Broker // Error: {:?}", e);
            }
        }
        // Sleep for 100ms
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
