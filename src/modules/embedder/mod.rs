use crate::modules::embedder::model::*;
use crate::prelude::*;
use anyhow::Context;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncBufReadExt, BufReader};

mod commands;
mod model;

register_startup_listener!(check_deps);
register_startup_listener!(validate_storage_paths);

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
];

const FORMAT_ARGS: &[&str] = &["-f", "bv*+ba/b"];

pub fn yt_dlp_storage_args() -> (String, String) {
    let home = EMBEDDER_HOME_DIR
        .get()
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_HOME_DIR));
    let temp = EMBEDDER_TEMP_DIR
        .get()
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_TEMP_DIR));

    let home_arg = format!("home:{}", home.to_string_lossy());
    let temp_arg = format!("temp:{}", temp.to_string_lossy());
    (home_arg, temp_arg)
}

async fn download(request: DownloadRequest) -> Result<()> {
    debug!("Downloading {}", request.url);
    let DownloadRequest {
        url,
        strip_audio,
        sender,
    } = request;

    let mut cmd = ProcessCommand::new("yt-dlp");
    let (home_arg, temp_arg) = yt_dlp_storage_args();
    cmd.arg(url.to_string())
        .args(BASE_ARGS)
        .arg("-P")
        .arg(home_arg)
        .arg("-P")
        .arg(temp_arg)
        .args(FORMAT_ARGS)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");

    let mut out_lines = BufReader::new(stdout).lines();
    let mut err_lines = BufReader::new(stderr).lines();

    let mut stderr_closed = false;

    loop {
        tokio::select! {
            //select will swap between handling stdout_line = {} and stderr_line = {}
            // pseudorandomly if both have a line waiting, otherwise handles the one that has a line waiting
            stdout_line = out_lines.next_line() => {
                let Some(line) = stdout_line? else {
                    // command finished
                    break;
                };

                debug!("yt-dlp stdout: {}", line);

                if let Ok(event) = serde_json::from_str::<YtDlpEvent>(&line) {
                    debug!("yt-dlp event: {:?}", event);
                    match event {
                        YtDlpEvent::Finished { id, path } => {
                            let _ = sender.send(YtDlpEvent::Finished { id, path });
                            break; // match this variant to break early
                        }
                        other => {
                            let _ = sender.send(other);
                            // send all others for them to handle
                        }
                    }
                }
            }

            stderr_line = err_lines.next_line(), if !stderr_closed => {
                match stderr_line {
                    Ok(Some(line)) => {
                        debug!("yt-dlp stderr: {}", line);
                    }
                    Ok(None) => {
                        stderr_closed = true;
                    }
                    Err(e) => {
                        debug!("error reading yt-dlp stderr: {}", e);
                        stderr_closed = true;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn validate_storage_paths() -> Result<()> {
    let home_dir = EMBEDDER_HOME_DIR
        .get()
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_HOME_DIR));
    let temp_dir = EMBEDDER_TEMP_DIR
        .get()
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_TEMP_DIR));
    ensure_dir_writable("home", &home_dir).await?;
    ensure_dir_writable("temp", &temp_dir).await?;
    Ok(())
}

async fn ensure_dir_writable(label: &str, path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .await
        .with_context(|| format!("failed to create {label} directory at {}", path.display()))?;

    let test_path = path.join(".peoplebot-write-test");
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&test_path)
        .await
        .with_context(|| format!("cannot write to {label} directory at {}", path.display()))?;

    let _ = fs::remove_file(&test_path).await;
    Ok(())
}
