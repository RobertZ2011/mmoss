use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use anyhow::Result;
use bevy::ecs::{
    entity::{Entity, EntityHashMap},
    world::EntityRef,
};
use bevy_trait_query::All;
use log::{debug, error, trace};
use tokio::{sync::Mutex, task::yield_now};

use crate::{
    core::WorldContainer,
    net::transport::Unreliable,
    replication::{
        AddedComponentData, ComponentType, Id, Message, MobType, Replicated, SpawnData, SpawnId,
        UpdateData,
    },
};

pub mod factory;

use factory::component::Factory as ComponentFactory;
use factory::mob::Factory as MobFactory;

struct Pending {
    updates: HashMap<Id, UpdateData>,
    spawns: VecDeque<SpawnData>,
    added_component: HashMap<SpawnId, VecDeque<AddedComponentData>>,
}

impl Pending {
    pub fn new() -> Self {
        Self {
            updates: HashMap::new(),
            spawns: VecDeque::new(),
            added_component: HashMap::new(),
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
            Message::AddComponent(add_component) => {
                debug!("Received add component: {:?}", add_component);
                pending
                    .added_component
                    .entry(add_component.spawn_id)
                    .or_insert_with(VecDeque::new)
                    .push_back(add_component);
            }
        }
        yield_now().await;

        Ok(())
    }
}

pub trait UpdateCallbacks {
    fn on_component_updated(&mut self, entity: Entity, spawn_id: SpawnId, replicated_id: Id);
    fn on_spawn(&mut self, entity: Entity, spawn_id: SpawnId, mob_type: MobType);
    fn on_component_added(
        &mut self,
        entity: Entity,
        spawn_id: SpawnId,
        component_type: ComponentType,
        replicated_id: Id,
    );
}

pub struct NoopUpdateCallbacks;

impl UpdateCallbacks for NoopUpdateCallbacks {
    fn on_component_updated(&mut self, _entity: Entity, _spawn_id: SpawnId, _replicated_id: Id) {}
    fn on_spawn(&mut self, _entity: Entity, _spawn_id: SpawnId, _mob_type: MobType) {}
    fn on_component_added(
        &mut self,
        _entity: Entity,
        _spawn_id: SpawnId,
        _component_type: ComponentType,
        _replicated_id: Id,
    ) {
    }
}

pub struct Manager<W: WorldContainer> {
    pending: Arc<Mutex<Pending>>,
    mob_factory: Arc<MobFactory<W>>,
    component_factory: Arc<ComponentFactory<W>>,
    spawn_id_lookup: EntityHashMap<SpawnId>,
    entity_lookup: HashMap<SpawnId, Entity>,
}

impl<W: WorldContainer> Manager<W> {
    pub fn new(
        transport: Box<dyn Unreliable<Message>>,
        mob_factory: Arc<MobFactory<W>>,
        component_factory: Arc<ComponentFactory<W>>,
    ) -> (Self, Incoming) {
        let pending = Arc::new(Mutex::new(Pending::new()));
        (
            Self {
                pending: pending.clone(),
                mob_factory,
                component_factory,
                spawn_id_lookup: EntityHashMap::new(),
                entity_lookup: HashMap::new(),
            },
            Incoming { pending, transport },
        )
    }

    pub async fn update_world(&mut self, world: &mut W, callbacks: &mut impl UpdateCallbacks) {
        let mut pending = self.pending.lock().await;
        // Process spawns
        if pending.spawns.len() > 0 {
            trace!("Processing {} spawns", pending.spawns.len());
        }
        for spawn in pending.spawns.drain(..) {
            match self.mob_factory.construct(world, spawn.mob_type).await {
                Ok(entity) => {
                    self.spawn_id_lookup.insert(entity, spawn.spawn_id);
                    self.entity_lookup.insert(spawn.spawn_id, entity);
                    callbacks.on_spawn(entity, spawn.spawn_id, spawn.mob_type);
                }
                Err(e) => error!("Failed to spawn mob of type {:?}: {}", spawn.mob_type, e),
            }
        }

        // Process add component
        for (spawn_id, added_components) in pending.added_component.iter_mut() {
            let entity = self.entity_lookup.get(spawn_id);
            if entity.is_none() {
                error!("No entity found for spawn ID {:?}", spawn_id);
                added_components.clear();
                continue;
            }

            let entity = *entity.unwrap();
            for added_component in added_components.drain(..) {
                if let Err(e) = self
                    .component_factory
                    .add_component(
                        world,
                        entity,
                        added_component.component_type,
                        added_component.replicated_id,
                        &added_component.data,
                    )
                    .await
                {
                    error!(
                        "Failed to add component {:?} to entity {:?}: {}",
                        added_component.replicated_id,
                        entity.index(),
                        e
                    );
                } else {
                    callbacks.on_component_added(
                        entity,
                        *spawn_id,
                        added_component.component_type,
                        added_component.replicated_id,
                    );
                }
            }
        }

        // Process updates
        for (entity, components) in world
            .world_mut()
            .query::<(EntityRef, All<&mut dyn Replicated>)>()
            .iter_mut(world.world_mut())
        {
            for mut component in components {
                let id = component.id();
                if let Some(update) = pending.updates.remove(&id) {
                    if let Err(e) = component.replicate(&update.data) {
                        error!("Failed to replicate update for {:?}: {}", id, e);
                        continue;
                    }

                    if let Some(spawn_id) = self.spawn_id_lookup.get(&entity.id()) {
                        callbacks.on_component_updated(entity.id(), *spawn_id, id);
                    } else {
                        error!("No spawn ID found for entity {:?}", entity.id());
                    }
                }
            }
        }
    }
}
