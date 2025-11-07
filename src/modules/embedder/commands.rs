use crate::{
    modules::embedder::model::{DownloadRequest, EmbedderDataKey, YtDlpEvent},
    prelude::*,
};
use poise::{CreateReply, ReplyHandle};
use tokio::sync::watch;
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
        let event = receiver.borrow_and_update().clone(); //clone until I can be bothered to see if we can use references
        match event {
            YtDlpEvent::DLStarted { id } => {
                handle = edit_or_send_new(&ctx, handle, format!("Downloading..."))
                    .await
                    .ok();
            }
            YtDlpEvent::DLProgress { id, percent, eta } => {
                handle = edit_or_send_new(
                    &ctx,
                    handle,
                    format!("Downloading... {}%, {}", percent, eta),
                )
                .await
                .ok();
            }
            YtDlpEvent::PPStarted { id } => {
                handle = edit_or_send_new(&ctx, handle, format!("Processing..."))
                    .await
                    .ok();
            }
            YtDlpEvent::PPProgress { id, percent, eta } => {
                handle =
                    edit_or_send_new(&ctx, handle, format!("Processing... {}%, {}", percent, eta))
                        .await
                        .ok();
            }
            YtDlpEvent::Finished { id, path } => {
                let attachment = CreateAttachment::path(path).await?; //can fail to open the file, but not likely
                let message = CreateMessage::new()
                    .content(format!("[original link](<{}>)", original_url))
                    .add_file(attachment);
                ctx.channel_id()
                    .send_message(&ctx.http(), message)
                    .await
                    .ok();
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
