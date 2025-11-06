#![warn(clippy::pedantic, clippy::cargo, clippy::nursery)]

use crate::{model::GlobalDataRegistry, prelude::*};
use dotenvy::dotenv;
use futures::future::{join_all, try_join_all};
use model::{EnvRegistry, EnvValidationError, StartupListenerRegistry};
use poise::{Framework, FrameworkOptions};
use tracing_subscriber::{EnvFilter, filter::LevelFilter, fmt};

mod helpers;
mod macros;
mod model;
mod modules;
pub mod prelude;

register_env!(DISCORD_TOKEN, String);
register_env!(DEV_GUILD_ID, GuildId);

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    dotenv().ok();
    init_tracing();

    verify_env_requirements().await?;
    fire_startup_events().await?;

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES;
    let framework = init_framework();
    let token = DISCORD_TOKEN.get();

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
                    DEV_GUILD_ID.get().clone(),
                )
                .await?;

                #[cfg(not(debug_assertions))]
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                init_global_data(ctx).await;

                Ok(GlobalState {
                    ..Default::default()
                })
            })
        })
        .build()
}

async fn fire_startup_events() -> Result<()> {
    let futures = inventory::iter::<StartupListenerRegistry>
        .into_iter()
        .map(|listener| listener.0())
        .collect::<Vec<_>>();
    info!("Firing {} startup events", futures.len());

    //if a startup event returns an error it should short circuit, hence using try variant of join_all
    //startups are meant for logic that your module HAS to have run before the bot can start
    try_join_all(futures).await?;
    Ok(())
}

async fn verify_env_requirements() -> Result<()> {
    let futures = inventory::iter::<EnvRegistry>
        .into_iter()
        .map(|requirement| requirement.0())
        .collect::<Vec<_>>();
    info!("Verifying {} environment variables", futures.len());

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
    let futures = inventory::iter::<EventListenerRegistry>
        .into_iter()
        .map(|listener| listener.0(ctx, event))
        .collect::<Vec<_>>();

    join_all(futures).await;
    Ok(())
}

fn collect_commands() -> Vec<Command<GlobalState, Error>> {
    let commands: Vec<Command<GlobalState, Error>> = inventory::iter::<CommandRegistry>
        .into_iter()
        .flat_map(|p| p.0())
        .collect();
    info!("Registering {} commands", commands.len());
    commands
}

async fn init_global_data(ctx: &poise::serenity_prelude::Context) {
    let mut type_map = ctx.data.write().await;
    let mut count = 0;
    inventory::iter::<GlobalDataRegistry>
        .into_iter()
        .for_each(|data| {
            data.0(&mut type_map);
            count += 1;
        });
    info!("Registered {} global data types", count);
}

#[cfg(not(debug_assertions))]
const DEFAULT_FILTER_DIRECTIVES: &str = "peoplebot=info,poise=info,serenity=info,tokio=warn";
#[cfg(debug_assertions)]
const DEFAULT_FILTER_DIRECTIVES: &str = "peoplebot=debug,poise=info,serenity=info,tokio=warn";

pub fn init_tracing() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_FILTER_DIRECTIVES.into()));

    let builder = fmt::fmt().with_env_filter(env_filter).with_target(false);

    #[cfg(debug_assertions)]
    let builder = builder
        .with_target(true)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    #[cfg(not(debug_assertions))]
    let builder = builder.compact();

    if let Err(error) = builder.try_init() {
        eprintln!("tracing subscriber already initialized: {error}");
    }
}
