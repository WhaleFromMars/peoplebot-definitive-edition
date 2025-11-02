use crate::prelude::*;

#[command(slash_command, prefix_command)]
pub async fn source(_ctx: Context<'_>) -> Result<()> {
    Ok(())
}
