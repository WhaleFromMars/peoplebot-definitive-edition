use std::path::PathBuf;

use crate::{
    modules::embedder::model::{DownloadRequest, EmbedderDataKey, YtDlpEvent},
    prelude::*,
};
use poise::{CreateReply, ReplyHandle};
use tokio::{
    fs::{self, remove_file},
    sync::watch,
};
use url::Url;

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
                handle = ctx
                    .reply(format!(
                        "Failed to queue download, server might be overloaded"
                    ))
                    .await
                    .ok();
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
                    // this should be a new message so we can ping them with a fail
                    // or return as an error variant and use the global on error handled to match the variant and handle it there
                    handle = edit_or_send_new(
                        &ctx,
                        handle,
                        format!(
                            "File too large, server limit is {} MB, file size is {} MB",
                            guild_limit / 1024 / 1024,
                            file_size.len() / 1024 / 1024
                        ),
                    )
                    .await
                    .ok();
                    break;
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
                if let Some(handle) = handle {
                    //if we have a handle, try delete the message
                    handle.delete(ctx).await.ok(); //ignore error, nothing we can do
                }

                if let Err(_) = remove_file(&path).await {
                    error!("Failed to remove file: {path}");
                }
                break;
            }
            _ => { /*discard other events*/ }
        }
    }

    Ok(())
}

/// Edit an existing message or send a new one if the handle has expired
/// Will only return an error if a new message cannot be sent
async fn edit_or_send_new<'a>(
    ctx: &Context<'a>,
    handle: Option<ReplyHandle<'a>>,
    content: impl Into<String>,
) -> Result<ReplyHandle<'a>> {
    let content = content.into();

    if let Some(handle) = handle {
        match handle
            .edit(*ctx, CreateReply::new().content(&content))
            .await
        {
            Ok(_) => Ok(handle),
            Err(_) => {
                // The handle has expired; send a new message.
                let handle = ctx.say(&content).await?;
                Ok(handle)
            }
        }
    } else {
        // No existing handle; just send a new message.
        let handle = ctx.say(&content).await?;
        Ok(handle)
    }
}
