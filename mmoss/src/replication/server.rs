use bevy::ecs::{
    entity::{Entity, EntityHashSet},
    world::World,
};
use bevy_trait_query::All;
use log::{error, trace};
use tokio::sync::Mutex;

use crate::{
    net::transport::Reliable,
    replication::{Message, MobType, Replicated, SpawnData, UpdateData},
};

struct Inner {
    /// All connected clients
    clients: Vec<Box<dyn Reliable<Message>>>,
    /// All connected clients that are pending their first full state sync
    pending_full_sync: Vec<Box<dyn Reliable<Message>>>,
}

impl Inner {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
            pending_full_sync: Vec::new(),
        }
    }
}

pub struct Manager {
    inner: Mutex<Inner>,
    /// All objects that have changed since the last update
    dirty: Mutex<EntityHashSet>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::new()),
            dirty: Mutex::new(EntityHashSet::new()),
        }
    }

    pub async fn add_client(&mut self, client: Box<dyn Reliable<Message>>) {
        self.inner.lock().await.pending_full_sync.push(client);
    }

    pub async fn mark_dirty(&self, entity: Entity) {
        self.dirty.lock().await.insert(entity);
    }

    pub async fn serialize(&self, world: &mut World) {
        let mut inner = self.inner.lock().await;
        // First, handle any clients that are pending their first full state sync
        if !inner.pending_full_sync.is_empty() {
            let mut query = world.query::<(&MobType, All<&dyn Replicated>)>();
            for (mob_type, component) in query.iter(world) {
                // Serialize the current state of all replicated components
                let mut replicated = Vec::new();
                for comp in component {
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
                    replicated.push((comp.id(), data));
                }

                let message = Message::Spawn(SpawnData {
                    mob_type: *mob_type,
                    replicated,
                });

                trace!(
                    "Sending spawn message to {} clients, {:#?}",
                    inner.pending_full_sync.len(),
                    message
                );
                let clients = inner.pending_full_sync.drain(..).collect::<Vec<_>>();
                for mut client in clients {
                    if let Err(e) = client.send(&message).await {
                        error!("Failed to send spawn message: {}", e);
                        continue;
                    }
                    inner.clients.push(client);
                }
            }
        }

        let dirty = self.dirty.lock().await.drain().collect::<Vec<_>>();
        if !dirty.is_empty() {
            trace!("Dirty entities: {:?}", dirty.len());
        }

        let mut query = world.query::<&dyn Replicated>();
        for replicated in query.iter_many(world, dirty) {
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

                trace!("Replicating message to {} clients", inner.clients.len());
                for client in &mut inner.clients {
                    client.send(&message).await.unwrap();
                }
            }
        }
    }
}
