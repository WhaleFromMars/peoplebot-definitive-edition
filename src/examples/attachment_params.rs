//https://github.com/serenity-rs/poise/blob/next/examples/feature_showcase/attachment_parameter.rs
use crate::prelude::*;

register_commands!(file_details, totalsize);

#[command(prefix_command, slash_command)]
pub async fn file_details(
    ctx: Context<'_>,
    #[description = "File to examine"] file: Attachment,
    #[description = "Second file to examine"] file_2: Option<Attachment>,
) -> Result<(), Error> {
    ctx.say(format!(
        "First file name: **{}**. File size difference: **{}** bytes",
        file.filename,
        file.size - file_2.map_or(0, |f| f.size)
    ))
    .await?;
    Ok(())
}

#[command(prefix_command)]
pub async fn totalsize(
    ctx: Context<'_>,
    #[description = "Files to evaluate"] files: Vec<Attachment>,
) -> Result<(), Error> {
    let total = files.iter().map(|f| f.size as u64).sum::<u64>();

    ctx.say(format!(
        "Total file size: `{}B`. Average size: `{}B`",
        total,
        total.checked_div(files.len() as _).unwrap_or(0)
    ))
    .await?;

    Ok(())
}
