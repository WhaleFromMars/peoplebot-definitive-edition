use crate::modules::embedder::model::{DownloadRequest, YtDlpEvent};
use crate::{modules::embedder::model::EmbedderDataKey, prelude::*};
use std::{path::PathBuf, process::Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};

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
        FullEvent::CacheReady { .. } => {}
        _ => {}
    }
    Ok(())
}

pub const BASE_ARGS: &[&str] = &[
    "--no-sponsorblock", // cleaner output
    "--newline",         // one event per line
    "--no-warnings",     // less noise
    "--progress",        // ensure progress ticks
    "--progress-delta",
    "1", //only report progress changes every 1 second
    //start & progress events for downloading and post-processing
    "--print",
    r#"before_dl:{"event":"dl_started","id":"%(id)s"}"#,
    "--progress-template",
    r#"download:{"event":"dl_progress","id":"%(info.id)s","percent":"%(progress._percent_str)s","eta":"%(progress._eta_str)s"}"#,
    "--print",
    r#"post_process:{"event":"pp_started","id":"%(id)s"}"#,
    "--progress-template",
    r#"postprocess:{"event":"pp_progress","id":"%(info.id)s","percent":"%(progress._percent_str)s","eta":"%(progress._eta_str)s"}"#,
    //Completion event
    "--print",
    r#"after_move:{"event":"moved","id":"%(id)s","path":%(filepath)j}"#,
    //Paths & formats
    "-P",
    "home:./out",
    "-P",
    "temp:./tmp",
    "-f",
    "bv*+ba/b",
];

async fn download(request: DownloadRequest) -> Result<()> {
    debug!("Downloading {}", request.url);
    let DownloadRequest {
        url,
        strip_audio,
        sender,
    } = request;

    let mut cmd = ProcessCommand::new("yt-dlp");
    cmd.arg(url.to_string())
        .args(BASE_ARGS)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take().expect("piped stdout");
    // let stderr = child.stderr.take().expect("piped stderr");
    let mut out_lines = BufReader::new(stdout).lines();
    // let mut err_lines = BufReader::new(stderr).lines();

    while let Some(line) = out_lines.next_line().await? {
        debug!("yt-dlp output: {}", line);
        if let Ok(event) = serde_json::from_str::<YtDlpEvent>(&line) {
            debug!("yt-dlp event: {:?}", event);
            match event {
                YtDlpEvent::Finished { id, path } => {
                    let _ = sender.send(YtDlpEvent::Finished { id, path });
                    break;
                }
                other => {
                    let _ = sender.send(other);
                }
            }
        }
    }

    Ok(())
}
