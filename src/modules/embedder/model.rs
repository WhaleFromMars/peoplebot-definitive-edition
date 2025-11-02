use std::sync::atomic::AtomicBool;

use crate::prelude::*;
use poise::serenity_prelude::prelude::{TypeMap, TypeMapKey};
use url::Url;

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
    pub response: DownloadResponse,
}

#[derive(Default)]
pub struct DownloadResponse {
    pub sender: Option<UserId>,
    pub prev_response: Option<MessageId>,
}
