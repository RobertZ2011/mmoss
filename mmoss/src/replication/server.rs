use bevy::ecs::{
    entity::{Entity, EntityHashSet},
    world::World,
};
use log::trace;
use tokio::sync::Mutex;

use crate::{
    net::transport::Reliable,
    replication::{Message, Replicated, UpdateData},
};

struct Inner {
    /// All connected clients
    clients: Vec<Box<dyn Reliable<Message>>>,
}

impl Inner {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
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
        self.inner.lock().await.clients.push(client);
    }

    pub async fn mark_dirty(&self, entity: Entity) {
        self.dirty.lock().await.insert(entity);
    }

    pub async fn serialize(&self, world: &mut World) {
        let dirty = self.dirty.lock().await.drain().collect::<Vec<_>>();
        let mut inner = self.inner.lock().await;

        if !dirty.is_empty() {
            trace!("Serializing {} dirty entities", dirty.len());
        }

        let mut query = world.query::<&dyn Replicated>();
        for replicated in query.iter_many(world, dirty) {
            for component in replicated {
                let message = Message::Update(UpdateData {
                    id: component.id(),
                    data: {
                        let mut data = vec![0u8; 512];
                        let len = component.serialize(&mut data).unwrap_or(0);
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
