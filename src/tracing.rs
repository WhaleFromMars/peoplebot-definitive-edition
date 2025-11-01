use std::env;

use tracing_subscriber::{EnvFilter, filter::LevelFilter, fmt};

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

    let init_result = builder.try_init();

    if let Err(error) = init_result {
        eprintln!("tracing subscriber already initialized: {error}");
    }
}
