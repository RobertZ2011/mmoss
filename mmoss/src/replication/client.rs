use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use anyhow::Result;
use async_trait::async_trait;
use bevy::ecs::entity::Entity;
use log::{debug, error, trace};
use tokio::{sync::Mutex, task::yield_now};

use crate::{
    core::WorldContainer,
    net::transport::Unreliable,
    replication::{ComponentType, Id, Message, MobType, Replicated, SpawnData, UpdateData},
};

struct Pending {
    updates: HashMap<Id, UpdateData>,
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
                trace!("Received update: {:?}", update);
                pending.updates.entry(update.id).or_insert(update);
            }
            Message::Spawn(spawn) => {
                debug!("Received spawn: {:?}", spawn);
                pending.spawns.push_back(spawn);
            }
        }
        yield_now().await;

        Ok(())
    }
}

pub struct Manager<'f, W: WorldContainer> {
    pending: Arc<Mutex<Pending>>,
    mob_factory: &'f Factory<W>,
}

impl<'f, W: WorldContainer> Manager<'f, W> {
    pub fn new(
        transport: Box<dyn Unreliable<Message>>,
        mob_factory: &'f Factory<W>,
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

    pub async fn update_world(&mut self, world: &mut W) {
        let mut pending = self.pending.lock().await;
        // Process spawns
        if pending.spawns.len() > 0 {
            trace!("Processing {} spawns", pending.spawns.len());
        }
        for spawn in pending.spawns.drain(..) {
            if let Err(e) = self
                .mob_factory
                .construct(world, spawn.mob_type, &spawn.replicated)
                .await
            {
                error!("Failed to spawn mob of type {:?}: {}", spawn.mob_type, e);
            }
        }

        // Process updates
        /*trace!(
            "Processing updates for {} components",
            pending.updates.len()
        );*/
        let mut components = world.world_mut().query::<&mut dyn Replicated>();
        for replicated in components.iter_mut(world.world_mut()) {
            for mut component in replicated {
                let id = component.id();
                if let Some(update) = pending.updates.remove(&id) {
                    component.replicate(&update.data).unwrap();
                }
            }
        }
    }
}

#[async_trait(?Send)]
pub trait FactoryEntry<W: WorldContainer> {
    async fn construct(
        &self,
        world: &mut W,
        replicated: &[(ComponentType, Id, Vec<u8>)],
    ) -> Result<Entity>;
}

pub struct Factory<W: WorldContainer> {
    prototypes: HashMap<MobType, Box<dyn FactoryEntry<W>>>,
}

impl<W: WorldContainer> Factory<W> {
    pub fn new() -> Self {
        Self {
            prototypes: HashMap::new(),
        }
    }

    pub fn register_mob(&mut self, mob_type: MobType, constructor: impl FactoryEntry<W> + 'static) {
        self.prototypes.insert(mob_type, Box::new(constructor));
    }

    pub async fn construct(
        &self,
        world: &mut W,
        mob_type: MobType,
        replicated: &Vec<(ComponentType, Id, Vec<u8>)>,
    ) -> Result<Entity> {
        let constructor = self.prototypes.get(&mob_type).ok_or_else(|| {
            anyhow::anyhow!("No prototype registered for mob type {:?}", mob_type)
        })?;
        constructor.construct(world, replicated).await
    }
}
