use std::collections::HashMap;
use crate::{block::{Block, BlockHash}, network::{NetworkConnection, NetworkMessageData}};

pub struct Node {
    connection: NetworkConnection,
    blocks: HashMap<BlockHash, Block>,
    last_block_hash: BlockHash,
    hash_difficulty: u32,
}

impl Node {
    pub fn new(start_block: Block, connection: NetworkConnection) -> Node {
        let last_block_hash = start_block.hash();

        let mut blocks = HashMap::new();
        blocks.insert(last_block_hash.clone(), start_block.clone());

        Node {
            connection,
            blocks,
            hash_difficulty: 4 * 5,
            last_block_hash: last_block_hash.clone(),
        }
    }

    pub async fn work(&mut self) {
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
        let new_block = Block::new(height + 1, prev_block_hash.clone(), guess);
        let new_block_hash = new_block.hash();

        if new_block_hash.is_valid(self.hash_difficulty) {
            println!(
                "[{0}] Found {1}::{2}",
                self.connection.id,
                new_block.height,
                new_block_hash.hex_encode()
            );
            self.connection.send(NetworkMessageData::PublishBlock(new_block.clone())).await.unwrap();
            self.add_block(new_block);
        }
    }

    fn add_block(&mut self, new_block: Block) {
        let new_block_hash = new_block.hash().clone();
        let new_block_height = new_block.height;

        if !self.is_block_valid(&new_block) {
            println!(
                "[{0}] Rejecting Block {1}::{2}",
                self.connection.id,
                new_block_height,
                new_block_hash.hex_encode()
            );
            return;
        }

        // println!(
        //     "[{0}] Block {1}::{2}",
        //     self.connection.id,
        //     new_block_height,
        //     hex::encode(new_block_hash),
        // );
        self.blocks.insert(new_block_hash.clone(), new_block);

        //Safe to unwrap because there is at least genesis block hash there
        let last_block = self.blocks.get(&self.last_block_hash).unwrap();

        if new_block_height > last_block.height {
            println!(
                "[{0}] Highest block {1}::{2}",
                self.connection.id,
                new_block_height,
                new_block_hash.hex_encode(),
            );
            self.last_block_hash = new_block_hash.clone()
        }
    }

    fn is_block_valid(&self, block: &Block) -> bool {
        match self.blocks.get(&block.prev_hash) {
            Some(prev_block) => {
                block.height == prev_block.height + 1 && block.hash().is_valid(self.hash_difficulty)
            },
            None => false
        }
    }
}
