/// Registers one or more commands with the bot.
#[macro_export]
macro_rules! register_commands {
      ($($command:path),+ $(,)?) => {
          const _: () = {
              fn __peoplebot_command_list() -> Vec<
                  poise::Command<$crate::model::GlobalState, $crate::prelude::Error>
              > {
                  vec![$($command()),+]
              }

              inventory::submit! {
                  $crate::model::BotCommand(__peoplebot_command_list)
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
                ctx: poise::FrameworkContext<
                    'a,
                    $crate::model::GlobalState,
                    $crate::prelude::Error,
                >,
                event: &'a poise::serenity_prelude::FullEvent,
            ) -> ::futures::future::BoxFuture<'a, $crate::prelude::Result<()>> {
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
            -> ::futures::future::BoxFuture<'static, $crate::prelude::Result<()>> {
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

/// Registers an environment variable name and type it must coerce to, this check triggers at startup.
/// The variable name is prefixed automatically as `DEV_<YOUR_IDENTIFIER>` in debug builds and `PROD_<YOUR_IDENTIFIER>` otherwise.
/// If you manually prefix the variable name with PROD_ or DEV_, it will ONLY exist in that environment.
/// \
/// Retrieving the value is done using get_env(&YOUR_IDENTIFIER), you do not need to prefix this,
/// the function will return the correct version for prod or dev environments.
/// ```
/// register_env!(YOUR_IDENTIFIER, String);
///
/// fn () {
///  let env_value = get_env(&YOUR_IDENTIFIER);
/// }
/// ```
#[macro_export]
macro_rules! register_env {
    ($store:ident, $ty:ty) => {
        #[allow(non_upper_case_globals)]
        pub static $store: $crate::model::EnvStore<$ty> =
            $crate::model::EnvStore::new(stringify!($store));

        const _: () = {
            fn __peoplebot_env_wrapper()
            -> ::futures::future::BoxFuture<'static, $crate::model::EnvRequirementResult> {
                ::std::boxed::Box::pin(async move {
                    if !$store.is_active() {
                        return ::std::result::Result::Ok(());
                    }
                    let env_name = $store.name();
                    let value = match $crate::prelude::env::var(env_name) {
                        Ok(val) => val,
                        Err(::std::env::VarError::NotPresent) => {
                            return ::std::result::Result::Err($crate::model::EnvError::Missing {
                                var: env_name,
                            });
                        }
                        Err(::std::env::VarError::NotUnicode(val)) => {
                            return ::std::result::Result::Err($crate::model::EnvError::Invalid {
                                var: env_name,
                                reason: format!(
                                    "value is not valid UTF-8: {}",
                                    val.to_string_lossy()
                                ),
                            });
                        }
                    };

                    let parsed =
                        value
                            .parse::<$ty>()
                            .map_err(|err| $crate::model::EnvError::Invalid {
                                var: env_name,
                                reason: err.to_string(),
                            })?;

                    $store.set(parsed);
                    Ok(())
                })
            }

            ::inventory::submit! {
                $crate::model::EnvRequirement {
                    validate: __peoplebot_env_wrapper,
                }
            }
        };
    };
}
