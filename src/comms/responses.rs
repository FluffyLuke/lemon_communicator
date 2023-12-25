use std::{fmt, str::FromStr};

use serde::Serialize;
use serde_json::{json, Value};
use strum_macros::EnumString;

use super::client::Client;

extern crate strum;
#[derive(EnumString, Debug, Serialize)]
pub enum RequestType {
    join_network,
    result,
    still_alive,
    vibe_check,
}


#[derive(Serialize)]
pub enum Status {
    ok,
    error,
    server_error,
    wrong_client_input,
    unknown_client,
}


impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::ok => write!(f, "ok"),
            Status::error => write!(f, "error"),
            Status::server_error => write!(f, "server_error"),
            Status::wrong_client_input => write!(f, "wrong_client_input"),
            Status::unknown_client => write!(f, "unknown_client")
        }
    }
}

#[derive(Debug)]
pub enum RequestError {
    BadJSON,
    BadType,
    CouldNotFound(&'static str)
}

impl fmt::Display for RequestError{
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

#[derive(Debug, Serialize)]
pub enum ChangeType<'a> {
    exit_network(&'a Client),
    join_network(&'a Client),
}

pub fn generic_response(message_type: RequestType, status: Status, error: Option<&str>) -> String {
    let error = match error {
        Some(err) => err,
        None => "null",
    };
    json!({
        "type": message_type,
        "status": status,
        "error": error,
    }).as_str().unwrap().to_string()
}

pub fn result_response(status: Status, error: Option<&str>) -> String {
    let error = match error {
        Some(err) => err,
        None => "null",
    };
    json!({
        "type": RequestType::result,
        "status": status,
        "error": error,
    }).to_string()
}

pub fn network_state_response(message_type: &str, status: Status, error: Option<&str>, clients: &mut Vec<Client>) -> serde_json::Value {
    if error == None {
        let json = json!({
            "type": message_type,
            "status": status,
            "error": "null",
            "clients": serde_json::to_string(clients).unwrap()
        });
        return json;
    }
    let error = error.unwrap();
    json!({
        "type": message_type,
        "status": status,
        "error": error,
        "clients": serde_json::to_string(clients).unwrap()
    })
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