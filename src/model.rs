use derive_new::new;
use futures::future::BoxFuture;

use crate::prelude::*;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, GlobalState, Error>;

#[derive(new)]
pub struct GlobalState {}

pub struct CommandRegistry(pub fn() -> Vec<Command<GlobalState, Error>>);

inventory::collect!(CommandRegistry);

pub type StartupListenerFn = fn() -> BoxFuture<'static, Result<(), Error>>;

pub struct StartupListener {
    pub handle: StartupListenerFn,
}

inventory::collect!(StartupListener);

pub type EventHandlerFn = for<'a> fn(
    FrameworkContext<'a, GlobalState, Error>,
    &'a FullEvent,
) -> BoxFuture<'a, Result<(), Error>>;

pub struct EventListener {
    pub handle: EventHandlerFn,
}

inventory::collect!(EventListener);
