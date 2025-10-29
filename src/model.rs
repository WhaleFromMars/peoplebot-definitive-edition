use derive_new::new;
use futures::future::BoxFuture;

use crate::prelude::*;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, GlobalState, Error>;

#[derive(new)]
pub struct GlobalState {}

pub struct CommandRegistry(pub fn() -> Vec<Command<GlobalState, Error>>);
inventory::collect!(CommandRegistry);

/// Registers one or more command constructors with the global inventory.
#[macro_export]
macro_rules! register_commands {
      ($($command:path),+ $(,)?) => {
          const _: () = {
              fn __peoplebot_command_list() -> Vec<
                  poise::Command<$crate::model::GlobalState, $crate::model::Error>
              > {
                  vec![$($command()),+]
              }

              inventory::submit! {
                  $crate::model::CommandRegistry(__peoplebot_command_list)
              }
          };
      };
  }

pub type EventHandlerFn = for<'a> fn(
    FrameworkContext<'a, GlobalState, Error>,
    &'a FullEvent,
) -> BoxFuture<'a, Result<(), Error>>;
pub struct EventListener {
    pub handle: EventHandlerFn,
}

inventory::collect!(EventListener);

/// Registers an async event handler with the global inventory.
#[macro_export]
macro_rules! register_event_listener {
    ($handler:path) => {
        const _: () = {
            fn __peoplebot_event_wrapper<'a>(
                ctx: poise::FrameworkContext<'a, $crate::model::GlobalState, $crate::model::Error>,
                event: &'a poise::serenity_prelude::FullEvent,
            ) -> futures::future::BoxFuture<'a, Result<(), $crate::model::Error>> {
                Box::pin($handler(ctx, event))
            }

            inventory::submit! {
                $crate::model::EventListener {
                    handle: __peoplebot_event_wrapper,
                }
            }
        };
    };
}
