use std::{collections::HashMap, fmt::Display};
use primitive_types::U256;

use crate::{block::{Block, BlockHash}, network::{NetworkConnection, NetworkMessageData}, transaction::{Transaction, TransactionAction}, wallet::{PubKey, Wallet}};

#[derive(Clone, Debug, Default)]
pub struct BlockState {
    balances: HashMap<PubKey, U256>,
}

impl Display for BlockState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        writeln!(f, "BlockState::balances")?;

        for (pub_key, val) in &self.balances {
            writeln!(f, "{} : {}", pub_key, val)?;
        }

        Ok(())
    }
}

pub struct Node {
    connection: NetworkConnection,
    blocks: HashMap<BlockHash, Block>,
    block_states: HashMap<BlockHash, BlockState>,
    last_block_hash: BlockHash,
    hash_difficulty: u32,
    wallet: Wallet,
}

impl Node {
    pub fn new(start_block: Block, connection: NetworkConnection, wallet: Wallet) -> Node {
        let last_block_hash = start_block.hash();

        let mut blocks = HashMap::new();
        blocks.insert(last_block_hash.clone(), start_block.clone());

        let mut block_states = HashMap::new();
        block_states.insert(last_block_hash.clone(), Default::default());

        Node {
            connection,
            block_states,
            blocks,
            last_block_hash: last_block_hash.clone(),
            hash_difficulty: 4 * 5,
            wallet,
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
        let reward_transaction = Transaction::block_reward(
            self.wallet.pub_key.clone()
        );

        let transactions = vec![reward_transaction];

        let new_block = Block::new(
            height + 1,
            prev_block_hash.clone(),
            guess,
            transactions
        );
        let new_block_hash = new_block.hash();

        if self.is_block_valid(&new_block, &prev_block) {
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

        let prev_block = match self.blocks.get(&new_block.prev_hash) {
            Some(block) => block.clone(),
            None => {
                println!(
                    "[{0}] Prev block not found {1}::{2}",
                    self.connection.id,
                    new_block_height,
                    new_block_hash.hex_encode()
                );

                return
            }
        };

        if !self.is_block_valid(&new_block, &prev_block) {
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

        self.create_block_state(&new_block);

        self.blocks.insert(new_block_hash.clone(), new_block);

        if let Some(last_block) = self.blocks.get(&self.last_block_hash) {
            if new_block_height > last_block.height {
                // println!(
                //     "[{0}] Highest block {1}::{2}",
                //     self.connection.id,
                //     new_block_height,
                //     new_block_hash.hex_encode(),
                // );
                self.last_block_hash = new_block_hash.clone()
            }
        } else {
            self.last_block_hash = new_block_hash.clone()
        }
    }

    fn create_block_state(&mut self, block: &Block) -> BlockState {
        let prev_block_state = self.block_states.get(&block.prev_hash).expect("Prev block state not found!");

        let mut block_state = prev_block_state.clone();

        for transaction in &block.transactions {
            for action in &transaction.actions {
                match action {
                    TransactionAction::BlockReward(block_reward_action) => {
                        *block_state.balances.entry(block_reward_action.receiver.clone()).or_insert(U256::zero()) += U256::from(1);
                    }
                }
            }
        }

        let block_hash = block.hash().clone();
        self.block_states.insert(block_hash, block_state.clone());

        println!("{}", block_state);

        block_state
    }

    fn is_block_valid(&self, block: &Block, prev_block: &Block) -> bool {
        block.height == prev_block.height + 1 && block.is_valid(self.hash_difficulty)
    }
}
