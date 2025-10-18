use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use anyhow::Result;
use bevy::{ecs::entity::Entity, prelude::World};
use log::{error, trace};
use tokio::sync::Mutex;

use crate::{
    net::transport::Unreliable,
    replication::{Id, Message, MobType, Replicated, SpawnData, UpdateData},
};

struct Pending {
    updates: HashMap<Id, VecDeque<UpdateData>>,
    spawns: VecDeque<SpawnData>,
}

impl Pending {
    pub fn new() -> Self {
        Self {
            updates: HashMap::new(),
            spawns: VecDeque::new(),
        }
    }
}

pub struct Incoming {
    transport: Box<dyn Unreliable<Message>>,
    pending: Arc<Mutex<Pending>>,
}

impl Incoming {
    pub async fn process_incoming(&mut self) -> Result<()> {
        let message = self.transport.receive().await?;
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
                trace!("Received spawn: {:?}", spawn);
                pending.spawns.push_back(spawn);
            }
        }

        Ok(())
    }
}

pub struct Manager<'f> {
    pending: Arc<Mutex<Pending>>,
    mob_factory: &'f Factory,
}

impl<'f> Manager<'f> {
    pub fn new(
        transport: Box<dyn Unreliable<Message>>,
        mob_factory: &'f Factory,
    ) -> (Self, Incoming) {
        let pending = Arc::new(Mutex::new(Pending::new()));
        (
            Self {
                pending: pending.clone(),
                mob_factory,
            },
            Incoming { pending, transport },
        )
    }

    pub async fn update_world(&mut self, world: &mut World) {
        // Process spawns
        let spawns = self
            .pending
            .lock()
            .await
            .spawns
            .drain(..)
            .collect::<Vec<_>>();
        for spawn in spawns {
            if let Err(e) = self
                .mob_factory
                .construct(world, spawn.mob_type, &spawn.replicated)
            {
                error!("Failed to spawn mob of type {:?}: {}", spawn.mob_type, e);
            }
        }

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

pub struct Factory {
    prototypes:
        HashMap<MobType, Box<dyn Fn(&mut World, &[(usize, Id, Vec<u8>)]) -> Result<Entity> + Sync>>,
}

impl Factory {
    pub fn new() -> Self {
        Self {
            prototypes: HashMap::new(),
        }
    }

    pub fn register_mob<F>(&mut self, mob_type: MobType, constructor: F)
    where
        F: 'static + Fn(&mut World, &[(usize, Id, Vec<u8>)]) -> Result<Entity> + Sync,
    {
        self.prototypes.insert(mob_type, Box::new(constructor));
    }

    pub fn construct(
        &self,
        world: &mut World,
        mob_type: MobType,
        replicated: &Vec<(usize, Id, Vec<u8>)>,
    ) -> Result<Entity> {
        let constructor = self.prototypes.get(&mob_type).ok_or_else(|| {
            anyhow::anyhow!("No prototype registered for mob type {:?}", mob_type)
        })?;
        constructor(world, replicated)
    }
}
