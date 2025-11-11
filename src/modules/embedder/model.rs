use std::path::PathBuf;

use crate::modules::embedder::download;
use crate::prelude::*;
use futures::StreamExt;
use tokio::{sync::Semaphore, task::JoinHandle};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

//these are envs instead of a config as they should be set by whoever hosts the bot, not guild owners.
register_env!(EMBEDDER_CONCURRENCY_LIMIT, usize);
register_env!(EMBEDDER_SIZE_LIMIT, u64);
register_env!(EMBEDDER_MAX_QUEUE, Option<usize>);
register_env!(EMBEDDER_HOME_DIR, Option<PathBuf>);
register_env!(EMBEDDER_TEMP_DIR, Option<PathBuf>);

register_global_data!(init);

pub const DEFAULT_HOME_DIR: &str = "./out";
pub const DEFAULT_TEMP_DIR: &str = "./tmp";

pub struct DownloadQueue {
    sender: MPSCSender<DownloadRequest>,
    handle: JoinHandle<()>,
    cancel: CancellationToken,
}

impl DownloadQueue {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<DownloadRequest>(
            EMBEDDER_MAX_QUEUE
                .get()
                .clone()
                .unwrap_or(Semaphore::MAX_PERMITS),
        );

        let cancel = CancellationToken::new();
        let cancel_child = cancel.clone();

        let handle = tokio::spawn(async move {
            let stream = ReceiverStream::new(receiver).take_until(cancel_child.cancelled());
            stream
                .for_each_concurrent(EMBEDDER_CONCURRENCY_LIMIT.get().clone(), |job| async move {
                    let _ = download(job).await;
                })
                .await;
        });

        Self {
            sender,
            handle,
            cancel,
        }
    }

    pub async fn enqueue(
        &self,
        job: DownloadRequest,
    ) -> Result<(), mpsc::error::SendError<DownloadRequest>> {
        self.sender.send(job).await
    }

    pub fn try_enqueue(
        &self,
        job: DownloadRequest,
    ) -> Result<(), mpsc::error::TrySendError<DownloadRequest>> {
        self.sender.try_send(job)
    }

    pub async fn shutdown(self) {
        self.cancel.cancel();
        drop(self.sender);
        let _ = self.handle.await; //wait for any tasks to finish
    }
}

pub struct EmbedderData {
    pub download_queue: DownloadQueue,
}

impl EmbedderData {
    pub fn new() -> Self {
        Self {
            download_queue: DownloadQueue::new(),
        }
    }
}

pub struct EmbedderDataKey;
impl TypeMapKey for EmbedderDataKey {
    type Value = Arc<Mutex<EmbedderData>>;
}

fn init(map: &mut TypeMap) {
    map.insert::<EmbedderDataKey>(Arc::new(Mutex::new(EmbedderData::new())));
}

#[derive(new)]
pub struct DownloadRequest {
    pub url: Url,
    pub strip_audio: bool,
    pub sender: WatchSender<YtDlpEvent>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "event")]
pub enum YtDlpEvent {
    DLStarted {
        id: String,
    },
    DLProgress {
        id: String,
        percent: String,
        eta: String,
    },
    PPStarted {
        id: String,
    },
    PPProgress {
        id: String,
        percent: String,
        eta: String,
    },
    Finished {
        id: String,
        path: String,
    },

    // Unknown/forward-compat events fall here instead of erroring
    #[serde(other)]
    Unknown,
}

pub enum YtDlpError {
    Unknown,
}
