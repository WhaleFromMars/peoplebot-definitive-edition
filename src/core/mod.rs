pub(crate) mod env;
pub(crate) mod error;

pub(crate) use env::{EnvError, EnvStore, EnvValidationError};
use futures::future::BoxFuture;
use songbird::Songbird;

use crate::prelude::*;

pub(crate) type Context<'a> = poise::Context<'a, GlobalState, Error>;

pub(crate) struct GlobalState {
    pub(crate) songbird: Arc<Songbird>,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            songbird: songbird::Songbird::serenity(),
        }
    }
}

pub(crate) struct GlobalDataRegistry(pub(crate) fn(&mut TypeMap));
inventory::collect!(GlobalDataRegistry);

pub(crate) struct CommandRegistry(pub(crate) fn() -> Vec<Command<GlobalState, Error>>);
inventory::collect!(CommandRegistry);

pub(crate) struct StartupListenerRegistry(pub(crate) fn() -> BoxFuture<'static, Result<()>>);
inventory::collect!(StartupListenerRegistry);

pub(crate) struct EnvRegistry(
    pub(crate) fn() -> BoxFuture<'static, std::result::Result<(), EnvError>>,
);
inventory::collect!(EnvRegistry);

pub(crate) struct EventListenerRegistry(
    pub(crate)  for<'a> fn(
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
