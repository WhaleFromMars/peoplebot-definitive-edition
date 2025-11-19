use crate::{modules::embedder::model::*, prelude::*};
use tokio::fs;

register_commands!(embed);

#[command(slash_command, prefix_command)]
pub async fn embed(
    ctx: Context<'_>,
    link: String,
    #[description = "Whether to embed the link anonymously"] anonymous: Option<bool>,
    #[description = "Whether to strip audio from the video"] strip_audio: Option<bool>,
) -> Result<()> {
    ctx.defer_ephemeral().await?; //defer gives us 15m to reply before it ends the interaction

    let anonymous = anonymous.unwrap_or(false);
    let strip_audio = strip_audio.unwrap_or(false);
    let url = Url::parse(&link)?;

    let original_url = url.clone(); //a copy we can use later
    let name = if anonymous {
        "anon".to_string()
    } else {
        ctx.author().mention().to_string()
    };

    let embedder_data = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<EmbedderDataKey>()
            .expect("Embedder data not found")
            .clone()
    };

    let (sender, mut receiver) = watch::channel(YtDlpEvent::Unknown);
    let request = DownloadRequest {
        url,
        strip_audio,
        sender,
    };

    #[allow(unused_assignments)] //it doesnt see it gets used in edit_or_send_new
    let mut handle: Option<ReplyHandle> = None;

    {
        let data = embedder_data.lock().await;
        match data.download_queue.try_enqueue(request) {
            Ok(_) => {
                handle = ctx.reply(format!("Awaiting Download...")).await.ok();
            }
            Err(_) => {
                bail_to_user!("Failed to queue download, server might be overloaded");
            }
        }
    }

    while receiver.changed().await.is_ok() {
        let event = receiver.borrow_and_update().clone(); //can be done without a clone but its so messy, minor perf is negligible
        match event {
            YtDlpEvent::DLStarted { .. } => {
                // the .. ignores any remaining fields that we dont care for
                handle = edit_or_send_new(&ctx, handle, format!("Downloading..."))
                    .await
                    .ok();
            }
            YtDlpEvent::DLProgress { percent, .. } => {
                handle = edit_or_send_new(&ctx, handle, format!("Downloading... {}", percent))
                    .await
                    .ok();
            }
            YtDlpEvent::PPStarted { .. } => {
                handle = edit_or_send_new(&ctx, handle, format!("Processing..."))
                    .await
                    .ok();
            }
            YtDlpEvent::PPProgress { percent, .. } => {
                handle = edit_or_send_new(&ctx, handle, format!("Processing... {}", percent))
                    .await
                    .ok();
            }
            YtDlpEvent::Finished { path, .. } => {
                let file_size = fs::metadata(&path).await?;
                let guild_limit = attachment_byte_limit(&ctx, ctx.guild_id());
                if file_size.len() > guild_limit {
                    fs::remove_file(&path).await.ok();
                    handle.delete(ctx).await.ok();

                    let reply = CreateMessage::new() //dont use <> to allow it to embed if provider supports it, as we failed to
                        .content(format!("-# sent by: {name} - [[link]]({original_url})"));
                    ctx.channel_id().send_message(&ctx.http(), reply).await.ok();

                    bail_to_user!(
                        "File for [[link]](<{original_url}>) too large to embed, server limit is {}, file size is {}, sent link instead",
                        format_bytes(guild_limit),
                        format_bytes(file_size.len())
                    );
                }
                let attachment = CreateAttachment::path(&path).await?; //can fail to open the file, but not likely

                let message = CreateMessage::new()
                    .content(format!("-# sent by: {name} - [[link]](<{original_url}>)"))
                    .add_file(attachment);

                ctx.channel_id()
                    .send_message(&ctx.http(), message)
                    .await
                    .ok(); //consume potential error
                //theres nothing we can do if it fails to send, and we want to make sure to delete the file afterwards
                handle.delete(ctx).await.ok();

                if let Err(_) = fs::remove_file(&path).await {
                    error!("Failed to remove file: {path}"); //we dont bail because the core logic still succeeded
                }
                break;
            }
            _ => { /*discard other events*/ }
        }
    }

    Ok(())
}
