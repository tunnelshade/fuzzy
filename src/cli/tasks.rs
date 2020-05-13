use std::path::Path;
use std::error::Error;

use log::{info, debug};
use clap::ArgMatches;
use tonic::Request;

use crate::models::{NewTask, PatchTask};
use crate::common::profiles::construct_profile_from_disk;
use crate::common::xpc::get_orchestrator_client;

pub async fn cli(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
    debug!("Creating interface client");
    let mut client = get_orchestrator_client().await?;

    match args.subcommand() {
        // Adding a new task
        ("add", Some(sub_matches)) => {
            debug!("Adding a new task");
            let profile_path = sub_matches.value_of("profile_path").unwrap();

            let profile = construct_profile_from_disk(Path::new(profile_path)).await?;

            let new_task = NewTask {
                name: sub_matches.value_of("name").unwrap().to_owned(),
                active: true,
                profile: serde_json::to_string(&profile)?,
            };

            // Validate executor & driver as we do crude transforms via enums & strum
            let _ = client.submit_task(Request::new(new_task)).await?;
            // TODO: Error handling
            info!("Successfilly added task");
        },
        ("edit", Some(sub_matches)) => {
            debug!("Editing a task");
            let profile_path = sub_matches.value_of("profile_path");

            let mut profile: Option<String> = None;
            if let Some(profile_path) = profile_path {
                let config = construct_profile_from_disk(Path::new(profile_path)).await?;
                profile = Some(serde_json::to_string(&config)?);
            }

            let name = sub_matches.value_of("name").map(|s| s.to_owned());
            let active = sub_matches.is_present("active");
            let id = sub_matches.value_of("id").expect("No ID provided").parse::<i32>()?;

            let patch_task = PatchTask {
                id,
                name,
                active,
                profile,
            };

            // Validate executor & driver as we do crude transforms via enums & strum
            let _ = client.update_task(Request::new(patch_task)).await?;
            info!("Updated task successfully");
        },
        // Listing all tasks
        ("list", Some(_)) => {
            debug!("Listing all tasks");

            let response = client.get_tasks(Request::new({})).await?;
            let tasks = response.into_inner().data;

            let tasks_heading = vec!["ID", "Name", "Active"];
            let mut tasks_vec = Vec::new();
            for t in tasks.iter() {
                tasks_vec.push(super::formatter::format_task(t));
            }

            super::print_results(tasks_heading, tasks_vec);
        },
        _ => {},
    }

    Ok(())
}
