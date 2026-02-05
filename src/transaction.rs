use crate::wallet::PubKey;

#[derive(Clone, Debug)]
pub struct BlockRewardAction {
    receiver: PubKey
}

#[derive(Clone, Debug)]
pub enum TransactionAction {
    BlockReward(BlockRewardAction)
}

#[derive(Clone, Debug)]
pub struct Transaction {
    pub actions: Vec<TransactionAction>,
}

impl Transaction {
    pub fn block_reward(receiver: PubKey) -> Self {
        Self {
            actions: vec![
                TransactionAction::BlockReward(BlockRewardAction { receiver })
            ]
        }
    }

    pub fn is_valid(&self) -> bool {
        true
    }
}
