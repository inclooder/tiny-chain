use crate::wallet::PubKey;

#[derive(Clone, Debug)]
pub struct BlockRewardAction {
    pub receiver: PubKey
}

#[derive(Clone, Debug)]
pub enum TransactionAction {
    BlockReward(BlockRewardAction)
}

impl TransactionAction {
    pub fn is_valid(&self) -> bool {
        match self {
            TransactionAction::BlockReward(_block_reward_action) => true
        }
    }
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
        self.actions.iter().all(|e| e.is_valid())
    }
}
