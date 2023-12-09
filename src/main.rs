mod comms;
mod command_args;

use command_args::parse_commands;
use command_args::set_commands;
use comms::start_server;

#[tokio::main]
async fn main(){

    // Get command line arguments and parse them
    let matched_commands = set_commands();
    let parsed_commands = parse_commands(matched_commands);

    println!("Starting server");
    start_server(parsed_commands).await.unwrap();
}
