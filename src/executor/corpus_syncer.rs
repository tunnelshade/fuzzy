use std::error::Error;
use std::time::{UNIX_EPOCH, SystemTime, Duration};

use regex::Regex;
use log::{info, error, debug};
use tonic::transport::channel::Channel;
use tokio::sync::broadcast;

use crate::xpc::orchestrator_client::OrchestratorClient;
use crate::utils::fs::InotifyFileWatcher;
use super::CorpusConfig;
use crate::common::corpora::{upload_corpus_from_disk, download_corpus_to_disk, CORPUS_FILE_EXT};
use crate::common::xpc::get_orchestrator_client;

/// A file system corpus syncer. Need to convert this into trait when implementing docker
pub struct CorpusSyncer {
    config: CorpusConfig,
    worker_task_id: Option<i32>,
}

impl CorpusSyncer {
    pub fn new(config: CorpusConfig, worker_task_id: Option<i32>) -> Result<Self, Box<dyn Error>> {
        Ok(Self { config, worker_task_id })
    }

    pub async fn setup_corpus(&self) -> Result<(), Box<dyn Error>> {
        debug!("Syncing initial corpus");
        let mut client = get_orchestrator_client().await?;
        download_corpus_to_disk(
            self.config.label.clone(),
            self.worker_task_id,
            UNIX_EPOCH,
            &self.config.path,
            &mut client).await?;
        Ok(())
    }

    pub async fn sync_corpus(
            &self,
            mut kill_switch: broadcast::Receiver<u8>
        ) -> Result<(), Box<dyn Error>> {

        debug!("Will try to keep corpus in sync at: {:?}", self.config.path);
        let client = get_orchestrator_client().await?;
        let worker_task_id = self.worker_task_id;

        // Create a local set
        // let local_set = task::LocalSet::new();

        // Create necessary clones and pass along for upload sync if upload enabled
        let client_clone = client.clone();
        let corpus_config_upload = self.config.clone();

        // Create necessary clones and pass along for download sync
        let last_updated = SystemTime::now();
        let corpus_config = self.config.clone();
        let refresh_interval = self.config.refresh_interval;

        tokio::select! {
            _ = download(corpus_config, worker_task_id, last_updated, refresh_interval, client) => {
                error!("Downloading corpus exited first, whaaatt!");
            },
            // Doing this should be very necessary
            _ = upload(corpus_config_upload, worker_task_id, client_clone), if self.config.upload => {
                error!("Uploading corpus exited first, whaaatt!");
            },
            _ = kill_switch.recv() => {
                debug!("Kill receieved for corpus sync at {:?}", self.config);
            },
        }

        // Error handled at spawn level
        // let (_, _) = tokio::join!(upload_handle, download_handle);

        // local_set.await;

        Ok(())
    }
}

async fn upload(
        corpus: CorpusConfig,
        worker_task_id: Option<i32>,
        client: OrchestratorClient<Channel>) -> Result<(), Box<dyn Error>> {
    if corpus.upload == true {
        let mut client = client;
        info!("Creating corpus upload sync");
        let ext_regex = Regex::new(format!(".*\\.{}$", CORPUS_FILE_EXT).as_str()).unwrap();
        let mut watcher = InotifyFileWatcher::new(&corpus.path, Some(corpus.upload_filter))?;

        while let Some(file) = watcher.get_new_file().await {
            // Match user provided match pattern
            if ext_regex.is_match(file.as_str()) == false {
                let file_path = corpus.path.clone();
                let file_path = file_path.join(file);
                info!("Uploading new corpus: {:?}", file_path);
                upload_corpus_from_disk(file_path.as_path(), corpus.label.clone(), worker_task_id, &mut client).await?
            } else {
                debug!("Skipping upload of a user unmatched pattern: {:?}", file);
            }
        }
    } else {
        debug!("Returning early as corpus upload seems to have been disabled for {:?}", worker_task_id);
    }
    Ok(())
}

async fn download(
        corpus_config: CorpusConfig,
        worker_task_id: Option<i32>,
        mut last_updated: SystemTime,
        refresh_interval: u64,
        mut client: OrchestratorClient<Channel>) -> Result<(), Box<dyn Error>> {
    let mut interval = tokio::time::interval(Duration::from_secs(refresh_interval));
    loop {
        interval.tick().await;
        let result = download_corpus_to_disk(corpus_config.label.clone(),
                                             worker_task_id,
                                             last_updated,
                                             &corpus_config.path,
                                             &mut client).await;
        // If successful update, set last_updated
        if let Err(e) = result {
            error!("Download sync job failed: {}", e);
        } else {
            last_updated = SystemTime::now();
        }
    }
}
