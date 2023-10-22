use std::{io, str::FromStr, error::Error, fmt};

type Result<T> = std::result::Result<T, InputError>;

pub fn get_input_with_message(message: &str) -> Result<String> {
    println!("{}", message);
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)
        .map_err(|err| InputError::StdinError(err))?;
    Ok(input.trim().to_string())
}

pub fn get_input() -> Result<String> {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)
        .map_err(|err| InputError::StdinError(err))?;
    Ok(input.trim().to_string())
}

pub fn get_input_parsed<T: FromStr>() -> Result<T> {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)
        .map_err(|err| InputError::StdinError(err))?;
    let result = input.trim().parse::<T>()
        .map_err(|err| InputError::StdinError(err))?;
    Ok(result)
}

#[derive(Debug)]
enum InputError<T>{
    StdinError(std::io::Error),
    ParsingError(std::io::Error),
}

impl <T> fmt::Display for InputError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InputError::StdinError(e) => {
                write!(f, "Input stdin error: {}", e)
            }
            InputError::ParsingError(e) => {
                write!(f, "Input parsing error: {}", e)
            }
        }
    }
}

