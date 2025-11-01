use crate::prelude::*;

register_commands!(embed);

#[command(slash_command, prefix_command)]
pub async fn embed(
    _ctx: Context<'_>,
    link: String,
    anonymous: Option<bool>,
    strip_audio: Option<bool>,
) -> Result<()> {
    Ok(())
}
