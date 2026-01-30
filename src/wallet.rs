use once_cell::sync::Lazy;
use secp256k1::{All, PublicKey, Secp256k1, SecretKey};

static SECP: Lazy<Secp256k1<All>> = Lazy::new(|| Secp256k1::new());

pub struct PubKey([u8; 65]);
pub struct PrivKey([u8; 32]);

pub struct Wallet {
    pub_key: PubKey,
    priv_key: PrivKey,
}


pub struct Signature {
    sig: [u8; 64],
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
