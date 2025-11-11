/// Registers one or more commands with the bot.
/// ```
/// use peoplebot::prelude::*;
///
/// register_commands!(command);
///
/// #[command(slash_command, prefix_command)]
/// async fn command(
///     _ctx: Context<'_>,
///     param: String,
/// ) -> Result<()> {
///     Ok(())
/// }
///
/// ```
#[macro_export]
macro_rules! register_commands {
      ($($command:path),+ $(,)?) => {
          const _: () = {
              fn __peoplebot_command_list() -> Vec<
                  poise::Command<$crate::core::GlobalState, $crate::prelude::Error>
              > {
                  vec![$($command()),+]
              }

              inventory::submit! {
                  $crate::core::CommandRegistry(__peoplebot_command_list)
              }
          };
      };
  }

/// Registers an async event handler to be called when an event occurs.
/// ```
/// use peoplebot::prelude::*;
///
/// async fn event_listener(
///     ctx: FrameworkContext<'_, GlobalState, Error>,
///     event: &FullEvent,
/// ) -> Result<()> {
///     match event {
///         FullEvent::MessageCreate(message) => {
///             if message.content == "!ping" {
///                 message.reply(ctx, "Pong!").await?;
///             }
///         }
///         _ => {}
///     }
///     Ok(())
/// }
///
/// register_event_listener!(event_listener);
/// ```
/// This macro is just short hand for the following:
/// ```
/// inventory::submit! {
///    peoplebot::core::EventListenerRegistry(event_listener)
///}
/// ```
#[macro_export]
macro_rules! register_event_listener {
    ($handler:path) => {
        const _: () = {
            fn __peoplebot_event_wrapper<'a>(
                ctx: poise::FrameworkContext<'a, $crate::core::GlobalState, $crate::prelude::Error>,
                event: &'a poise::serenity_prelude::FullEvent,
            ) -> ::futures::future::BoxFuture<'a, $crate::prelude::Result<()>> {
                async move { $handler(ctx, event).await }.boxed()
            }

            ::inventory::submit! {
                $crate::core::EventListenerRegistry(__peoplebot_event_wrapper)

            }
        };
    };
}

/// Registers an async startup hook to be called when the process starts. Occurs before any discord specific logic occurs.
/// ```
/// use peoplebot::prelude::*;
///
/// async fn startup_listener() -> Result<()> {
///     Ok(())
/// }
///
/// register_startup_listener!(startup_listener);
/// ```
/// This macro is just short hand for the following:
/// ```
/// inventory::submit! {
///    peoplebot::core::StartupListenerRegistry(startup_listener)
///}
/// ```
#[macro_export]
macro_rules! register_startup_listener {
    ($handler:path) => {
        const _: () = {
            fn __peoplebot_startup_wrapper()
            -> ::futures::future::BoxFuture<'static, $crate::prelude::Result<()>> {
                async move { $handler().await }.boxed()
            }

            ::inventory::submit! {
                $crate::core::StartupListenerRegistry(__peoplebot_startup_wrapper)
            }
        };
    };
}

/// Registers a global data initializer function to be invoked during framework startup.
/// The registered initializer must insert the data into the [`TypeMap`].
/// This macro can be invoked multiple times if you prefer separate types instead of nesting them.
/// * The outermost type you insert **must** be wrapped in an [`Arc`].
/// * Any interior mutable data must be protected by a [`Mutex`] or [`RwLock`], or be an atomic type.
///
/// ```
/// use peoplebot::prelude::*;
/// use std::collections::HashMap;
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicUsize, Ordering};
/// use tokio::sync::RwLock;
///
/// pub struct YourCounters;
/// impl TypeMapKey for YourCounters {
///     type Value = Arc<RwLock<HashMap<String, u64>>>;
/// }
///
/// pub struct YourTotalMessages;
/// impl TypeMapKey for YourTotalMessages {
///     type Value = Arc<AtomicUsize>;
/// }
///
/// fn init_counters(map: &mut TypeMap) {
///     map.insert::<YourCounters>(Arc::new(RwLock::new(HashMap::new())));
/// }
///
/// fn init_total(map: &mut TypeMap) {
///     map.insert::<YourTotalMessages>(Arc::new(AtomicUsize::new(0)));
/// }
///
/// register_global_data!(init_counters);
/// register_global_data!(init_total);
///
/// ```
///
/// Later, you can retrieve your data from a `Context` instance:
///
/// ```
/// async fn bump(ctx: &poise::serenity_prelude::Context, name: &str) {
///     let counters: Arc<RwLock<HashMap<String, u64>>> = {
///         // Hold the framework TypeMap read guard only long enough to clone the Arc.
///         let data = ctx.data.read().await;
///         data.get::<YourCounters>()
///             .expect("YourCounters missing")
///             .clone()
///     };
///
///     // Use a scope to release the write lock as soon as possible.
///     {
///         let mut map = counters.write().await;
///         *map.entry(name.to_string()).or_insert(0) += 1;
///     }
///
///     let total: Arc<AtomicUsize> = {
///         // Again, grab a fresh read guard, clone the Arc, then drop the guard.
///         let data = ctx.data.read().await;
///         data.get::<YourTotalMessages>()
///             .expect("YourTotalMessages missing")
///             .clone()
///     };
///     total.fetch_add(1, Ordering::SeqCst);
/// }
/// ```
/// This macro is just short hand for the following:
/// ```
/// inventory::submit! {
///    peoplebot::core::GlobalDataRegistry(your_init_function)
///}
/// ```
#[macro_export]
macro_rules! register_global_data {
    ($init:path) => {
        const _: () = {
            fn __peoplebot_global_data_wrapper(
                map: &mut ::poise::serenity_prelude::prelude::TypeMap,
            ) {
                $init(map);
            }

            ::inventory::submit! {
                $crate::core::GlobalDataRegistry(__peoplebot_global_data_wrapper)
            }
        };
    };
}

/// Registers an environment variable and the type it must coerce to; this check runs at startup.
///
/// # Prefix resolution
/// - **Unprefixed identifiers** (e.g. `CONCURRENCY_LIMIT`) resolve by build:
///   - **Debug**: use `DEV_<IDENTIFIER>` if set; otherwise fall back to `BOTH_<IDENTIFIER>`.
///   - **Release**: use `PROD_<IDENTIFIER>` if set; otherwise fall back to `BOTH_<IDENTIFIER>`.
/// - **Manually prefixed identifiers**:
///   - `DEV_…` → validated **only in debug** builds.
///   - `PROD_…` → validated **only in release** builds.
///   - `BOTH_…` → validated in **both** builds (no DEV/PROD preference).
///
/// # Usage
/// Retrieving the value is done with `YOUR_IDENTIFIER.get()`. You never need to include a prefix;
/// the correct env var is chosen automatically according to the rules above.
///
/// ```
/// register_env!(YOUR_IDENTIFIER, String);
///
/// fn example() {
///     let env_value = YOUR_IDENTIFIER.get();
/// }
/// ```
///
/// For optional values, use:
/// ```
/// register_env!(YOUR_OTHER_IDENTIFIER, Option<u32>);
/// ```
/// Missing/empty values become `None`; parse/UTF-8 errors still fail validation.

#[macro_export]
macro_rules! register_env {
    // Optional form: Option<T>
    ($store:ident, Option<$ty:ty>) => {
        #[allow(non_upper_case_globals)]
        pub static $store: $crate::core::EnvStore<Option<$ty>> =
            $crate::core::EnvStore::new(stringify!($store));

        const _: () = {
            fn __peoplebot_env_wrapper() -> ::futures::future::BoxFuture<
                'static,
                std::result::Result<(), $crate::core::EnvError>,
            > {
                ::std::boxed::Box::pin(async move {
                    $crate::core::env::validate_env::<$crate::core::EnvStore<Option<$ty>>, $ty>(
                        &$store,
                    )
                    .await
                })
            }

            ::inventory::submit! {
                $crate::core::EnvRegistry(__peoplebot_env_wrapper)
            }
        };
    };

    // Required form: T
    ($store:ident, $ty:ty) => {
        #[allow(non_upper_case_globals)]
        pub static $store: $crate::core::EnvStore<$ty> =
            $crate::core::EnvStore::new(stringify!($store));

        const _: () = {
            fn __peoplebot_env_wrapper() -> ::futures::future::BoxFuture<
                'static,
                std::result::Result<(), $crate::core::EnvError>,
            > {
                ::std::boxed::Box::pin(async move {
                    $crate::core::env::validate_env::<$crate::core::EnvStore<$ty>, $ty>(&$store)
                        .await
                })
            }

            ::inventory::submit! {
                $crate::core::EnvRegistry(__peoplebot_env_wrapper)
            }
        };
    };
}

/// Returns early from the current function with a [`crate::core::error::UserError`] that is safe to show to end users.
/// Don't forget to delete any temporary ephemerals before calling this.
#[macro_export]
macro_rules! bail_to_user {
    ($($arg:tt)*) => {
        ::anyhow::bail!($crate::core::error::UserError(::anyhow::anyhow!($($arg)*)))
    };
}
