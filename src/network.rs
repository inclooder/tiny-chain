use uuid::Uuid;
use tokio::{sync::mpsc};
use std::collections::HashMap;

use crate::block::Block;

#[derive(Clone, Debug)]
pub enum NetworkMessageData {
    PublishBlock(Block)
}

type ConnectionId = Uuid;

#[derive(Clone, Debug)]
pub struct NetworkMessage {
    pub sender: ConnectionId,
    pub data: NetworkMessageData,
}


pub struct NetworkConnection {
    pub id: ConnectionId,
    tx: mpsc::Sender<NetworkMessage>,
    rx: mpsc::Receiver<NetworkMessage>
}

#[derive(Debug)]
pub enum NetworkError {
    Error(String),
    NoMessagesToReceive,
    Failed
}

impl NetworkConnection {
    pub fn new(id: ConnectionId, tx: mpsc::Sender<NetworkMessage>, rx: mpsc::Receiver<NetworkMessage>) -> Self {
        NetworkConnection {
            id,
            tx,
            rx
        }
    }

    pub async fn send(&mut self, data: NetworkMessageData) -> Result<(), NetworkError> {
        let msg = NetworkMessage {
            sender: self.id,
            data
        };

        self.tx.send(msg).await.map_err(|_err| NetworkError::Failed)
    }

    pub async fn receive(&mut self) -> Result<NetworkMessage, NetworkError> {
        self.rx.try_recv().map_err(|_err| NetworkError::NoMessagesToReceive)
    }
}

#[derive(Default)]
pub struct Network {
    in_rxs: HashMap<Uuid, mpsc::Receiver<NetworkMessage>>,
    out_txs: HashMap<Uuid, mpsc::Sender<NetworkMessage>>
}

impl Network {
    pub fn new() -> Network {
        Network {
            ..Default::default()
        }
    }

    pub async fn connect(&mut self) -> NetworkConnection {
        let (in_tx, in_rx) = mpsc::channel::<NetworkMessage>(32);
        let (out_tx, out_rx) = mpsc::channel::<NetworkMessage>(32);

        let connection_id = Uuid::new_v4();

        self.in_rxs.insert(connection_id, in_rx);
        self.out_txs.insert(connection_id, out_tx);

        NetworkConnection::new(
            connection_id,
            in_tx,
            out_rx
        )
    }

    pub async fn work(&mut self) {
        //Remove closed channels
        self.in_rxs.retain(|_, in_rx| !in_rx.is_closed());
        self.out_txs.retain(|_, out_tx| !out_tx.is_closed());

        for (sender_uuid, in_rx) in self.in_rxs.iter_mut() {
            if let Ok(message) = in_rx.try_recv() {
                for (receiver_uuid, out_tx) in self.out_txs.iter_mut() {
                    if sender_uuid != receiver_uuid {
                        out_tx.send(message.clone()).await.unwrap();
                    }
                }
            }
        }
    }
}
