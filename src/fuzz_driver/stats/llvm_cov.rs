use std::time::SystemTime;
use std::path::PathBuf;
use std::error::Error;

use tracing::{debug, error};
use tokio::fs::read_dir;
use tokio::stream::StreamExt;

use crate::executor;
use crate::models::NewFuzzStat;
use super::{FuzzStatCollector, FuzzStatConfig};
use crate::common::xpc::get_orchestrator_client;
use crate::common::corpora::download_corpus_to_disk;
use crate::utils::err_output;

#[derive(Clone)]
pub struct LlvmCovCollector {
    config: FuzzStatConfig,
    worker_task_id: Option<i32>,
    corpus_label: String,
    last_sync: SystemTime,
}

impl LlvmCovCollector {
    fn new(config: FuzzStatConfig, corpus_label: String, worker_task_id: Option<i32>) -> Self {
        Self {
            config,
            worker_task_id,
            corpus_label,
            last_sync: SystemTime::now(),
        }
    }
}

#[tonic::async_trait]
impl FuzzStatCollector for LlvmCovCollector {
    async fn get_stat(&self) -> Result<Option<NewFuzzStat>, Box<dyn Error>> {
        debug!("Getting new stat using llvm-cov collector");
        let mut client = get_orchestrator_client().await?;

        // Create an executor
        let mut executor = executor::new(self.config.execution.clone(), self.worker_task_id);

        // Get latest corpus to cwd
        executor.setup().await?;

        // Download latest corpus found by this worker
        let cwd = executor.get_cwd_path();
        let num_files = download_corpus_to_disk(
            self.corpus_label.clone(),
            None,
            self.worker_task_id,
            Some(10),
            self.last_sync,
            cwd.as_path(),
            &mut client,
        ).await?;
        debug!("{} corpus downloaded for stat collection", num_files);

        let output = executor.spawn_blocking().await?;
        if output.status.success() == false {
            error!("Stat collection execution failed");
            err_output(output);
        }

        // We look for .json files anyway
        let entries = read_dir(cwd.as_path()).await?;
        let json_files = entries.filter_map(|f| {
            if let Ok(file) = f {
                let path = file.path();
                let extension = path.extension();
                if extension.is_some() && extension.unwrap() == "json" {
                    return Some(path)
                }
            }
            None
        });
        let llvm_json: Vec<PathBuf> = json_files.collect::<Vec<PathBuf>>().await;

        for file in llvm_json {
        }
        Ok(None)
    }
}
