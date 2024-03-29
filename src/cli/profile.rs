use std::error::Error;
use std::path::Path;

use clap::ArgMatches;
use tracing::{debug, error, info, warn};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::broadcast,
    sync::oneshot,
    task::LocalSet,
};

use crate::common::cli::parse_volume_map_settings;
use crate::executor::{self, ExecutorConfig};
use crate::fuzz_driver::{self, FuzzConfig};
use crate::utils::fs::read_file;

pub async fn cli(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    match args.subcommand() {
        // Adding a new task
        ("executor", Some(sub_matches)) => {
            // TODO: Fix profile
            parse_volume_map_settings(sub_matches);
            debug!("Testing executor profile");
            // Get profile
            let profile = sub_matches.value_of("file_path").unwrap();

            // Read profile
            let content = read_file(Path::new(profile)).await?;
            let content_str = String::from_utf8(content);
            assert!(content_str.is_ok());

            // Convert to json
            let config: ExecutorConfig = serde_yaml::from_str(content_str.unwrap().as_str())?;

            // Create Executor
            let mut executor = executor::new(config, None);
            let (longshot, longshot_recv) = broadcast::channel::<u8>(5);

            executor.setup().await?;

            let local_set = LocalSet::new();

            executor.spawn().await?;

            // Spawn off stdout output
            let mut stdout_reader = executor.get_stdout_reader().unwrap();
            local_set.spawn_local(async move {
                while let Ok(Some(line)) = stdout_reader.next_line().await {
                    info!("Stdout: {}", line);
                }
            });

            // Spawn off stderr output
            let mut stderr_reader = executor.get_stderr_reader().unwrap();
            local_set.spawn_local(async move {
                while let Ok(Some(line)) = stderr_reader.next_line().await {
                    warn!("Stderr: {}", line);
                }
            });

            let mut stream = signal(SignalKind::interrupt())?;
            tokio::select! {
                _ = local_set => {
                    warn!("Executor exited first, something is wrong");
                },
                _ = stream.recv() => {
                    info!("Received Ctrl-c for task set")
                },
                _ = executor.wait(longshot_recv) => {},
            }
            debug!("select! has ended, firing longshot");
            if let Err(e) = longshot.send(0) {
                error!("Error sending longshot: {:?}", e);
            }
            executor.close().await?;
        }
        ("task", Some(sub_matches)) => {
            parse_volume_map_settings(sub_matches);
            debug!("Testing fuzz driver profile");

            let local = LocalSet::new();

            // Get profile
            let profile = sub_matches.value_of("file_path").unwrap();

            // Read profile
            let content = read_file(Path::new(profile)).await?;
            let content_str = String::from_utf8(content);
            assert!(content_str.is_ok());

            // Convert to json
            let config: FuzzConfig = serde_yaml::from_str(content_str.unwrap().as_str())?;

            let mut driver = fuzz_driver::new(config, None);

            // Fake tx, will not be used
            let (tx, rx) = oneshot::channel::<u8>();
            let (death_tx, _) = oneshot::channel::<u8>();

            let mut stream = signal(SignalKind::interrupt())?;
            tokio::select! {
                result = driver.start(rx, death_tx) => {
                    error!("Fuzz driver exited first, something is wrong");
                    if let Err(e) = result {
                        error!("Cause: {}", e);
                    }
                },
                _ = stream.recv() => {
                    info!("Received Ctrl-c for task set");
                    let _ = tx.send(0);
                },
            }

            local.await;
        }
        // Listing all tasks
        _ => {}
    }

    Ok(())
}
