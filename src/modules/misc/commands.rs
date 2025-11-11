use poise::CreateReply;

use crate::prelude::*;

register_commands!(source);

#[command(slash_command, prefix_command)]
pub async fn source(ctx: Context<'_>) -> Result<()> {
    ctx.send(
        CreateReply::new()
            .content("<https://github.com/WhaleFromMars/peoplebot-definitive-edition>")
            .ephemeral(true),
    )
    .await?;
    Ok(())
}
