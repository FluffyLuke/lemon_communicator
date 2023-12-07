use tokio::{net::TcpListener, io::AsyncWriteExt};

use crate::command_args::ParsedCommands;

pub async fn start_server(commands: ParsedCommands) -> std::io::Result<()>{
    let addr = format!("127.0.0.1:{}", commands.port);
    let listener = TcpListener::bind(addr).await?;

    let (mut stream, _) = listener.accept().await?;
    stream.write_all(b"dupa").await?;

    Ok(())
}