use std::{fmt, str::FromStr};

use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use strum_macros::EnumString;

use super::client::Client;

//
// Enums
//

extern crate strum;
#[derive(EnumString, Debug, Serialize, Deserialize)]
pub enum RequestType {
    #[serde(rename="join_network")]
    JoinNetwork,
    #[serde(rename="result")]
    Result,
    #[serde(rename="still_alive")]
    StillAlive,
    #[serde(rename="vibe_check")]
    VibeCheck,
    #[serde(rename="network_change")]
    NetworkChange,
}

#[derive(EnumString, Debug, Serialize, Deserialize, Clone)]
pub enum NetworkChangeType {
    #[serde(rename="exit_network")]
    ExitNetwork,
    #[serde(rename="join_network")]
    JoinNetwork,
    #[serde(rename="client_change")]
    ClientChange,
    #[serde(rename="server_shutdown")]
    ServerShutdown,
}


#[derive(Debug, Serialize, Deserialize, EnumString)]
pub enum Status {
    #[serde(rename="ok")]
    Ok,
    #[serde(rename="Error")]
    Error,
    #[serde(rename="server_error")]
    ServerError,
    #[serde(rename="wrong_client_input")]
    WrongClientInput,
    #[serde(rename="unknown_client")]
    UnknownClient,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::Ok => write!(f, "ok"),
            Status::Error => write!(f, "error"),
            Status::ServerError => write!(f, "server_error"),
            Status::WrongClientInput => write!(f, "wrong_client_input"),
            Status::UnknownClient => write!(f, "unknown_client")
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestError {
    BadJSON,
    BadType,
    CouldNotFound(&'static str)
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestError::BadJSON=> {
                write!(f, "Request wasn't a JSON or has a bad structure")
            }
            RequestError::CouldNotFound(field) => {
                write!(f, "Could not found field \"{}\" in request", field)
            }
            RequestError::BadType=> {
                write!(f, "Wrong type was used in request")
            }
        }
    }
}

pub type NetChangeMesErr = NetworkChangeMessageError;

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkChangeMessageError {
    ClientNotProvided
}

impl fmt::Display for NetworkChangeMessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NetworkChangeMessageError => {
                write!(f, "Change was about a client, but the client was not provided")
            }
        }
    }
}

//
//  MESSAGES
//
#[derive(Deserialize, Serialize, Debug)]
pub struct GenericMessage {
    #[serde(rename="type")]
    pub response_type: RequestType,
    pub status: Status,
    pub error: Option<String>
}

pub fn generic_message(message_type: RequestType, status: Status, error: Option<&str>) -> GenericMessage {
    let error = match error {
        Some(err) => err,
        None => "null",
    };
    let message = json!({
        "type": message_type,
        "status": status,
        "error": error,
    });
    let message = message.as_str().unwrap();
    let message: GenericMessage = serde_json::from_str(message).unwrap();
    message
}

pub fn result_response(status: Status, error: Option<&str>) -> GenericMessage {
    let error = match error {
        Some(err) => err,
        None => "null",
    };
    let message = json!({
        "type": RequestType::Result,
        "status": status,
        "error": error,
    });
    let message = message.as_str().unwrap();
    let message: GenericMessage = serde_json::from_str(message).unwrap();
    message
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NetworkStateMessage {
    #[serde(rename="type")]
    pub response_type: RequestType,
    pub status: Status,
    pub error: Option<String>,

    pub clients: Vec<Client>
}


pub fn network_state_response(message_type: &str, status: Status, error: Option<&str>, clients: &mut Vec<Client>) -> NetworkStateMessage {
    let error = match error {
        Some(err) => err,
        None => "null",
    };
    let message = json!({
        "type": message_type,
        "status": status,
        "error": error,
        "clients": serde_json::to_string(clients).unwrap()
    });
    let message = message.as_str().unwrap();
    let message: NetworkStateMessage = serde_json::from_str(message).unwrap();
    message
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NetworkChanges {
    #[serde(rename="type")]
    pub response_type: RequestType,
    pub status: Status,
    pub error: Option<String>,
    
    pub changes: Vec<NetworkChange>
}

impl NetworkChanges {
    pub fn new(changes: Vec<NetworkChange>) -> NetworkChanges {
        NetworkChanges {
            response_type: RequestType::NetworkChange,
            status: Status::Ok,
            error: None,
            changes
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NetworkChange {
    #[serde(rename="type")]
    pub change_type: NetworkChangeType,
    pub client: Option<Client>
}

pub fn network_change(change_type: NetworkChangeType, client: Option<&Client>) -> Result<NetworkChange, NetworkChangeMessageError> {
    let change = match change_type {
        NetworkChangeType::ClientChange => {
            let client = match client{
                Some(client) => client,
                None => return Err(NetChangeMesErr::ClientNotProvided),
            };
            json!({
                "type": NetworkChangeType::ClientChange,
                "id": client.id,
                "client_addr": client.addr,
                "client_name": client.name,
            })
        },
        NetworkChangeType::ExitNetwork => {
            let client = match client{
                Some(client) => client,
                None => return Err(NetChangeMesErr::ClientNotProvided),            
            };
            json!({
                "type": NetworkChangeType::ExitNetwork,
                "id": client.id,
            })
        },
        NetworkChangeType::JoinNetwork => {
            let client = match client{
                Some(client) => client,
                None => return Err(NetChangeMesErr::ClientNotProvided),
            };
            json!({
                "type": NetworkChangeType::JoinNetwork,
                "id": client.id,
            })
        },
        NetworkChangeType::ServerShutdown => {
            json!({
                "type": NetworkChangeType::ServerShutdown,
            })
        },
    };
    let change = serde_json::from_str(change.as_str().unwrap()).unwrap();
    Ok(change)
}


pub fn get_request_type_str(request: &str) -> Result<(RequestType, Value), RequestError> {
    let field = "type";
    println!("{}", request);
    let root: Value = serde_json::from_str(request)
        .map_err(|_| RequestError::BadJSON)?;
    let request = root.get(field);

    if let None = request {
        return Err(RequestError::CouldNotFound(field));
    }
    let request = request.unwrap().as_str().unwrap();
    let request = RequestType::from_str(request);

    if let Err(_) = request {
        return Err(RequestError::BadType);
    }

    Ok((request.unwrap(), root))
}

pub fn get_request_type_value(root: Value) -> Result<(RequestType, Value), RequestError> {
    let field = "type";
    let request = root.get(field);

    if let None = request {
        return Err(RequestError::CouldNotFound(field));
    }
    let request = request.unwrap().as_str().unwrap();
    let request = RequestType::from_str(request);

    if let Err(_) = request {
        return Err(RequestError::BadType);
    }

    Ok((request.unwrap(), root))
}
