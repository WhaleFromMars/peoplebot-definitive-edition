use std::{path::PathBuf, sync::atomic::AtomicBool};

use crate::prelude::*;
use poise::serenity_prelude::prelude::{TypeMap, TypeMapKey};
use tokio::sync::watch::{Receiver, Sender};
use url::Url;

//these are envs as they should be set by whoever hosts the bot, not guild owners.
register_env!(EMBEDDER_CONCURRENCY_LIMIT, u8);
register_env!(EMBEDDER_SIZE_LIMIT, u64);

pub struct EmbedderData {
    pub dl_is_running: AtomicBool,
    pub dl_queue: VecDeque<DownloadRequest>,
}

pub struct EmbedderDataKey;
impl TypeMapKey for EmbedderDataKey {
    type Value = Arc<Mutex<EmbedderData>>;
}

register_global_data!(init);

fn init(map: &mut TypeMap) {
    map.insert::<EmbedderDataKey>(Arc::new(Mutex::new(EmbedderData {
        dl_is_running: AtomicBool::new(false),
        dl_queue: VecDeque::new(),
    })));
}

#[derive(new)]
pub struct DownloadRequest {
    pub url: Url,
    pub strip_audio: bool,
    pub tx: Sender<DownloadRequestStatus>,
}

pub enum DownloadRequestStatus {
    Idle,
    QueuePositionChanged(usize),
    Progress(usize),
    Finished(Result<PathBuf>),
}
