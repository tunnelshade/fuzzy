use std::error::Error;
use std::str::FromStr;

use log::debug;
use clap::ArgMatches;
use tonic::Request;

use crate::models::NewTask;
use crate::xpc::orchestrator_client::OrchestratorClient;

pub async fn cli(args: &ArgMatches, connect_addr: String) -> Result<(), Box<dyn Error>> {
    debug!("Creating interface client");
    let mut client = OrchestratorClient::connect(connect_addr).await?;

    match args.subcommand() {
        // Adding a new task
        ("add", Some(sub_matches)) => {
            debug!("Adding a new task");
            let new_task = NewTask {
                name: sub_matches.value_of("name").unwrap().to_owned(),
                active: false,
            };
            // Validate executor & driver as we do crude transforms via enums & strum
            let response = client.submit_task(Request::new(new_task)).await?;
            // TODO: Error handling
            println!("{:?}", response);
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