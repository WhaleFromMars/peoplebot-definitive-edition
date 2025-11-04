use crate::{
    modules::embedder::model::{EmbedderData, EmbedderDataKey},
    prelude::*,
};

mod commands;
mod model;

register_startup_listener!(check_deps);
register_event_listener!(event_listener);

async fn check_deps() -> Result<()> {
    let yt = async {
        ProcessCommand::new("yt-dlp")
            .arg("--version")
            .output()
            .await
    };
    let ff = async { ProcessCommand::new("ffmpeg").arg("-version").output().await };

    let (yt_res, ff_res) = join!(yt, ff);

    let yt_ok = yt_res.is_ok_and(|o| o.status.success());
    let ff_ok = ff_res.is_ok_and(|o| o.status.success());

    if yt_ok && ff_ok {
        Ok(())
    } else {
        let mut missing = Vec::new();
        if !yt_ok {
            missing.push("yt-dlp");
        }
        if !ff_ok {
            missing.push("ffmpeg");
        }
        bail!("missing deps: {}", missing.join(", "));
    }
}

async fn event_listener(
    ctx: FrameworkContext<'_, GlobalState, Error>,
    event: &FullEvent,
) -> Result<()> {
    let embedder_data = {
        let data = ctx.serenity_context.data.read().await;
        data.get::<EmbedderDataKey>()
            .expect("Embedder data not found")
            .clone()
    };
    match event {
        FullEvent::CacheReady { .. } => {
            let embedder_data = embedder_data;
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    process_download_queue(&*embedder_data).await;
                }
            });
        }
        _ => {}
    }
    Ok(())
}

async fn process_download_queue(data: &Mutex<EmbedderData>) {}
