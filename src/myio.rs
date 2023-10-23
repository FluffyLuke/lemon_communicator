use std::{io, str::FromStr, error::Error, fmt::{self, Display}};


pub fn get_input_with_message(message: &str) -> Result<String, InputError> {
    println!("{}", message);
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)
        .map_err(|err| InputError::StdinError)?;
    Ok(input.trim().to_string())
}

pub fn get_input() -> Result<String, InputError> {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)
        .map_err(|err| InputError::StdinError)?;
    Ok(input.trim().to_string())
}

pub fn get_input_parsed<T: FromStr>() -> Result<T, InputError> {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)
        .map_err(|err| InputError::StdinError)?;
    let result = input.trim().parse::<T>()
        .map_err(|err| InputError::ParsingError)?;
    Ok(result)
}

#[derive(Debug)]
pub enum InputError {
    StdinError,
    ParsingError,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::StdinError => {
                write!(f, "Input stdin error")
            }
            InputError::ParsingError => {
                write!(f, "Input parsing error")
            }
        }
    }
}

