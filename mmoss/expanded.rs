#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
pub mod core {
    pub mod mob {
        use bevy::ecs::component::Component;
        use mmoss_proc_macros::Replicated;
        use crate::replication::{self, Id, MobType};
        extern crate self as mmoss;
        pub struct MobComponent {
            #[replication_id]
            pub id: Id,
            pub mob_type: MobType,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for MobComponent {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "MobComponent",
                    "id",
                    &self.id,
                    "mob_type",
                    &&self.mob_type,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for MobComponent {
            #[inline]
            fn clone(&self) -> MobComponent {
                MobComponent {
                    id: ::core::clone::Clone::clone(&self.id),
                    mob_type: ::core::clone::Clone::clone(&self.mob_type),
                }
            }
        }
        impl ::bevy::ecs::component::Component for MobComponent
        where
            Self: Send + Sync + 'static,
        {
            const STORAGE_TYPE: ::bevy::ecs::component::StorageType = ::bevy::ecs::component::StorageType::Table;
            type Mutability = ::bevy::ecs::component::Mutable;
            fn register_required_components(
                _requiree: ::bevy::ecs::component::ComponentId,
                required_components: &mut ::bevy::ecs::component::RequiredComponentsRegistrator,
            ) {}
            fn clone_behavior() -> ::bevy::ecs::component::ComponentCloneBehavior {
                use ::bevy::ecs::component::{
                    DefaultCloneBehaviorBase, DefaultCloneBehaviorViaClone,
                };
                (&&&::bevy::ecs::component::DefaultCloneBehaviorSpecialization::<
                    Self,
                >::default())
                    .default_clone_behavior()
            }
        }
        impl replication::Replicated for MobComponent {
            fn id(&self) -> replication::Id {
                self.id
            }
            fn serialize(&self, data: &mut [u8]) -> ::anyhow::Result<usize> {
                let mut cursor = 0;
                Ok(cursor)
            }
            fn replicate(&mut self, data: &[u8]) -> ::anyhow::Result<usize> {
                let mut cursor = 0;
                Ok(cursor)
            }
        }
        impl MobComponent {
            pub fn new(id: Id, mob_type: MobType) -> Self {
                Self { id, mob_type }
            }
        }
    }
}
pub mod net {
    pub mod protocol {
        pub mod mmoss {}
    }
    pub mod transport {
        use anyhow::Result;
        use async_trait::async_trait;
        use std::marker::PhantomData;
        pub mod tcp {
            use std::{marker::PhantomData, net::SocketAddr};
            use anyhow::Result;
            use async_trait::async_trait;
            use tokio::{
                io::{AsyncReadExt, AsyncWriteExt},
                net::{TcpListener, TcpStream, ToSocketAddrs},
            };
            use crate::net::transport::{Message, MessageFactory, Reliable, Unreliable};
            pub struct Listener<M: Message> {
                listener: TcpListener,
                _marker: PhantomData<M>,
            }
            impl<M: Message> Listener<M> {
                pub async fn bind(addr: impl ToSocketAddrs) -> Result<Self> {
                    Ok(Self {
                        listener: TcpListener::bind(addr).await?,
                        _marker: PhantomData,
                    })
                }
                pub async fn accept<F: MessageFactory<Message = M>>(
                    &self,
                    factory: F,
                ) -> Result<(Connection<F>, SocketAddr)> {
                    let (stream, addr) = self.listener.accept().await?;
                    Ok((Connection { stream, factory }, addr))
                }
            }
            pub struct Connection<F: MessageFactory> {
                stream: TcpStream,
                factory: F,
            }
            impl<F: MessageFactory> Connection<F> {
                pub async fn connect(
                    addr: impl ToSocketAddrs,
                    factory: F,
                ) -> Result<Self> {
                    let stream = TcpStream::connect(addr).await?;
                    Ok(Self { stream, factory })
                }
            }
            const BUFFER_SIZE: usize = 512;
            impl<F: MessageFactory> Unreliable<F::Message> for Connection<F> {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn send<'life0, 'life1, 'async_trait>(
                    &'life0 mut self,
                    message: &'life1 F::Message,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Result<()>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Result<()>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let mut __self = self;
                        let __ret: Result<()> = {
                            let mut buffer = [0u8; BUFFER_SIZE];
                            let len = message.serialize(&mut buffer)?;
                            __self.stream.write(&buffer[..len]).await?;
                            Ok(())
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn receive<'life0, 'async_trait>(
                    &'life0 mut self,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Result<F::Message>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Result<F::Message>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let mut __self = self;
                        let __ret: Result<F::Message> = {
                            let mut buffer = [0u8; BUFFER_SIZE];
                            let len = __self.stream.read(&mut buffer).await?;
                            let (message, _) = __self
                                .factory
                                .deserialize(&(), &buffer[..len])?;
                            Ok(message)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                fn try_receive(&mut self) -> Result<Option<F::Message>> {
                    let mut buffer = [0u8; BUFFER_SIZE];
                    match self.stream.try_read(&mut buffer) {
                        Ok(len) if len > 0 => {
                            let (message, _) = self
                                .factory
                                .deserialize(&(), &buffer[..len])?;
                            Ok(Some(message))
                        }
                        Ok(_) => Ok(None),
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            Ok(None)
                        }
                        Err(e) => Err(e.into()),
                    }
                }
            }
            impl<F: MessageFactory> Reliable<F::Message> for Connection<F> {}
        }
        pub mod udp {
            use std::net::SocketAddr;
            use crate::net::transport::{
                Addressed, AddressedFactory, Message, MessageFactory, Unreliable,
            };
            use anyhow::Result;
            use async_trait::async_trait;
            use tokio::net::{ToSocketAddrs, UdpSocket};
            pub struct Udp<F: MessageFactory> {
                socket: UdpSocket,
                factory: AddressedFactory<SocketAddr, F>,
            }
            impl<F: MessageFactory> Udp<F> {
                pub async fn bind<A: ToSocketAddrs>(
                    addr: A,
                    factory: F,
                ) -> Result<Self> {
                    Ok(Self {
                        socket: UdpSocket::bind(addr).await?,
                        factory: AddressedFactory::new(factory),
                    })
                }
            }
            impl<F: MessageFactory> Unreliable<Addressed<SocketAddr, F::Message>>
            for Udp<F> {
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn send<'life0, 'life1, 'async_trait>(
                    &'life0 mut self,
                    message: &'life1 Addressed<SocketAddr, F::Message>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Result<()>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Result<()>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let mut __self = self;
                        let __ret: Result<()> = {
                            let mut buffer = [0u8; 512];
                            let len = message.serialize(&mut buffer)?;
                            __self
                                .socket
                                .send_to(&buffer[..len], message.address)
                                .await?;
                            Ok(())
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                #[allow(
                    elided_named_lifetimes,
                    clippy::async_yields_async,
                    clippy::diverging_sub_expression,
                    clippy::let_unit_value,
                    clippy::needless_arbitrary_self_type,
                    clippy::no_effect_underscore_binding,
                    clippy::shadow_same,
                    clippy::type_complexity,
                    clippy::type_repetition_in_bounds,
                    clippy::used_underscore_binding
                )]
                fn receive<'life0, 'async_trait>(
                    &'life0 mut self,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                            Output = Result<Addressed<SocketAddr, F::Message>>,
                        > + ::core::marker::Send + 'async_trait,
                    >,
                >
                where
                    'life0: 'async_trait,
                    Self: 'async_trait,
                {
                    Box::pin(async move {
                        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                            Result<Addressed<SocketAddr, F::Message>>,
                        > {
                            #[allow(unreachable_code)] return __ret;
                        }
                        let mut __self = self;
                        let __ret: Result<Addressed<SocketAddr, F::Message>> = {
                            let mut buffer = [0u8; 512];
                            let (len, addr) = __self
                                .socket
                                .recv_from(&mut buffer)
                                .await?;
                            let (message, _) = __self
                                .factory
                                .deserialize(&addr, &buffer[..len])?;
                            Ok(message)
                        };
                        #[allow(unreachable_code)] __ret
                    })
                }
                fn try_receive(
                    &mut self,
                ) -> Result<Option<Addressed<SocketAddr, F::Message>>> {
                    let mut buffer = [0u8; 512];
                    match self.socket.try_recv_from(&mut buffer) {
                        Ok((len, addr)) => {
                            let (message, _) = self
                                .factory
                                .deserialize(&addr, &buffer[..len])?;
                            Ok(Some(message))
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            Ok(None)
                        }
                        Err(e) => Err(e.into()),
                    }
                }
            }
        }
        /// Message trait for serialization
        pub trait Message: Send + Sync {
            fn serialize(&self, data: &mut [u8]) -> Result<usize>;
        }
        /// Message factory trait for deserialization
        pub trait MessageFactory<Context = ()>: Send {
            type Message: Message;
            fn deserialize(
                &self,
                context: &Context,
                data: &[u8],
            ) -> Result<(Self::Message, usize)>;
        }
        /// Unreliable transport for sending and receiving messages of type `M`
        pub trait Unreliable<M: Message>: Send {
            #[must_use]
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn send<'life0, 'life1, 'async_trait>(
                &'life0 mut self,
                message: &'life1 M,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = Result<()>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                Self: 'async_trait;
            #[must_use]
            #[allow(
                elided_named_lifetimes,
                clippy::type_complexity,
                clippy::type_repetition_in_bounds
            )]
            fn receive<'life0, 'async_trait>(
                &'life0 mut self,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<
                        Output = Result<M>,
                    > + ::core::marker::Send + 'async_trait,
                >,
            >
            where
                'life0: 'async_trait,
                Self: 'async_trait;
            fn try_receive(&mut self) -> Result<Option<M>>;
        }
        /// Reliable transport for sending and receiving messages of type `M`
        pub trait Reliable<M: Message>: Unreliable<M> {}
        pub struct Addressed<A: Send, M: Message> {
            pub address: A,
            pub message: M,
        }
        impl<A: Send + Sync, M: Message> Addressed<A, M> {
            pub fn new(address: A, message: M) -> Self {
                Self { address, message }
            }
        }
        impl<A: Send + Sync, M: Message> Message for Addressed<A, M> {
            fn serialize(&self, data: &mut [u8]) -> Result<usize> {
                self.message.serialize(data)
            }
        }
        pub struct AddressedFactory<A, F: MessageFactory> {
            pub factory: F,
            _marker: PhantomData<A>,
        }
        impl<A, F: MessageFactory> AddressedFactory<A, F> {
            pub fn new(factory: F) -> Self {
                Self {
                    factory,
                    _marker: PhantomData,
                }
            }
        }
        impl<A: Clone + Send + Sync, F: MessageFactory> MessageFactory<A>
        for AddressedFactory<A, F> {
            type Message = Addressed<A, F::Message>;
            fn deserialize(
                &self,
                context: &A,
                data: &[u8],
            ) -> Result<(Self::Message, usize)> {
                let (message, len) = self.factory.deserialize(&(), data)?;
                Ok((
                    Addressed {
                        address: context.clone(),
                        message,
                    },
                    len,
                ))
            }
        }
        impl Message for String {
            fn serialize(&self, data: &mut [u8]) -> Result<usize> {
                Ok(bincode::encode_into_slice(self, data, bincode::config::standard())?)
            }
        }
        /// String factory that creates a new string every time
        pub struct StringFactoryNew;
        impl MessageFactory for StringFactoryNew {
            type Message = String;
            fn deserialize(
                &self,
                _context: &(),
                data: &[u8],
            ) -> Result<(Self::Message, usize)> {
                let (message, len): (Self::Message, _) = bincode::decode_from_slice(
                    data,
                    bincode::config::standard(),
                )?;
                Ok((message, len))
            }
        }
        impl Message for Vec<u8> {
            fn serialize(&self, data: &mut [u8]) -> Result<usize> {
                Ok(bincode::encode_into_slice(self, data, bincode::config::standard())?)
            }
        }
        pub struct VecU8FactoryNew;
        impl MessageFactory for VecU8FactoryNew {
            type Message = Vec<u8>;
            fn deserialize(
                &self,
                _context: &(),
                data: &[u8],
            ) -> Result<(Self::Message, usize)> {
                let (message, len): (Self::Message, _) = bincode::decode_from_slice(
                    data,
                    bincode::config::standard(),
                )?;
                Ok((message, len))
            }
        }
    }
}
pub mod replication {
    use anyhow::Result;
    use bevy_trait_query::queryable;
    use bincode::{Decode, Encode};
    use mmoss_proc_macros::Replicated;
    use crate::net::transport::{
        Message as MessageTrait, MessageFactory as MessageFactoryTrait,
    };
    pub mod client {
        use std::collections::{HashMap, VecDeque};
        use anyhow::Result;
        use bevy::prelude::World;
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
        pub struct Manager<'f> {
            transport: Mutex<Box<dyn Unreliable<Message>>>,
            pending: Mutex<Pending>,
            mob_factory: &'f Factory,
        }
        impl<'f> Manager<'f> {
            pub fn new(
                transport: Box<dyn Unreliable<Message>>,
                mob_factory: &'f Factory,
            ) -> Self {
                Self {
                    transport: Mutex::new(transport),
                    pending: Mutex::new(Pending::new()),
                    mob_factory,
                }
            }
            pub async fn process_incoming(&self) -> Result<()> {
                let message = self.transport.lock().await.receive().await?;
                let mut pending = self.pending.lock().await;
                match message {
                    Message::Update(update) => {
                        pending.updates.entry(update.id).or_default().push_back(update);
                    }
                    Message::Spawn(spawn) => {
                        {
                            {
                                let lvl = ::log::Level::Trace;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        { ::log::__private_api::GlobalLogger },
                                        format_args!("Received spawn: {0:?}", spawn),
                                        lvl,
                                        &(
                                            "mmoss::replication::client",
                                            "mmoss::replication::client",
                                            ::log::__private_api::loc(),
                                        ),
                                        (),
                                    );
                                }
                            }
                        };
                        pending.spawns.push_back(spawn);
                    }
                }
                Ok(())
            }
            pub async fn update_world(&self, world: &mut World) {
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
                        {
                            {
                                let lvl = ::log::Level::Error;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        { ::log::__private_api::GlobalLogger },
                                        format_args!(
                                            "Failed to spawn mob of type {0:?}: {1}",
                                            spawn.mob_type,
                                            e,
                                        ),
                                        lvl,
                                        &(
                                            "mmoss::replication::client",
                                            "mmoss::replication::client",
                                            ::log::__private_api::loc(),
                                        ),
                                        (),
                                    );
                                }
                            }
                        };
                    }
                }
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
            prototypes: HashMap<
                MobType,
                Box<dyn Fn(&mut World, &[(Id, Vec<u8>)]) -> Result<()> + Sync>,
            >,
        }
        impl Factory {
            pub fn new() -> Self {
                Self { prototypes: HashMap::new() }
            }
            pub fn register<F>(&mut self, mob_type: MobType, constructor: F)
            where
                F: 'static + Fn(&mut World, &[(Id, Vec<u8>)]) -> Result<()> + Sync,
            {
                self.prototypes.insert(mob_type, Box::new(constructor));
            }
            pub fn construct(
                &self,
                world: &mut World,
                mob_type: MobType,
                replicated: &Vec<(Id, Vec<u8>)>,
            ) -> Result<()> {
                let constructor = self
                    .prototypes
                    .get(&mob_type)
                    .ok_or_else(|| {
                        ::anyhow::Error::msg(
                            ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "No prototype registered for mob type {0:?}",
                                        mob_type,
                                    ),
                                )
                            }),
                        )
                    })?;
                constructor(world, replicated)
            }
        }
    }
    pub mod server {
        use std::mem;
        use bevy::ecs::{
            entity::{Entity, EntityHashSet},
            world::World,
        };
        use bevy_trait_query::{All, ReadTraits};
        use log::{error, trace};
        use tokio::sync::Mutex;
        use crate::{
            core::mob::MobComponent, net::transport::Reliable,
            replication::{Message, Replicated, SpawnData, UpdateData},
        };
        struct Inner {
            /// All connected clients
            clients: Vec<Box<dyn Reliable<Message>>>,
            /// All connected clients that are pending their first full state sync
            pending_full_sync: Vec<Box<dyn Reliable<Message>>>,
            /// Newly spawned entities that need to be sent to clients
            newly_spawned: EntityHashSet,
        }
        impl Inner {
            pub fn new() -> Self {
                Self {
                    clients: Vec::new(),
                    pending_full_sync: Vec::new(),
                    newly_spawned: EntityHashSet::new(),
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
            pub async fn register_new_entity(&self, entity: Entity) {
                self.inner.lock().await.newly_spawned.insert(entity);
            }
            async fn serialize_spawned<'a>(
                clients: &mut [Box<dyn Reliable<Message>>],
                iter: impl Iterator<
                    Item = (&'a MobComponent, ReadTraits<'a, dyn Replicated>),
                >,
            ) {
                for (mob, components) in iter {
                    let mut replicated = Vec::new();
                    for comp in components {
                        let mut data = ::alloc::vec::from_elem(0u8, 512);
                        let result = comp.serialize(&mut data);
                        if result.is_err() {
                            {
                                {
                                    let lvl = ::log::Level::Error;
                                    if lvl <= ::log::STATIC_MAX_LEVEL
                                        && lvl <= ::log::max_level()
                                    {
                                        ::log::__private_api::log(
                                            { ::log::__private_api::GlobalLogger },
                                            format_args!(
                                                "Failed to serialize spawn {0:?}: {1}",
                                                comp.id(),
                                                result.unwrap_err(),
                                            ),
                                            lvl,
                                            &(
                                                "mmoss::replication::server",
                                                "mmoss::replication::server",
                                                ::log::__private_api::loc(),
                                            ),
                                            (),
                                        );
                                    }
                                }
                            };
                            continue;
                        }
                        let len = result.unwrap();
                        data.truncate(len);
                        {
                            {
                                let lvl = ::log::Level::Trace;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        { ::log::__private_api::GlobalLogger },
                                        format_args!(
                                            "Serialized component {0:?}: {1} bytes",
                                            comp.id(),
                                            data.len(),
                                        ),
                                        lvl,
                                        &(
                                            "mmoss::replication::server",
                                            "mmoss::replication::server",
                                            ::log::__private_api::loc(),
                                        ),
                                        (),
                                    );
                                }
                            }
                        };
                        replicated.push((comp.id(), data));
                    }
                    let message = Message::Spawn(SpawnData {
                        mob_type: mob.mob_type,
                        replicated,
                    });
                    {
                        {
                            let lvl = ::log::Level::Trace;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!(
                                        "Sending spawn message to {0} clients, {1:?}",
                                        clients.len(),
                                        message,
                                    ),
                                    lvl,
                                    &(
                                        "mmoss::replication::server",
                                        "mmoss::replication::server",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                    for client in &mut *clients {
                        if let Err(e) = client.send(&message).await {
                            {
                                {
                                    let lvl = ::log::Level::Error;
                                    if lvl <= ::log::STATIC_MAX_LEVEL
                                        && lvl <= ::log::max_level()
                                    {
                                        ::log::__private_api::log(
                                            { ::log::__private_api::GlobalLogger },
                                            format_args!("Failed to send spawn message: {0}", e),
                                            lvl,
                                            &(
                                                "mmoss::replication::server",
                                                "mmoss::replication::server",
                                                ::log::__private_api::loc(),
                                            ),
                                            (),
                                        );
                                    }
                                }
                            };
                            continue;
                        }
                    }
                }
            }
            pub async fn serialize(&self, world: &mut World) {
                let mut inner = self.inner.lock().await;
                let dirty = self.dirty.lock().await.drain().collect::<Vec<_>>();
                if !dirty.is_empty() {
                    {
                        {
                            let lvl = ::log::Level::Trace;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!("Dirty entities: {0:?}", dirty.len()),
                                    lvl,
                                    &(
                                        "mmoss::replication::server",
                                        "mmoss::replication::server",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                }
                let mut query = world.query::<&dyn Replicated>();
                for replicated in query.iter_many(world, dirty) {
                    for component in replicated {
                        let message = Message::Update(UpdateData {
                            id: component.id(),
                            data: {
                                let mut data = ::alloc::vec::from_elem(0u8, 512);
                                let result = component.serialize(&mut data);
                                if result.is_err() {
                                    {
                                        {
                                            let lvl = ::log::Level::Error;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                && lvl <= ::log::max_level()
                                            {
                                                ::log::__private_api::log(
                                                    { ::log::__private_api::GlobalLogger },
                                                    format_args!(
                                                        "Failed to serialize update {0:?}: {1}",
                                                        component.id(),
                                                        result.unwrap_err(),
                                                    ),
                                                    lvl,
                                                    &(
                                                        "mmoss::replication::server",
                                                        "mmoss::replication::server",
                                                        ::log::__private_api::loc(),
                                                    ),
                                                    (),
                                                );
                                            }
                                        }
                                    };
                                    continue;
                                }
                                let len = result.unwrap();
                                data.truncate(len);
                                data
                            },
                        });
                        {
                            {
                                let lvl = ::log::Level::Trace;
                                if lvl <= ::log::STATIC_MAX_LEVEL
                                    && lvl <= ::log::max_level()
                                {
                                    ::log::__private_api::log(
                                        { ::log::__private_api::GlobalLogger },
                                        format_args!(
                                            "Replicating message to {0} clients",
                                            inner.clients.len(),
                                        ),
                                        lvl,
                                        &(
                                            "mmoss::replication::server",
                                            "mmoss::replication::server",
                                            ::log::__private_api::loc(),
                                        ),
                                        (),
                                    );
                                }
                            }
                        };
                        for client in &mut inner.clients {
                            client.send(&message).await.unwrap();
                        }
                    }
                }
                if !inner.newly_spawned.is_empty() {
                    {
                        {
                            let lvl = ::log::Level::Trace;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!(
                                        "Newly spawned entities: {0:?}",
                                        inner.newly_spawned.len(),
                                    ),
                                    lvl,
                                    &(
                                        "mmoss::replication::server",
                                        "mmoss::replication::server",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                    let mut query = world
                        .query::<(&MobComponent, All<&dyn Replicated>)>();
                    let entities = mem::replace(
                        &mut inner.newly_spawned,
                        EntityHashSet::new(),
                    );
                    Self::serialize_spawned(
                            &mut inner.clients,
                            query.iter_many(world, entities),
                        )
                        .await;
                    inner.newly_spawned.clear();
                }
                if !inner.pending_full_sync.is_empty() {
                    {
                        {
                            let lvl = ::log::Level::Trace;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api::log(
                                    { ::log::__private_api::GlobalLogger },
                                    format_args!(
                                        "Clients pending full sync: {0}",
                                        inner.pending_full_sync.len(),
                                    ),
                                    lvl,
                                    &(
                                        "mmoss::replication::server",
                                        "mmoss::replication::server",
                                        ::log::__private_api::loc(),
                                    ),
                                    (),
                                );
                            }
                        }
                    };
                    let mut query = world
                        .query::<(&MobComponent, All<&dyn Replicated>)>();
                    Self::serialize_spawned(
                            &mut inner.pending_full_sync,
                            query.iter(world),
                        )
                        .await;
                    let mut drained = inner
                        .pending_full_sync
                        .drain(..)
                        .collect::<Vec<_>>();
                    inner.clients.append(&mut drained);
                }
            }
        }
    }
    #[repr(transparent)]
    pub struct Id(pub u32);
    #[automatically_derived]
    impl ::core::fmt::Debug for Id {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Id", &&self.0)
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Id {
        #[inline]
        fn clone(&self) -> Id {
            let _: ::core::clone::AssertParamIsClone<u32>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Id {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Id {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Id {
        #[inline]
        fn eq(&self, other: &Id) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Id {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u32>;
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Id {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    impl<__Context> ::bincode::Decode<__Context> for Id {
        fn decode<__D: ::bincode::de::Decoder<Context = __Context>>(
            decoder: &mut __D,
        ) -> core::result::Result<Self, ::bincode::error::DecodeError> {
            core::result::Result::Ok(Self {
                0: ::bincode::Decode::decode(decoder)?,
            })
        }
    }
    impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for Id {
        fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context = __Context>>(
            decoder: &mut __D,
        ) -> core::result::Result<Self, ::bincode::error::DecodeError> {
            core::result::Result::Ok(Self {
                0: ::bincode::BorrowDecode::<'_, __Context>::borrow_decode(decoder)?,
            })
        }
    }
    impl ::bincode::Encode for Id {
        fn encode<__E: ::bincode::enc::Encoder>(
            &self,
            encoder: &mut __E,
        ) -> core::result::Result<(), ::bincode::error::EncodeError> {
            ::bincode::Encode::encode(&self.0, encoder)?;
            core::result::Result::Ok(())
        }
    }
    #[repr(transparent)]
    pub struct MobType(pub u32);
    #[automatically_derived]
    impl ::core::fmt::Debug for MobType {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "MobType", &&self.0)
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for MobType {
        #[inline]
        fn clone(&self) -> MobType {
            let _: ::core::clone::AssertParamIsClone<u32>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for MobType {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for MobType {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for MobType {
        #[inline]
        fn eq(&self, other: &MobType) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for MobType {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u32>;
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for MobType {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.0, state)
        }
    }
    impl<__Context> ::bincode::Decode<__Context> for MobType {
        fn decode<__D: ::bincode::de::Decoder<Context = __Context>>(
            decoder: &mut __D,
        ) -> core::result::Result<Self, ::bincode::error::DecodeError> {
            core::result::Result::Ok(Self {
                0: ::bincode::Decode::decode(decoder)?,
            })
        }
    }
    impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for MobType {
        fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context = __Context>>(
            decoder: &mut __D,
        ) -> core::result::Result<Self, ::bincode::error::DecodeError> {
            core::result::Result::Ok(Self {
                0: ::bincode::BorrowDecode::<'_, __Context>::borrow_decode(decoder)?,
            })
        }
    }
    impl ::bincode::Encode for MobType {
        fn encode<__E: ::bincode::enc::Encoder>(
            &self,
            encoder: &mut __E,
        ) -> core::result::Result<(), ::bincode::error::EncodeError> {
            ::bincode::Encode::encode(&self.0, encoder)?;
            core::result::Result::Ok(())
        }
    }
    pub trait Replicated: 'static {
        fn id(&self) -> Id;
        fn serialize(&self, data: &mut [u8]) -> Result<usize>;
        fn replicate(&mut self, data: &[u8]) -> Result<usize>;
    }
    impl bevy_trait_query::TraitQuery for dyn Replicated {}
    impl<
        __Component: Replicated + bevy_trait_query::imports::Component,
    > bevy_trait_query::TraitQueryMarker<dyn Replicated> for (__Component,) {
        type Covered = __Component;
        fn cast(ptr: *mut u8) -> *mut dyn Replicated {
            ptr as *mut __Component as *mut _
        }
    }
    unsafe impl bevy_trait_query::imports::QueryData for &dyn Replicated {
        type ReadOnly = Self;
        const IS_READ_ONLY: bool = true;
        type Item<'__w, '__s> = bevy_trait_query::ReadTraits<'__w, dyn Replicated>;
        #[inline]
        fn shrink<'wlong: 'wshort, 'wshort, 's>(
            item: Self::Item<'wlong, 's>,
        ) -> Self::Item<'wshort, 's> {
            item
        }
        #[inline]
        unsafe fn fetch<'w, 's>(
            state: &'s Self::State,
            fetch: &mut Self::Fetch<'w>,
            entity: bevy_trait_query::imports::Entity,
            table_row: bevy_trait_query::imports::TableRow,
        ) -> Self::Item<'w, 's> {
            <bevy_trait_query::All<
                &dyn Replicated,
            > as bevy_trait_query::imports::QueryData>::fetch(
                state,
                fetch,
                entity,
                table_row,
            )
        }
    }
    unsafe impl bevy_trait_query::imports::ReadOnlyQueryData for &dyn Replicated {}
    unsafe impl<'__a> bevy_trait_query::imports::WorldQuery for &'__a dyn Replicated {
        type Fetch<'__w> = <bevy_trait_query::All<
            &'__a dyn Replicated,
        > as bevy_trait_query::imports::WorldQuery>::Fetch<'__w>;
        type State = bevy_trait_query::TraitQueryState<dyn Replicated>;
        #[inline]
        unsafe fn init_fetch<'w>(
            world: bevy_trait_query::imports::UnsafeWorldCell<'w>,
            state: &Self::State,
            last_run: bevy_trait_query::imports::Tick,
            this_run: bevy_trait_query::imports::Tick,
        ) -> Self::Fetch<'w> {
            <bevy_trait_query::All<
                &dyn Replicated,
            > as bevy_trait_query::imports::WorldQuery>::init_fetch(
                world,
                state,
                last_run,
                this_run,
            )
        }
        const IS_DENSE: bool = <bevy_trait_query::All<
            &dyn Replicated,
        > as bevy_trait_query::imports::WorldQuery>::IS_DENSE;
        #[inline]
        unsafe fn set_archetype<'w>(
            fetch: &mut Self::Fetch<'w>,
            state: &Self::State,
            archetype: &'w bevy_trait_query::imports::Archetype,
            tables: &'w bevy_trait_query::imports::Table,
        ) {
            <bevy_trait_query::All<
                &dyn Replicated,
            > as bevy_trait_query::imports::WorldQuery>::set_archetype(
                fetch,
                state,
                archetype,
                tables,
            );
        }
        #[inline]
        unsafe fn set_table<'w>(
            fetch: &mut Self::Fetch<'w>,
            state: &Self::State,
            table: &'w bevy_trait_query::imports::Table,
        ) {
            <bevy_trait_query::All<
                &dyn Replicated,
            > as bevy_trait_query::imports::WorldQuery>::set_table(fetch, state, table);
        }
        #[inline]
        fn update_component_access(
            state: &Self::State,
            access: &mut bevy_trait_query::imports::FilteredAccess,
        ) {
            <bevy_trait_query::All<
                &dyn Replicated,
            > as bevy_trait_query::imports::WorldQuery>::update_component_access(
                state,
                access,
            );
        }
        #[inline]
        fn init_state(world: &mut bevy_trait_query::imports::World) -> Self::State {
            <bevy_trait_query::All<
                &dyn Replicated,
            > as bevy_trait_query::imports::WorldQuery>::init_state(world)
        }
        #[inline]
        fn get_state(_: &bevy_trait_query::imports::Components) -> Option<Self::State> {
            {
                ::core::panicking::panic_fmt(
                    format_args!(
                        "transmuting and any other operations concerning the state of a query are currently broken and shouldn\'t be used. See https://github.com/JoJoJet/bevy-trait-query/issues/59",
                    ),
                );
            };
        }
        #[inline]
        fn matches_component_set(
            state: &Self::State,
            set_contains_id: &impl Fn(bevy_trait_query::imports::ComponentId) -> bool,
        ) -> bool {
            <bevy_trait_query::All<
                &dyn Replicated,
            > as bevy_trait_query::imports::WorldQuery>::matches_component_set(
                state,
                set_contains_id,
            )
        }
        #[inline]
        fn shrink_fetch<'wlong: 'wshort, 'wshort>(
            fetch: Self::Fetch<'wlong>,
        ) -> Self::Fetch<'wshort> {
            fetch
        }
    }
    unsafe impl<'__a> bevy_trait_query::imports::QueryData for &'__a mut dyn Replicated {
        type ReadOnly = &'__a dyn Replicated;
        type Item<'__w, '__s> = bevy_trait_query::WriteTraits<'__w, dyn Replicated>;
        const IS_READ_ONLY: bool = false;
        #[inline]
        fn shrink<'wlong: 'wshort, 'wshort, 's>(
            item: Self::Item<'wlong, 's>,
        ) -> Self::Item<'wshort, 's> {
            item
        }
        #[inline]
        unsafe fn fetch<'w, 's>(
            state: &'s Self::State,
            fetch: &mut Self::Fetch<'w>,
            entity: bevy_trait_query::imports::Entity,
            table_row: bevy_trait_query::imports::TableRow,
        ) -> Self::Item<'w, 's> {
            <bevy_trait_query::All<
                &mut dyn Replicated,
            > as bevy_trait_query::imports::QueryData>::fetch(
                state,
                fetch,
                entity,
                table_row,
            )
        }
    }
    unsafe impl<'__a> bevy_trait_query::imports::WorldQuery
    for &'__a mut dyn Replicated {
        type Fetch<'__w> = <bevy_trait_query::All<
            &'__a dyn Replicated,
        > as bevy_trait_query::imports::WorldQuery>::Fetch<'__w>;
        type State = bevy_trait_query::TraitQueryState<dyn Replicated>;
        #[inline]
        unsafe fn init_fetch<'w>(
            world: bevy_trait_query::imports::UnsafeWorldCell<'w>,
            state: &Self::State,
            last_run: bevy_trait_query::imports::Tick,
            this_run: bevy_trait_query::imports::Tick,
        ) -> Self::Fetch<'w> {
            <bevy_trait_query::All<
                &mut dyn Replicated,
            > as bevy_trait_query::imports::WorldQuery>::init_fetch(
                world,
                state,
                last_run,
                this_run,
            )
        }
        const IS_DENSE: bool = <bevy_trait_query::All<
            &mut dyn Replicated,
        > as bevy_trait_query::imports::WorldQuery>::IS_DENSE;
        #[inline]
        unsafe fn set_archetype<'w>(
            fetch: &mut Self::Fetch<'w>,
            state: &Self::State,
            archetype: &'w bevy_trait_query::imports::Archetype,
            table: &'w bevy_trait_query::imports::Table,
        ) {
            <bevy_trait_query::All<
                &mut dyn Replicated,
            > as bevy_trait_query::imports::WorldQuery>::set_archetype(
                fetch,
                state,
                archetype,
                table,
            );
        }
        #[inline]
        unsafe fn set_table<'w>(
            fetch: &mut Self::Fetch<'w>,
            state: &Self::State,
            table: &'w bevy_trait_query::imports::Table,
        ) {
            <bevy_trait_query::All<
                &mut dyn Replicated,
            > as bevy_trait_query::imports::WorldQuery>::set_table(fetch, state, table);
        }
        #[inline]
        fn update_component_access(
            state: &Self::State,
            access: &mut bevy_trait_query::imports::FilteredAccess,
        ) {
            <bevy_trait_query::All<
                &mut dyn Replicated,
            > as bevy_trait_query::imports::WorldQuery>::update_component_access(
                state,
                access,
            );
        }
        #[inline]
        fn init_state(world: &mut bevy_trait_query::imports::World) -> Self::State {
            <bevy_trait_query::All<
                &mut dyn Replicated,
            > as bevy_trait_query::imports::WorldQuery>::init_state(world)
        }
        #[inline]
        fn get_state(_: &bevy_trait_query::imports::Components) -> Option<Self::State> {
            {
                ::core::panicking::panic_fmt(
                    format_args!(
                        "transmuting and any other operations concerning the state of a query are currently broken and shouldn\'t be used. See https://github.com/JoJoJet/bevy-trait-query/issues/59",
                    ),
                );
            };
        }
        #[inline]
        fn matches_component_set(
            state: &Self::State,
            set_contains_id: &impl Fn(bevy_trait_query::imports::ComponentId) -> bool,
        ) -> bool {
            <bevy_trait_query::All<
                &mut dyn Replicated,
            > as bevy_trait_query::imports::WorldQuery>::matches_component_set(
                state,
                set_contains_id,
            )
        }
        #[inline]
        fn shrink_fetch<'wlong: 'wshort, 'wshort>(
            fetch: Self::Fetch<'wlong>,
        ) -> Self::Fetch<'wshort> {
            fetch
        }
    }
    pub struct UpdateData {
        pub id: Id,
        pub data: Vec<u8>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UpdateData {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "UpdateData",
                "id",
                &self.id,
                "data",
                &&self.data,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for UpdateData {
        #[inline]
        fn clone(&self) -> UpdateData {
            UpdateData {
                id: ::core::clone::Clone::clone(&self.id),
                data: ::core::clone::Clone::clone(&self.data),
            }
        }
    }
    impl<__Context> ::bincode::Decode<__Context> for UpdateData {
        fn decode<__D: ::bincode::de::Decoder<Context = __Context>>(
            decoder: &mut __D,
        ) -> core::result::Result<Self, ::bincode::error::DecodeError> {
            core::result::Result::Ok(Self {
                id: ::bincode::Decode::decode(decoder)?,
                data: ::bincode::Decode::decode(decoder)?,
            })
        }
    }
    impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for UpdateData {
        fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context = __Context>>(
            decoder: &mut __D,
        ) -> core::result::Result<Self, ::bincode::error::DecodeError> {
            core::result::Result::Ok(Self {
                id: ::bincode::BorrowDecode::<'_, __Context>::borrow_decode(decoder)?,
                data: ::bincode::BorrowDecode::<'_, __Context>::borrow_decode(decoder)?,
            })
        }
    }
    impl ::bincode::Encode for UpdateData {
        fn encode<__E: ::bincode::enc::Encoder>(
            &self,
            encoder: &mut __E,
        ) -> core::result::Result<(), ::bincode::error::EncodeError> {
            ::bincode::Encode::encode(&self.id, encoder)?;
            ::bincode::Encode::encode(&self.data, encoder)?;
            core::result::Result::Ok(())
        }
    }
    pub struct SpawnData {
        pub mob_type: MobType,
        pub replicated: Vec<(Id, Vec<u8>)>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for SpawnData {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "SpawnData",
                "mob_type",
                &self.mob_type,
                "replicated",
                &&self.replicated,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for SpawnData {
        #[inline]
        fn clone(&self) -> SpawnData {
            SpawnData {
                mob_type: ::core::clone::Clone::clone(&self.mob_type),
                replicated: ::core::clone::Clone::clone(&self.replicated),
            }
        }
    }
    impl<__Context> ::bincode::Decode<__Context> for SpawnData {
        fn decode<__D: ::bincode::de::Decoder<Context = __Context>>(
            decoder: &mut __D,
        ) -> core::result::Result<Self, ::bincode::error::DecodeError> {
            core::result::Result::Ok(Self {
                mob_type: ::bincode::Decode::decode(decoder)?,
                replicated: ::bincode::Decode::decode(decoder)?,
            })
        }
    }
    impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for SpawnData {
        fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context = __Context>>(
            decoder: &mut __D,
        ) -> core::result::Result<Self, ::bincode::error::DecodeError> {
            core::result::Result::Ok(Self {
                mob_type: ::bincode::BorrowDecode::<
                    '_,
                    __Context,
                >::borrow_decode(decoder)?,
                replicated: ::bincode::BorrowDecode::<
                    '_,
                    __Context,
                >::borrow_decode(decoder)?,
            })
        }
    }
    impl ::bincode::Encode for SpawnData {
        fn encode<__E: ::bincode::enc::Encoder>(
            &self,
            encoder: &mut __E,
        ) -> core::result::Result<(), ::bincode::error::EncodeError> {
            ::bincode::Encode::encode(&self.mob_type, encoder)?;
            ::bincode::Encode::encode(&self.replicated, encoder)?;
            core::result::Result::Ok(())
        }
    }
    pub enum Message {
        Spawn(SpawnData),
        Update(UpdateData),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Message {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Message::Spawn(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Spawn",
                        &__self_0,
                    )
                }
                Message::Update(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Update",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Message {
        #[inline]
        fn clone(&self) -> Message {
            match self {
                Message::Spawn(__self_0) => {
                    Message::Spawn(::core::clone::Clone::clone(__self_0))
                }
                Message::Update(__self_0) => {
                    Message::Update(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    impl<__Context> ::bincode::Decode<__Context> for Message {
        fn decode<__D: ::bincode::de::Decoder<Context = __Context>>(
            decoder: &mut __D,
        ) -> core::result::Result<Self, ::bincode::error::DecodeError> {
            let variant_index = <u32 as ::bincode::Decode<
                __D::Context,
            >>::decode(decoder)?;
            match variant_index {
                0u32 => {
                    core::result::Result::Ok(Self::Spawn {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    })
                }
                1u32 => {
                    core::result::Result::Ok(Self::Update {
                        0: ::bincode::Decode::<__D::Context>::decode(decoder)?,
                    })
                }
                variant => {
                    core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "Message",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                            min: 0,
                            max: 1,
                        },
                    })
                }
            }
        }
    }
    impl<'__de, __Context> ::bincode::BorrowDecode<'__de, __Context> for Message {
        fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de, Context = __Context>>(
            decoder: &mut __D,
        ) -> core::result::Result<Self, ::bincode::error::DecodeError> {
            let variant_index = <u32 as ::bincode::Decode<
                __D::Context,
            >>::decode(decoder)?;
            match variant_index {
                0u32 => {
                    core::result::Result::Ok(Self::Spawn {
                        0: ::bincode::BorrowDecode::<
                            __D::Context,
                        >::borrow_decode(decoder)?,
                    })
                }
                1u32 => {
                    core::result::Result::Ok(Self::Update {
                        0: ::bincode::BorrowDecode::<
                            __D::Context,
                        >::borrow_decode(decoder)?,
                    })
                }
                variant => {
                    core::result::Result::Err(::bincode::error::DecodeError::UnexpectedVariant {
                        found: variant,
                        type_name: "Message",
                        allowed: &::bincode::error::AllowedEnumVariants::Range {
                            min: 0,
                            max: 1,
                        },
                    })
                }
            }
        }
    }
    impl ::bincode::Encode for Message {
        fn encode<__E: ::bincode::enc::Encoder>(
            &self,
            encoder: &mut __E,
        ) -> core::result::Result<(), ::bincode::error::EncodeError> {
            match self {
                Self::Spawn(field_0) => {
                    <u32 as ::bincode::Encode>::encode(&(0u32), encoder)?;
                    ::bincode::Encode::encode(field_0, encoder)?;
                    core::result::Result::Ok(())
                }
                Self::Update(field_0) => {
                    <u32 as ::bincode::Encode>::encode(&(1u32), encoder)?;
                    ::bincode::Encode::encode(field_0, encoder)?;
                    core::result::Result::Ok(())
                }
            }
        }
    }
    impl MessageTrait for Message {
        fn serialize(&self, data: &mut [u8]) -> Result<usize> {
            Ok(bincode::encode_into_slice(self, data, bincode::config::standard())?)
        }
    }
    pub struct MessageFactoryNew;
    impl MessageFactoryTrait for MessageFactoryNew {
        type Message = Message;
        fn deserialize(
            &self,
            _context: &(),
            _data: &[u8],
        ) -> Result<(Self::Message, usize)> {
            Ok(bincode::decode_from_slice(_data, bincode::config::standard())?)
        }
    }
}
