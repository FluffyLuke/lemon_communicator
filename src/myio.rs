use std::io;

pub fn get_input_with_message(message: &str) -> std::io::Result<String> {
    println!("{}", message);
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)?;
    Ok(String::from(input.trim()))
}

pub fn get_input() -> std::io::Result<String> {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)?;
    Ok(String::from(input.trim()))
}