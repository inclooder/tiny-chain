use std::str::FromStr;

use hex::FromHexError;
use once_cell::sync::Lazy;
use secp256k1::ecdsa::RecoveryId;
use secp256k1::{All, Message, PublicKey, Secp256k1, SecretKey};
use secp256k1::hashes::{sha256, Hash};

static SECP: Lazy<Secp256k1<All>> = Lazy::new(|| Secp256k1::new());

#[derive(Debug, Clone)]
pub struct PubKey([u8; 65]);

#[derive(Debug, Clone)]
pub struct PrivKey([u8; 32]);

pub struct Wallet {
    pub pub_key: PubKey,
    pub priv_key: PrivKey,
}

pub struct Signature {
    data: [u8; 64],
    recid: u8,
}

impl Wallet {
    pub fn generate() -> Self {
        let (secret_key, public_key) = SECP.generate_keypair(&mut rand::rng());

        Self {
            pub_key: public_key.into(),
            priv_key: secret_key.into()
        }
    }

    pub fn from_priv_key(priv_key: PrivKey) -> Self {
        let secret_key = SecretKey::from_byte_array(priv_key.0).unwrap();
        let public_key = PublicKey::from_secret_key(&SECP, &secret_key);

        Self {
            pub_key: public_key.into(),
            priv_key: priv_key
        }

    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        let digest = sha256::Hash::hash(message);
        let message = Message::from_digest(digest.to_byte_array());

        let sk = self.priv_key.clone().try_into().unwrap();

        let recoverable_sig = SECP.sign_ecdsa_recoverable(message, &sk);

        let (recovery_id, data) = recoverable_sig.serialize_compact();

        Signature { 
            data: data,
            recid: recovery_id_to_number(&recovery_id)
        }
    }
}

fn recovery_id_to_number(recovery_id: &RecoveryId) -> u8 {
    match recovery_id {
        RecoveryId::Zero => 0,
        RecoveryId::One => 1,
        RecoveryId::Two => 2,
        RecoveryId::Three => 3,
    }
}

impl From<PublicKey> for PubKey {
    fn from(value: PublicKey) -> Self {
        Self(value.serialize_uncompressed())
    }
}

impl From<SecretKey> for PubKey {
    fn from(value: SecretKey) -> Self {
        let public_key = PublicKey::from_secret_key(&SECP, &value);
        public_key.into()
    }
}

impl From<SecretKey> for PrivKey {
    fn from(value: SecretKey) -> Self {
        Self(value.secret_bytes())
    }
}

impl Into<String> for PubKey {
    fn into(self) -> String {
        hex::encode(self.0)
    }
}

impl TryInto<SecretKey> for PrivKey {
    type Error =  secp256k1::Error;

    fn try_into(self) -> Result<SecretKey, Self::Error> {
        SecretKey::from_byte_array(self.0)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PubKeyParseError {
    #[error(transparent)]
    Hex(#[from] hex::FromHexError),
    #[error("Invalid Length {0}")]
    InvalidLength(usize)
}

impl FromStr for PubKey {
    type Err = PubKeyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = hex::decode(s)?;
        let bytes: [u8; 65] = decoded.clone().try_into().map_err(|_| PubKeyParseError::InvalidLength(decoded.len()))?;

        Ok(PubKey(bytes))
    }
}

impl Into<String> for PrivKey {
    fn into(self) -> String {
        hex::encode(self.0)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_pub_key_to_string() {
        let pub_key_bytes: [u8; 65] = std::array::from_fn(|i| i as u8);
        let pub_key = PubKey(pub_key_bytes);

        let pub_key_string: String = pub_key.into();

        assert_eq!(pub_key_string, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f40");
    }

    #[test]
    fn test_convert_string_to_pub_key() {
        let input_string: String = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f40".into();

        let pub_key: PubKey = input_string.parse().unwrap();

        let pub_key_string: String = pub_key.into();

        assert_eq!(pub_key_string, input_string);
    }
}
