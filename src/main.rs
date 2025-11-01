use crate::prelude::*;
use dotenvy::dotenv;
use futures::future::{join_all, try_join_all};
use model::{EnvRequirement, EnvValidationError, StartupListener};
use poise::{Framework, FrameworkOptions};
use tracing::init_tracing;

mod embedder;
#[cfg(debug_assertions)]
mod examples;
mod macros;
mod model;
pub mod prelude;
mod tracing;

register_env!(DISCORD_TOKEN, String);
register_env!(DEV_GUILD_ID, GuildId);

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    dotenv().ok();
    init_tracing();

    verify_env_requirements().await?;
    let token = get_env(&DISCORD_TOKEN);

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
                    get_env(&DEV_GUILD_ID),
                )
                .await?;

                #[cfg(not(debug_assertions))]
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(GlobalState::new())
            })
        })
        .build()
}

async fn fire_startup_events() -> Result<()> {
    let futures = inventory::iter::<StartupListener>
        .into_iter()
        .map(|listener| (listener.handle)())
        .collect::<Vec<_>>();

    try_join_all(futures).await?;
    Ok(())
}

async fn verify_env_requirements() -> Result<()> {
    let futures = inventory::iter::<EnvRequirement>
        .into_iter()
        .map(|requirement| (requirement.validate)())
        .collect::<Vec<_>>();

    let results = join_all(futures).await;
    let errors = results
        .into_iter()
        .filter_map(|result| result.err())
        .collect::<Vec<_>>();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(EnvValidationError::from_errors(errors).into())
    }
}

async fn event_handler(
    ctx: FrameworkContext<'_, GlobalState, Error>,
    event: &FullEvent,
) -> Result<()> {
    let futures = inventory::iter::<EventListener>
        .into_iter()
        .map(|listener| (listener.handle)(ctx, event))
        .collect::<Vec<_>>();

    try_join_all(futures).await?;
    Ok(())
}

fn collect_commands() -> Vec<Command<GlobalState, Error>> {
    inventory::iter::<BotCommand>
        .into_iter()
        .flat_map(|p| p.0())
        .collect()
}
