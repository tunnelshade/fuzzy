use std::path::{Path, PathBuf};
use std::error::Error;
use std::collections::HashMap;

use regex::Regex;
use log::debug;
use serde::{Serialize, Deserialize};
// use serde_regex::{Serialize, Deserialize};
use tokio::{
    process::{ChildStdout, ChildStderr},
    io::{BufReader, Lines},
    fs::File,
};

use corpus_syncer::CorpusSyncer;
use crash_syncer::CrashSyncer;

// Both of filesystem variants, need to change
pub mod corpus_syncer;
pub mod crash_syncer;
mod native;

#[derive(Debug, Clone)]
pub struct CrashConfig {
    pub path: Box<Path>,
    pub label: String,
    pub filter: Regex,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExecutorEnum {
    Native,
    Docker,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CorpusConfig {
    pub path: Box<Path>,
    pub label: String,
    pub refresh_interval: u64,
    pub upload: bool,

    #[serde(with = "serde_regex")]
    pub upload_filter: Regex,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutorConfig {
    pub executor: ExecutorEnum,
    pub cpus: i32,
    pub executable: String,
    pub args: Vec<String>,
    pub cwd: Box<Path>,
    pub corpus: CorpusConfig,
    pub envs: HashMap<String, String>,
}

#[tonic::async_trait]
pub trait Executor {
    /// Create a new executor with this configuration
    fn new(config: ExecutorConfig, worker_task_id: Option<i32>) -> Self;

    /// Setup stage often involves preparing things like download
    /// corpus, make it ready for launch
    async fn setup(&self) -> Result<(), Box<dyn Error>>;

    /// Actually responsible for launching of the process
    async fn spawn(&mut self) -> Result<(), Box<dyn Error>>;

    // TODO: Improve these ChildStdout signatures to support other executors
    /// Get stdout reader
    fn get_stdout_reader(&mut self) -> Option<Lines<BufReader<ChildStdout>>>;
    /// Get stderr reader
    fn get_stderr_reader(&mut self) -> Option<Lines<BufReader<ChildStderr>>>;

    // TODO: Switch to generic trait based returns so we can swap file monitors
    // fn get_file_watcher(&self, path: Path) -> Box<dyn file_watcher::FileWatcher>;
    async fn get_corpus_syncer(&self) -> Result<CorpusSyncer, Box<dyn Error>>;
    async fn get_crash_syncer(&self, config: CrashConfig) -> Result<CrashSyncer, Box<dyn Error>>;

    // Get absolute path for relative to cwd
    fn get_cwd_path(&self) -> PathBuf;

    // Clean up all spawned children
    fn close(&mut self) -> Result<(), Box<dyn Error>>;
}

pub fn new(config: ExecutorConfig, worker_task_id: Option<i32>) -> impl Executor {
    match config.executor {
        _ => {
            debug!("Creating native executor");
            native::NativeExecutor::new(config, worker_task_id)
        },
    }
}
