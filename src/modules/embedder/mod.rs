use crate::prelude::*;

mod commands;

register_startup_listener!(check_deps);

async fn check_deps() -> Result<()> {
    let yt = async {
        ProcessCommand::new("yt-dlp")
            .arg("--version")
            .output()
            .await
    };
    let ff = async { ProcessCommand::new("ffmpeg").arg("-version").output().await };

    let (yt_res, ff_res) = join!(yt, ff);

    let yt_ok = matches!(yt_res, Ok(ref o) if o.status.success());
    let ff_ok = matches!(ff_res, Ok(ref o) if o.status.success());

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
