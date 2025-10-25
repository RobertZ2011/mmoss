use std::mem;

use bevy::ecs::{
    entity::{Entity, EntityHashMap, EntityHashSet},
    world::{EntityRef, World},
};
use bevy_trait_query::{All, ReadTraits};
use log::{debug, error, trace};

use crate::{
    net::transport::Reliable,
    replication::{
        AddedComponentData, Message, MobType, Replicated, SpawnData, SpawnId, UpdateData,
    },
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
    /// Map from entity to spawn ID
    entity_spawn_ids: EntityHashMap<SpawnId>,
    /// Next spawn ID to use
    next_spawn_id: u32,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
            pending_full_sync: Vec::new(),
            newly_spawned: EntityHashSet::new(),
            dirty: EntityHashSet::new(),
            entity_spawn_ids: EntityHashMap::new(),
            next_spawn_id: 0,
        }
    }

    pub fn add_client(&mut self, client: Box<dyn Reliable<Message>>) {
        self.pending_full_sync.push(client);
    }

    pub fn mark_dirty(&mut self, entity: Entity) {
        self.dirty.insert(entity);
    }

    pub fn register_new_entity(&mut self, entity: Entity) {
        let spawn_id = SpawnId(self.next_spawn_id);
        self.next_spawn_id += 1;
        self.entity_spawn_ids.insert(entity, spawn_id);
        self.newly_spawned.insert(entity);
    }

    async fn serialize_spawned<'a>(
        &mut self,
        new_only: bool,
        iter: impl Iterator<Item = (EntityRef<'a>, &'a MobType, ReadTraits<'a, dyn Replicated>)>,
    ) {
        for (entity, mob_type, components) in iter {
            let spawn_id = self.entity_spawn_ids.get(&entity.id());
            if spawn_id.is_none() {
                error!("No spawn ID found for entity {:?}", entity.id());
                continue;
            }
            let spawn_id = *spawn_id.unwrap();

            let message = Message::Spawn(SpawnData {
                mob_type: *mob_type,
                spawn_id,
            });

            let clients = if new_only {
                &mut self.pending_full_sync
            } else {
                &mut self.clients
            };
            for client in clients.iter_mut() {
                if let Err(e) = client.send(&message).await {
                    error!("Failed to send spawn message: {}", e);
                    continue;
                }
            }

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

                trace!(
                    "Add serialized component {:?}: {} bytes",
                    comp.id(),
                    data.len()
                );

                let message = Message::AddComponent(AddedComponentData {
                    component_type: comp.replicated_component_type(),
                    spawn_id,
                    replicated_id: comp.id(),
                    data,
                });

                for client in clients.iter_mut() {
                    if let Err(e) = client.send(&message).await {
                        error!("Failed to send add component message: {}", e);
                        continue;
                    }
                }
            }
        }
    }

    pub async fn serialize(&mut self, world: &mut World) {
        if !self.dirty.is_empty() {
            trace!("Dirty entities: {:?}", self.dirty.len());
        }

        let mut query = world.query::<&dyn Replicated>();
        let mut num_replicated = 0;
        for replicated in query.iter_many(world, &self.dirty) {
            for component in replicated {
                num_replicated += 1;
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

                trace!("Replicating message {:?}", message);
                for client in &mut self.clients {
                    client.send(&message).await.unwrap();
                }
            }
        }
        debug!("Replicated {} components", num_replicated);
        self.dirty.clear();

        // Next, handle any newly spawned entities
        if !self.newly_spawned.is_empty() {
            trace!("Newly spawned entities: {:?}", self.newly_spawned.len());
            let mut query = world.query::<(EntityRef, &MobType, All<&dyn Replicated>)>();
            let entities = mem::replace(&mut self.newly_spawned, EntityHashSet::new());
            self.serialize_spawned(false, query.iter_many(world, entities))
                .await;
            self.newly_spawned.clear();
        }

        // Lastly, handle any clients that are pending their first full state sync
        if !self.pending_full_sync.is_empty() {
            trace!(
                "Clients pending full sync: {}",
                self.pending_full_sync.len()
            );
            let mut query = world.query::<(EntityRef, &MobType, All<&dyn Replicated>)>();
            self.serialize_spawned(true, query.iter(world)).await;

            let mut drained = self.pending_full_sync.drain(..).collect::<Vec<_>>();
            self.clients.append(&mut drained);
        }
    }
}
