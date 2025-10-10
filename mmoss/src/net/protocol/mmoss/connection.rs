use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

use crate::net::transport::{Addressed, Message, Unreliable};

struct ActiveConnection<M: Message> {
    _sender: Sender<M>,
}

struct PendingConnections<M: Message> {
    _sender: Sender<Connection<M>>,
    receiver: Mutex<Receiver<Connection<M>>>,
}

impl<M: Message> PendingConnections<M> {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(128);
        Self {
            _sender: sender,
            receiver: Mutex::new(receiver),
        }
    }
}

struct Inner<A, M: Message, T: Unreliable<Addressed<A, M>>> {
    transport: T,
    _active_connections: HashMap<A, ActiveConnection<M>>,
    _incoming: Receiver<M>,
    _outgoing: Sender<M>,
}

pub struct ConnectionManager<A, M: Message, T: Unreliable<Addressed<A, M>>> {
    transport: Mutex<Inner<A, M, T>>,
    pending_connections: PendingConnections<M>,
}

impl<A: std::hash::Hash + Eq + Clone, M: Message, T: Unreliable<Addressed<A, M>>>
    ConnectionManager<A, M, T>
{
    pub fn new(transport: T) -> Self {
        let (outgoing, incoming) = tokio::sync::mpsc::channel(128);
        Self {
            transport: Mutex::new(Inner {
                transport,
                _incoming: incoming,
                _outgoing: outgoing,
                _active_connections: HashMap::new(),
            }),
            pending_connections: PendingConnections::new(),
        }
    }

    pub async fn process_all(&self) {
        let mut inner = self.transport.lock().await;
        loop {
            if let Ok(_addressed) = inner.transport.try_receive() {}
        }
    }

    pub async fn accept(&self) -> Result<Connection<M>> {
        let mut receiver = self.pending_connections.receiver.lock().await;
        Ok(receiver
            .recv()
            .await
            .ok_or(anyhow::anyhow!("Failed to accept connection"))?)
    }
}

pub struct Connection<M: Message> {
    receiver: Receiver<M>,
    sender: Sender<M>,
}

#[async_trait(?Send)]
impl<M: Message> Unreliable<M> for Connection<M> {
    async fn send(&mut self, message: M) -> Result<()> {
        Ok(self
            .sender
            .send(message)
            .await
            .map_err(|_| anyhow::anyhow!("Failed to send message"))?)
    }

    async fn receive(&mut self) -> Result<M> {
        Ok(self
            .receiver
            .recv()
            .await
            .ok_or(anyhow::anyhow!("Failed to receive message"))?)
    }

    fn try_receive(&mut self) -> Result<Option<M>> {
        Ok(self.receiver.try_recv().ok())
    }
}
