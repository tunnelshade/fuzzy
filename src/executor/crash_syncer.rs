use std::error::Error;

use log::{info, error, debug};
use tonic::transport::channel::Channel;
use tokio::sync::broadcast;

use crate::xpc::orchestrator_client::OrchestratorClient;
use crate::utils::fs::InotifyFileWatcher;
use crate::fuzz_driver::CrashConfig;
use crate::common::crashes::upload_crash_from_disk;
use crate::common::xpc::get_orchestrator_client;

/// A file system corpus syncer. Need to convert this into trait when implementing docker
pub struct CrashSyncer {
    config: CrashConfig,
    worker_task_id: Option<i32>,
}

impl CrashSyncer {
    pub fn new(config: CrashConfig, worker_task_id: Option<i32>) -> Result<Self, Box<dyn Error>> {
        Ok(Self { config, worker_task_id })
    }

    pub async fn upload_crashes(
            &self,
            mut kill_switch: broadcast::Receiver<u8>
        ) -> Result<(), Box<dyn Error>> {

        debug!("Will try to keep crashes in sync at: {:?}", self.config.path);
        let client = get_orchestrator_client().await?;
        let worker_task_id = self.worker_task_id;

        // Create necessary clones and pass along for upload sync if upload enabled
        let config = self.config.clone();
        tokio::select! {
            result = upload(config, worker_task_id, client) => {
                error!("Crash upload sync job failed: {:?}", result);
            },
            _ = kill_switch.recv() => {}
        }

        // crash_sync_handle.await?;

        Ok(())
    }
}

async fn upload(
        config: CrashConfig,
        worker_task_id: Option<i32>,
        client: OrchestratorClient<Channel>) -> Result<(), Box<dyn Error>> {
    let mut client = client;
    info!("Creating crash upload sync");
    let mut watcher = InotifyFileWatcher::new(&config.path, Some(config.filter))?;

    while let Some(file) = watcher.get_new_file().await {
        // Match user provided match pattern
        let file_path = config.path.clone();
        let file_path = file_path.join(file);
        info!("Uploading new crash: {:?}", file_path);
        upload_crash_from_disk(file_path.as_path(), config.label.clone(), worker_task_id, &mut client).await?
    }
    Ok(())
}
