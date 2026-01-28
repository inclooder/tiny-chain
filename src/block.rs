use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct BlockHash([u8; 32]);

impl BlockHash {
    pub fn is_valid(&self, difficulty: u32) -> bool {
        let mut trailing_zeros = 0;

        for digit in self.0.iter().rev() {
            let zeros = digit.trailing_zeros();
            trailing_zeros += zeros;

            if zeros != u8::BITS {
                break;
            }
        }

        trailing_zeros >= difficulty
    }

    pub fn hex_encode(&self) -> String {
        hex::encode(self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub height: u64,
    pub prev_hash: BlockHash,
    pub guess: u128,
    pub hash: BlockHash,
}


impl Block {
    pub fn hash(&self) -> &BlockHash {
        return &self.hash;
    }

    pub fn new(height: u64, prev_hash: BlockHash, guess: u128) -> Self {
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
        hasher.update(self.prev_hash.0);
        hasher.update(self.guess.to_be_bytes());
        let hash: [u8; 32] = hasher.finalize().into();

        return BlockHash(hash);
    }

    fn recalculate_hash(&mut self) {
        self.hash = self.calculate_hash();
    }
}
