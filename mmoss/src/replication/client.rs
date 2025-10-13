use std::collections::{HashMap, VecDeque};

use anyhow::Result;
use bevy::prelude::World;
use tokio::sync::Mutex;

use crate::{
    net::transport::Unreliable,
    replication::{Id, Message, Replicated, SpawnData, UpdateData},
};

struct Pending {
    updates: HashMap<Id, VecDeque<UpdateData>>,
    spawns: HashMap<Id, VecDeque<SpawnData>>,
}

impl Pending {
    pub fn new() -> Self {
        Self {
            updates: HashMap::new(),
            spawns: HashMap::new(),
        }
    }
}

pub struct Manager {
    transport: Mutex<Box<dyn Unreliable<Message>>>,
    pending: Mutex<Pending>,
}

unsafe impl Send for Manager {}

impl Manager {
    pub fn new(transport: Box<dyn Unreliable<Message>>) -> Self {
        Self {
            transport: Mutex::new(transport),
            pending: Mutex::new(Pending::new()),
        }
    }

    pub async fn process_incoming(&self) -> Result<()> {
        let message = self.transport.lock().await.receive().await?;
        let mut pending = self.pending.lock().await;
        match message {
            Message::Update(update) => {
                pending
                    .updates
                    .entry(update.id)
                    .or_default()
                    .push_back(update);
            }
            Message::Spawn(spawn) => {
                pending.spawns.entry(spawn.id).or_default().push_back(spawn);
            }
        }

        Ok(())
    }

    pub async fn update_world(&self, world: &mut World) {
        // Process updates
        let mut components = world.query::<&mut dyn Replicated>();
        for replicated in components.iter_mut(world) {
            for mut component in replicated {
                let id = component.id();
                if let Some(updates) = self
                    .pending
                    .lock()
                    .await
                    .updates
                    .get_mut(&id)
                    .map(|v| v.drain(..).collect::<Vec<_>>())
                {
                    if updates.is_empty() {
                        continue;
                    }

                    for update in updates {
                        component.replicate(&update.data).unwrap();
                    }
                }
            }
        }
    }
}
