/// Registers one or more commands with the bot.
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

/// Registers an async event handler to be called when an event occurs.
#[macro_export]
macro_rules! register_event_listener {
    ($handler:path) => {
        const _: () = {
            fn __peoplebot_event_wrapper<'a>(
                ctx: poise::FrameworkContext<'a, $crate::model::GlobalState, $crate::model::Error>,
                event: &'a poise::serenity_prelude::FullEvent,
            ) -> ::futures::future::BoxFuture<'a, Result<(), $crate::model::Error>> {
                async move { $handler(ctx, event).await }.boxed()
            }

            ::inventory::submit! {
                $crate::model::EventListener {
                    handle: __peoplebot_event_wrapper,
                }
            }
        };
    };
}

/// Registers an async startup hook to be called when the process starts. Occurs before any discord specific logic occurs.
#[macro_export]
macro_rules! register_startup_listener {
    ($handler:path) => {
        const _: () = {
            fn __peoplebot_startup_wrapper()
            -> ::futures::future::BoxFuture<'static, Result<(), $crate::model::Error>> {
                async move { $handler().await }.boxed()
            }

            ::inventory::submit! {
                $crate::model::StartupListener {
                    handle: __peoplebot_startup_wrapper,
                }
            }
        };
    };
}
