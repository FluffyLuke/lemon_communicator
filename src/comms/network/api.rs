use std::{fmt, str::FromStr};

use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use strum_macros::EnumString;

use crate::comms::client::{Client, WeakClient, GetWeak};

// Used to describe the purpose of a message
#[derive(EnumString, Debug, Serialize, Deserialize)]
pub enum MessageType {
    #[strum(serialize = "join_network")]
    #[serde(rename="join_network")]
    JoinNetwork,

    #[strum(serialize = "result")]
    #[serde(rename="result")]
    Result,

    #[strum(serialize = "still_alive")]
    #[serde(rename="still_alive")]
    StillAlive,

    #[strum(serialize = "vibe_check")]
    #[serde(rename="vibe_check")]
    VibeCheck,

    #[strum(serialize = "network_change")]
    #[serde(rename="network_change")]
    NetworkChange,

    #[strum(serialize = "found_dead_client")]
    #[serde(rename="found_dead_client")]
    FoundDeadClient,

    #[strum(serialize = "exit_network")]
    #[serde(rename="exit_network")]
    ExitNetwork,

    #[strum(serialize = "get_network_state")]
    #[serde(rename="get_network_state")]
    GetNetworkState,
}

// Used to describe the status of a message 
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
pub enum MessageError {
    BadJSON,
    BadType,
    CouldNotFound(&'static str)
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageError::BadJSON=> {
                write!(f, "Request wasn't a JSON or has a bad structure")
            }
            MessageError::CouldNotFound(field) => {
                write!(f, "Could not found field \"{}\" in request", field)
            }
            MessageError::BadType=> {
                write!(f, "Wrong type was used in request")
            }
        }
    }
}

//
// Generic message
//

#[derive(Deserialize, Serialize, Debug)]
pub struct GenericMessage {
    #[serde(rename="type")]
    pub response_type: MessageType,
    pub status: Status,
    pub error: Option<String>
}

impl GenericMessage {
    pub fn new(message_type: MessageType, status: Status, error: Option<&str>) -> GenericMessage {
        let error = match error {
            Some(err) => err,
            None => "null",
        };
        let message = json!({
            "type": message_type,
            "status": status,
            "error": error,
        });
        let message = message.to_string();
        let message: GenericMessage = serde_json::from_str(&message).unwrap();
        message
    }

    pub fn result(status: Status, error: Option<&str>) -> GenericMessage {
        let error = match error {
            Some(err) => err,
            None => "null",
        };
        let message = json!({
            "type": MessageType::Result,
            "status": status,
            "error": error,
        });
        let message = message.to_string();
        let message: GenericMessage = serde_json::from_str(&message).unwrap();
        message
    }
}

//
//  Network messages
//

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


#[derive(Deserialize, Serialize, Debug)]
pub struct NetworkStateMessage {
    #[serde(rename="type")]
    pub response_type: MessageType,
    pub status: Status,
    pub error: Option<String>,

    pub clients: Vec<WeakClient>
}

impl NetworkStateMessage {
    pub fn new(clients: &Vec<Client>) -> NetworkStateMessage {
        let mut weak_clients = vec![];
        for client in clients {
            weak_clients.push(client.weak());
        }

        NetworkStateMessage {
            response_type: MessageType::GetNetworkState,
            status: Status::Ok,
            error: None,
            clients: weak_clients
        }
    }
    pub fn from_weak(clients: Vec<WeakClient>) -> NetworkStateMessage {
        NetworkStateMessage {
            response_type: MessageType::GetNetworkState,
            status: Status::Ok,
            error: None,
            clients
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NetworkChangesMessage {
    #[serde(rename="type")]
    pub response_type: MessageType,
    pub status: Status,
    pub error: Option<String>,
    
    pub changes: Vec<NetworkChange>
}

impl NetworkChangesMessage {
    pub fn new(changes: Vec<NetworkChange>) -> NetworkChangesMessage {
        NetworkChangesMessage {
            response_type: MessageType::NetworkChange,
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
    pub client: Option<WeakClient>
}

impl NetworkChange {
    pub fn new(change_type: NetworkChangeType, client: Option<WeakClient>) -> Result<NetworkChange, NetworkChangeMessageError> {
        //TODO make a better version of this function!!!
        let client = match client {
            Some(client) => Some(client),
            None => None,
        };
        let change = NetworkChange {
            change_type,
            client
        };
        Ok(change)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkChangeMessageError {
    ClientNotProvided
}

pub type NetChangeMesErr = NetworkChangeMessageError;

//
// Client requests messages
//

#[derive(Deserialize, Serialize, Debug)]
pub struct JoinNetworkMessage {
    #[serde(rename="type")]
    pub response_type: MessageType,
    pub status: Status,
    pub error: Option<String>,
    pub client: WeakClient,
}
pub type DeadClientMessage = JoinNetworkMessage;

//
//  additional functions
//

pub fn get_request_type_str(request: &str) -> Result<(MessageType, Value), MessageError> {
    let field = "type";
    println!("{}", request);
    let root: Value = serde_json::from_str(request)
        .map_err(|_| MessageError::BadJSON)?;
    let request = root.get(field);

    if let None = request {
        return Err(MessageError::CouldNotFound(field));
    }
    let request = request.unwrap().as_str().unwrap();
    let request = MessageType::from_str(request);

    if let Err(_) = request {
        return Err(MessageError::BadType);
    }

    Ok((request.unwrap(), root))
}

pub fn get_request_type_value(root: Value) -> Result<(MessageType, Value), MessageError> {
    let field = "type";
    let request = root.get(field);

    if let None = request {
        return Err(MessageError::CouldNotFound(field));
    }
    let request = request.unwrap().as_str().unwrap();
    let request = MessageType::from_str(request);

    if let Err(_) = request {
        return Err(MessageError::BadType);
    }

    Ok((request.unwrap(), root))
}