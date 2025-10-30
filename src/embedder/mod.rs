use crate::{prelude::*, register_startup_listener};

register_commands!(embed);
register_startup_listener!(check_deps);

pub async fn check_deps() -> Result<(), Error> {
    info!("checking dependencies");
    Ok(())
}

#[command(slash_command)]
pub async fn embed(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
