use std::{time::Duration};

use axum::{Router, routing::get};
use tokio::{time::sleep};

mod block;
mod node;
mod network;

use block::{Block};
use node::Node;
use network::Network;


#[tokio::main]
async fn main() {
    let mut network = Network::new();
    let genesis_block = Block::new(0 ,Default::default(), 0);
    let network_latency = 1000;

    let mut handles = vec![];
    
    for _ in 0..5 {
        println!("Creating node");
        let mut node = Node::new(genesis_block.clone(), network.connect().await);
        handles.push(tokio::spawn(async move {
            loop {
                node.work().await;
            }
        }));
    }

    handles.push(tokio::spawn(async move {
        loop {
            network.work().await;
            sleep(Duration::from_millis(network_latency)).await;
        }
    }));

    for handle in handles {
        handle.await.unwrap();
    }

    // let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    // let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    // axum::serve(listener, app).await.unwrap();
}
