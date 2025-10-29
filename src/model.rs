use derive_new::new;

use crate::prelude::*;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, GlobalState, Error>;

#[derive(new)]
pub struct GlobalState {}

pub struct CommandRegistry(pub fn() -> Vec<Command<GlobalState, Error>>);
inventory::collect!(CommandRegistry);

// pub struct EventRegistry
//
// pub struct EventHandler {
//     try_consume: fn(&FullEvent) -> Result<(), Error>,
//     consume: fn(&FullEvent) -> Result<(), Error>,
// }
