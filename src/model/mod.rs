mod env;

use std::sync::{Arc, atomic::AtomicBool};

pub use env::{EnvError, EnvStore, EnvValidationError, get_env};

use futures::future::BoxFuture;
use songbird::Songbird;

use crate::prelude::*;

pub type Context<'a> = poise::Context<'a, GlobalState, Error>;

pub struct GlobalState {
    pub dl_task_running: AtomicBool,
    pub songbird: Arc<Songbird>,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            dl_task_running: AtomicBool::new(false),
            songbird: songbird::Songbird::serenity(),
        }
    }
}

pub struct BotCommand(pub fn() -> Vec<Command<GlobalState, Error>>);

inventory::collect!(BotCommand);

pub type StartupListenerFn = fn() -> BoxFuture<'static, Result<()>>;

pub struct StartupListener {
    pub handle: StartupListenerFn,
}

inventory::collect!(StartupListener);

pub type EnvRequirementResult = std::result::Result<(), EnvError>;
pub type EnvRequirementFn = fn() -> BoxFuture<'static, EnvRequirementResult>;

pub struct EnvRequirement {
    pub validate: EnvRequirementFn,
}

inventory::collect!(EnvRequirement);

pub type EventListenerFn = for<'a> fn(
    FrameworkContext<'a, GlobalState, Error>,
    &'a FullEvent,
) -> BoxFuture<'a, Result<()>>;

pub struct EventListener {
    pub handle: EventListenerFn,
}

inventory::collect!(EventListener);
