use clap::{arg, command, Arg, ArgMatches};

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
        )
        .get_matches();

    match_result
}

pub fn parse_commands(matches: ArgMatches) -> ParsedCommands {
    let mut port: u16 = 10002; //Default port
    if let Some(command_port) = matches.get_one::<u16>("port") {
        port = *command_port;
    }
    ParsedCommands {
        port,
    }
}


pub struct ParsedCommands {
    pub port: u16,
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