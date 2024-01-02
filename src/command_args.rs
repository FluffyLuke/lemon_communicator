use core::time;
use std::time::Duration;

use clap::{command, Arg, ArgMatches};

pub fn set_commands() -> ArgMatches {

    let app_name = "Lemon Communicator";
    let version = "0.0.1";
    let about = "Simple communicator";

    let match_result = command!()
        .name(app_name)
        .version(version)
        .about(about)
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_parser(clap::value_parser!(u16))
        )
        .arg(
            Arg::new("update_interval")
                .short('u')
                .long("update_interval")
                .value_parser(clap::value_parser!(u64))
        )
        .arg(
            Arg::new("vibe_check_interval")
                .short('v')
                .long("vibe_check_interval")
                .value_parser(clap::value_parser!(u64))
        )
        .get_matches();

    match_result
}

pub fn parse_commands(matches: ArgMatches) -> ParsedArgs {
    let mut port: u16 = 10002; //Default port
    let mut update_client_interval = time::Duration::from_secs(15);
    let mut vibe_check_interval = time::Duration::from_secs(15);

    if let Some(value) = matches.get_one::<u16>("port") {
        port = *value;
    }
    if let Some(value) = matches.get_one::<u64>("update_interval") {
        update_client_interval = time::Duration::from_secs(*value);
    }
    if let Some(value) = matches.get_one::<u64>("update_interval") {
        vibe_check_interval = time::Duration::from_secs(*value);
    }
    
    ParsedArgs{
        port,
        update_client_interval,
        vibe_check_interval,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParsedArgs {
    pub port: u16,
    pub update_client_interval: Duration,
    pub vibe_check_interval: Duration,
}
// pub struct InitVars {
//     app_name: String,
//     version: String,
// }

// impl InitVars {
//     pub fn new(app_name: String, version: String) -> InitVars {
//         return InitVars {
//             app_name,
//             version
//         }
//     }
// }