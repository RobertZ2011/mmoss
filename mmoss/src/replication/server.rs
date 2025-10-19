use std::mem;

use bevy::ecs::{
    entity::{Entity, EntityHashSet},
    world::World,
};
use bevy_trait_query::{All, ReadTraits};
use log::{error, trace};

use crate::{
    net::transport::Reliable,
    replication::{Message, MobType, Replicated, SpawnData, UpdateData},
};

pub struct Manager {
    /// All connected clients
    clients: Vec<Box<dyn Reliable<Message>>>,
    /// All connected clients that are pending their first full state sync
    pending_full_sync: Vec<Box<dyn Reliable<Message>>>,
    /// Newly spawned entities that need to be sent to clients
    newly_spawned: EntityHashSet,
    /// All objects that have changed since the last update
    dirty: EntityHashSet,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
            pending_full_sync: Vec::new(),
            newly_spawned: EntityHashSet::new(),
            dirty: EntityHashSet::new(),
        }
    }

    pub fn add_client(&mut self, client: Box<dyn Reliable<Message>>) {
        self.pending_full_sync.push(client);
    }

    pub fn mark_dirty(&mut self, entity: Entity) {
        self.dirty.insert(entity);
    }

    pub fn register_new_entity(&mut self, entity: Entity) {
        self.newly_spawned.insert(entity);
    }

    async fn serialize_spawned<'a>(
        clients: &mut [Box<dyn Reliable<Message>>],
        iter: impl Iterator<Item = (&'a MobType, ReadTraits<'a, dyn Replicated>)>,
    ) {
        for (mob_type, components) in iter {
            let mut replicated = Vec::new();
            for comp in components {
                let mut data = vec![0u8; 512];

                let result = comp.serialize(&mut data);
                if result.is_err() {
                    error!(
                        "Failed to serialize spawn {:?}: {}",
                        comp.id(),
                        result.unwrap_err()
                    );
                    continue;
                }
                let len = result.unwrap();
                data.truncate(len);
                trace!("Serialized component {:?}: {} bytes", comp.id(), data.len());
                replicated.push((comp.replicated_component_type(), comp.id(), data));
            }

            let message = Message::Spawn(SpawnData {
                mob_type: *mob_type,
                replicated,
            });

            trace!(
                "Sending spawn message to {} clients, {:?}",
                clients.len(),
                message
            );
            for client in &mut *clients {
                if let Err(e) = client.send(&message).await {
                    error!("Failed to send spawn message: {}", e);
                    continue;
                }
            }
        }
    }

    pub async fn serialize(&mut self, world: &mut World) {
        if !self.dirty.is_empty() {
            trace!("Dirty entities: {:?}", self.dirty.len());
        }

        let mut query = world.query::<&dyn Replicated>();
        for replicated in query.iter_many(world, &self.dirty) {
            for component in replicated {
                let message = Message::Update(UpdateData {
                    id: component.id(),
                    data: {
                        let mut data = vec![0u8; 512];
                        let result = component.serialize(&mut data);
                        if result.is_err() {
                            error!(
                                "Failed to serialize update {:?}: {}",
                                component.id(),
                                result.unwrap_err()
                            );
                            continue;
                        }
                        let len = result.unwrap();
                        data.truncate(len);
                        data
                    },
                });

                trace!("Replicating message to {} clients", self.clients.len());
                for client in &mut self.clients {
                    client.send(&message).await.unwrap();
                }
            }
        }
        self.dirty.clear();

        // Next, handle any newly spawned entities
        if !self.newly_spawned.is_empty() {
            trace!("Newly spawned entities: {:?}", self.newly_spawned.len());
            let mut query = world.query::<(&MobType, All<&dyn Replicated>)>();
            let entities = mem::replace(&mut self.newly_spawned, EntityHashSet::new());
            Self::serialize_spawned(&mut self.clients, query.iter_many(world, entities)).await;
            self.newly_spawned.clear();
        }

        // Lastly, handle any clients that are pending their first full state sync
        if !self.pending_full_sync.is_empty() {
            trace!(
                "Clients pending full sync: {}",
                self.pending_full_sync.len()
            );
            let mut query = world.query::<(&MobType, All<&dyn Replicated>)>();
            Self::serialize_spawned(&mut self.pending_full_sync, query.iter(world)).await;

            let mut drained = self.pending_full_sync.drain(..).collect::<Vec<_>>();
            self.clients.append(&mut drained);
        }
    }
}
