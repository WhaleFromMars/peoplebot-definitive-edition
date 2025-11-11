pub mod env;
pub mod error;

pub use env::{EnvError, EnvStore, EnvValidationError};
use futures::future::BoxFuture;
use songbird::Songbird;

use crate::prelude::*;

pub type Context<'a> = poise::Context<'a, GlobalState, Error>;

pub struct GlobalState {
    pub songbird: Arc<Songbird>,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            songbird: songbird::Songbird::serenity(),
        }
    }
}

pub struct GlobalDataRegistry(pub fn(&mut TypeMap));
inventory::collect!(GlobalDataRegistry);

pub struct CommandRegistry(pub fn() -> Vec<Command<GlobalState, Error>>);
inventory::collect!(CommandRegistry);

pub struct StartupListenerRegistry(pub fn() -> BoxFuture<'static, Result<()>>);
inventory::collect!(StartupListenerRegistry);

pub struct EnvRegistry(pub fn() -> BoxFuture<'static, std::result::Result<(), EnvError>>);
inventory::collect!(EnvRegistry);

pub struct EventListenerRegistry(
    pub  for<'a> fn(
        FrameworkContext<'a, GlobalState, Error>,
        &'a FullEvent,
    ) -> BoxFuture<'a, Result<()>>,
);
inventory::collect!(EventListenerRegistry);

pub(crate) trait DeleteHandle<'a> {
    async fn delete(&self, ctx: Context<'a>) -> Result<(), Error>;
}

impl<'a> DeleteHandle<'a> for Option<ReplyHandle<'a>> {
    async fn delete(&self, ctx: Context<'a>) -> Result<(), Error> {
        if let Some(handle) = self {
            handle.delete(ctx).await?;
        }
        Ok(())
    }
}
