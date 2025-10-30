use crate::prelude::*;
use dotenvy::dotenv;
use futures::future::try_join_all;
use logging::init_logging;
use model::StartupListener;
use poise::{Framework, FrameworkOptions};

mod embedder;
#[cfg(debug_assertions)]
mod examples;
mod logging;
mod macros;
mod model;
pub mod prelude;

#[cfg(debug_assertions)]
const TOKEN_ENV: &str = "DEV_DISCORD_TOKEN";
#[cfg(not(debug_assertions))]
const TOKEN_ENV: &str = "PROD_DISCORD_TOKEN";

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    init_logging();

    let token = env::var(TOKEN_ENV).expect("Environment variable {TOKEN_ENV} must be set");

    fire_startup_events().await?;

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES;
    let framework = init_framework();

    let mut client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;
    client.start().await?;
    Ok(())
}

fn init_framework() -> Framework<GlobalState, Error> {
    #[cfg(debug_assertions)]
    let dev_guild_id: GuildId = env::var("DEV_GUILD_ID")
        .expect("env variable DEV_GUILD_ID should be set")
        .parse()
        .expect("env variable DEV_GUILD_ID should be a u64");

    Framework::builder()
        .options(FrameworkOptions {
            commands: collect_commands(),
            event_handler: |framework, event| Box::pin(event_handler(framework, event)),
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                #[cfg(debug_assertions)]
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    dev_guild_id,
                )
                .await?;

                #[cfg(not(debug_assertions))]
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(GlobalState::new())
            })
        })
        .build()
}

async fn fire_startup_events() -> Result<(), Error> {
    let futures = inventory::iter::<StartupListener>
        .into_iter()
        .map(|listener| (listener.handle)())
        .collect::<Vec<_>>();

    try_join_all(futures).await?;
    Ok(())
}

async fn event_handler(
    ctx: FrameworkContext<'_, GlobalState, Error>,
    event: &FullEvent,
) -> Result<(), Error> {
    let futures = inventory::iter::<EventListener>
        .into_iter()
        .map(|listener| (listener.handle)(ctx, event))
        .collect::<Vec<_>>();

    try_join_all(futures).await?;
    Ok(())
}

fn collect_commands() -> Vec<Command<GlobalState, Error>> {
    inventory::iter::<CommandRegistry>
        .into_iter()
        .flat_map(|p| p.0())
        .collect()
}
