use crate::{
    modules::embedder::model::{DownloadRequest, DownloadRequestStatus, EmbedderDataKey},
    prelude::*,
};
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

    let embedder_data = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<EmbedderDataKey>()
            .expect("Embedder data not found")
            .clone()
    };
    let (tx, mut rx) = watch::channel(DownloadRequestStatus::Idle);
    let download_request = DownloadRequest {
        url,
        strip_audio,
        tx,
    };

    {
        embedder_data
            .lock()
            .await
            .dl_queue
            .push_back(download_request);
    }

    loop {
        match &*rx.borrow() {
            DownloadRequestStatus::Finished(res) => {
                println!("finished: {res:?}");
                break;
            }
            _ => {}
        }

        if rx.changed().await.is_err() {
            //let user know that the download failed
            break;
        }
    }

    Ok(())
}
