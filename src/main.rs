use std::{collections::HashMap, time::Duration};

use axum::{Router, routing::get};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use tokio::{sync::mpsc, time::sleep};

type BlockHash = [u8; 32];

#[derive(Clone, Debug)]
struct Block {
    height: u64,
    prev_hash: BlockHash,
    guess: u128,
    hash: BlockHash,
}

fn is_block_hash_valid(block_hash: &BlockHash, difficulty: u32) -> bool {
    let mut trailing_zeros = 0;

    for digit in block_hash.iter().rev() {
        let zeros = digit.trailing_zeros();
        trailing_zeros += zeros;

        if zeros != u8::BITS {
            break;
        }
    }

    trailing_zeros >= difficulty
}


impl Block {
    fn hash(&self) -> BlockHash {
        return self.hash;
    }

    fn new(height: u64, prev_hash: BlockHash, guess: u128) -> Self {
        let mut block = Block {
            height,
            prev_hash,
            guess,
            hash: Default::default()
        };

        block.recalculate_hash();
        block
    }

    fn calculate_hash(&self) -> BlockHash {
        let mut hasher = Sha256::new();
        hasher.update(self.height.to_be_bytes());
        hasher.update(self.prev_hash);
        hasher.update(self.guess.to_be_bytes());
        let hash: [u8; 32] = hasher.finalize().into();

        return hash;
    }

    fn recalculate_hash(&mut self) {
        self.hash = self.calculate_hash();
    }
}

struct Node {
    connection: NetworkConnection,
    blocks: HashMap<BlockHash, Block>,
    last_block_hash: BlockHash,
    hash_difficulty: u32,
}

impl Node {
    fn new(start_block: Block, connection: NetworkConnection) -> Node {
        let last_block_hash = start_block.hash();

        let mut blocks = HashMap::new();
        blocks.insert(last_block_hash, start_block.clone());

        Node {
            connection,
            blocks,
            hash_difficulty: 4 * 5,
            last_block_hash,
        }
    }

    async fn work(&mut self) {
        self.mine().await;

        while let Ok(message) = self.connection.receive().await {
            match message.data {
                NetworkMessageData::PublishBlock(new_block) => {
                    self.add_block(new_block);
                }
            }
        }
    }

    async fn mine(&mut self) {
        let prev_block: &Block = self.blocks.get(&self.last_block_hash).unwrap();
        let prev_block_hash = prev_block.hash();
        let height = prev_block.height;

        let guess = rand::random();
        let new_block = Block::new(height + 1, prev_block_hash, guess);
        let new_block_hash = new_block.hash();

        if is_block_hash_valid(&new_block_hash, self.hash_difficulty) {
            println!(
                "[{0}] Found {1}::{2}",
                self.connection.id,
                new_block.height,
                hex::encode(new_block_hash),
            );
            self.connection.send(NetworkMessageData::PublishBlock(new_block)).await.unwrap();
        }
    }

    fn add_block(&mut self, new_block: Block) {
        let new_block_hash = new_block.hash();
        let new_block_height = new_block.height;

        if !self.is_block_valid(&new_block) {
            println!(
                "[{0}] Rejecting Block {1}::{2}",
                self.connection.id,
                new_block_height,
                hex::encode(new_block_hash),
            );
            return;
        }

        // println!(
        //     "[{0}] Block {1}::{2}",
        //     self.connection.id,
        //     new_block_height,
        //     hex::encode(new_block_hash),
        // );
        self.blocks.insert(new_block_hash, new_block);

        //Safe to unwrap because there is at least genesis block hash there
        let last_block = self.blocks.get(&self.last_block_hash).unwrap();

        if new_block_height > last_block.height {
            println!(
                "[{0}] Highest block {1}::{2}",
                self.connection.id,
                new_block_height,
                hex::encode(new_block_hash),
            );
            self.last_block_hash = new_block_hash
        }
    }

    fn is_block_valid(&self, block: &Block) -> bool {
        match self.blocks.get(&block.prev_hash) {
            Some(prev_block) => {
                block.height == prev_block.height + 1 && is_block_hash_valid(&block.hash(), self.hash_difficulty)
            },
            None => false
        }
    }
}

#[derive(Clone, Debug)]
enum NetworkMessageData {
    PublishBlock(Block)
}

type ConnectionId = Uuid;

#[derive(Clone, Debug)]
struct NetworkMessage {
    sender: ConnectionId,
    data: NetworkMessageData,
}


struct NetworkConnection {
    id: ConnectionId,
    tx: mpsc::Sender<NetworkMessage>,
    rx: mpsc::Receiver<NetworkMessage>
}

#[derive(Debug)]
enum NetworkError {
    Error(String),
    NoMessagesToReceive,
    Failed
}

impl NetworkConnection {
    fn new(id: ConnectionId, tx: mpsc::Sender<NetworkMessage>, rx: mpsc::Receiver<NetworkMessage>) -> Self {
        NetworkConnection {
            id,
            tx,
            rx
        }
    }

    async fn send(&mut self, data: NetworkMessageData) -> Result<(), NetworkError> {
        let msg = NetworkMessage {
            sender: self.id,
            data
        };

        self.tx.send(msg).await.map_err(|_err| NetworkError::Failed)
    }

    async fn receive(&mut self) -> Result<NetworkMessage, NetworkError> {
        self.rx.try_recv().map_err(|_err| NetworkError::NoMessagesToReceive)
    }
}

#[derive(Default)]
struct Network {
    in_rxs: Vec<mpsc::Receiver<NetworkMessage>>,
    out_txs: Vec<mpsc::Sender<NetworkMessage>>
}

impl Network {
    fn new() -> Network {
        Network {
            ..Default::default()
        }
    }

    async fn connect(&mut self) -> NetworkConnection {
        let (in_tx, in_rx) = mpsc::channel::<NetworkMessage>(32);
        let (out_tx, out_rx) = mpsc::channel::<NetworkMessage>(32);

        let connection_id = Uuid::new_v4();

        self.in_rxs.push(in_rx);
        self.out_txs.push(out_tx);

        NetworkConnection::new(
            connection_id,
            in_tx,
            out_rx
        )
    }

    async fn work(&mut self) {
        //Remove closed channels
        self.in_rxs.retain(|in_rx| !in_rx.is_closed());
        self.out_txs.retain(|out_tx| !out_tx.is_closed());

        for in_rx in self.in_rxs.iter_mut() {
            if let Ok(message) = in_rx.try_recv() {
                for out_tx in self.out_txs.iter_mut() {
                    out_tx.send(message.clone()).await.unwrap();
                }
            }
        }
    }
}

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
