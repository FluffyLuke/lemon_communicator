use tokio::{net::{TcpListener, TcpStream}, io::{AsyncWriteExt, BufReader, AsyncBufReadExt}};
use tokio::time::sleep;

use crate::{command_args::ParsedArgs, comms::{client::{client_handler, client_reqs::join_network}, network::api::{get_request_type_str, MessageType}}};

use self::{network::{api::{GenericMessage, Status}, NETWORK_CHANGES}, client::KNOWN_CLIENTS};

mod client;
mod network;

pub async fn start_server(args: ParsedArgs) -> std::io::Result<()> {
    let addr = format!("127.0.0.1:{}", args.port);
    let listener = TcpListener::bind(addr).await?;

    println!("Listener started at port: {}", args.port);
    
    let mut handles = vec![];

    handles.push(tokio::spawn(serve_client(args, listener)));
    println!("Listening for client requests");
    //handles.push(tokio::spawn(check_if_dead(args)));
    //println!("Vibe-checker ready. Vibe check every {} secs", args.vibe_check_interval.as_secs());
    handles.push(tokio::spawn(check_updates(args)));
    println!("Client updater ready. Update every {} secs", args.update_client_interval.as_secs());

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}

async fn serve_client(_args: ParsedArgs, listener: TcpListener) {
    let mut buf = String::new();
    let mut handles = vec![];
    loop {
        buf.clear();
        let (stream, addr) = match listener.accept().await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error while accepting client: {:?}", e);
                continue;
            },
        };
        
        println!("Serving new client: {}", addr);

        let mut buf_stream = BufReader::new(stream);

        let _bytes_read = match buf_stream.read_line(&mut buf).await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error while serving client: {:?}", e);
                continue;
            }
        };

        let result = match get_request_type_str(&buf) {
            Ok((MessageType::JoinNetwork, value)) => join_network(buf_stream, addr, value).await,
            Err(e) => {
                println!("Cannot parse client's request: {}", e);
                let result = wrong_request(&mut buf_stream).await;
                if let Err(e) = result {
                    eprintln!("Error while serving client: {:?}", e);
                }
                continue;
            }
            Ok((_other_request_type, _)) => {
                println!("Wrong request provided by client",);
                let result = wrong_request(&mut buf_stream).await;
                if let Err(e) = result {
                    eprintln!("Error while serving client: {:?}", e);
                }
                continue;
            },
        };
    
        let handle = match result {
            Ok(client) => tokio::spawn(client_handler::client_handler(client)),
            Err(err) => {
                eprintln!("Error while serving client: {:?}", err);
                continue;
            },
        };
        handles.push(handle);
    }
}

async fn wrong_request(socket: &mut BufReader<TcpStream>,) -> std::io::Result<()> {
    let error = "Wrong request";
    let response = GenericMessage::result(Status::Error, Some(error));
    let response = serde_json::to_string(&response).unwrap();
    socket.write_all(response.as_bytes()).await?;
    Ok(())
}

async fn check_updates(args: ParsedArgs) {
    loop {
        sleep(args.update_client_interval).await; // Wait between checks
        println!("Updating clients!");

        let changes = NETWORK_CHANGES.get_changes_json().await;
        if let None = changes {
            println!("No changes found!");
            continue;
        }
        let changes = changes.unwrap();
        KNOWN_CLIENTS.update_clients(changes).await;
        KNOWN_CLIENTS.get_network_state().await;
        println!("End of update!");
    }
}



// async fn check_if_dead(args: ParsedArgs){
//     loop {
//         sleep(args.vibe_check_interval).await; // Wait between checks
//         println!("Runs vibe check!");
//         let results = Arc::new(Mutex::new(vec![]));
//         {
//             let mut tasks = vec![];
//             let mut locked_clients = KNOWN_CLIENTS.clients.write().await;
//             for client in locked_clients.iter_mut() {
//                 let results_copy = Arc::clone(&results);
//                 let handle = tokio::spawn(async move{
//                     tokio::select! {
//                         if_alive = vibe_check(client) => {
//                             if !if_alive {
//                                 println!("Client {} is dead", client.name);
//                                 results_copy.lock().await.push(client.weak());
//                             }
//                         }
//                         _ = sleep(time::Duration::from_secs(10)) => {
//                             println!("Client {} is dead", client.name);
//                             results_copy.lock().await.push(client.weak());
//                         }
//                     }
//                 });
//                 tasks.push(handle);
//             }
//             for task in tasks {
//                 task.await;
//             }
//         }
//     }
// }

// Function checking for dead clients
// async fn check_if_dead(args: ParsedArgs) {
//     loop {
//         sleep(args.vibe_check_interval).await; // Wait between checks
//         println!("Runs vibe check!");

    //     // Vector containing ids of dead clients
    //     let dead_client_ids: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
    //     // Vector containing handles of all tokio threads created in this function
    //     let mut handles = vec![];
        
    //     // We want to drop all senders after this block,
    //     // so we wrap it in it's scope
    //     {
    //         // Channel to communicate between vibe checkers and a thread
    //         // that pushes dead client ids into "dead_client_ids" vector
    //         // It is like this to not block the dead_client_ids's mutex
    //         let (tx, mut rx) = mpsc::channel(64);

    //         // We copy clients to avoid locking data for to long for other tasks
    //         let copied_clients = KNOWN_CLIENTS.read().await;

    //         // Thread appending dead_client_id vector with ids
    //         let dead_client_ids_arc = Arc::clone(&dead_client_ids);
    //         handles.push(tokio::spawn(async move {
    //             let mut results = dead_client_ids_arc.lock().await;
    //             loop {
    //                 let id = rx.recv().await;
    //                 match id {
    //                     Some(id) => results.push(id),
    //                     None => return,
    //                 }
    //             }
    //         }));

    //         // Vibe checkers. If client is dead they broadcast it
    //         // thru the channel
    //         for client in copied_clients.iter() {
    //             let copy_tx = tx.clone();
    //             let client = client.clone();
    //             handles.push(tokio::spawn(async move {
    //                 tokio::select! {
    //                     if_alive = vibe_check(&mut client) => {
    //                         if !if_alive {
    //                             println!("Client {} is dead", client.name);
    //                             copy_tx.send(client.id).await.unwrap();
    //                         }
    //                     }
    //                     _ = sleep(time::Duration::from_secs(10)) => {
    //                         println!("Client {} is dead", client.name);
    //                         copy_tx.send(client.id).await.unwrap();
    //                     }
    //                 }
    //             }));
    //         }
    //     }
    //     // Await for all threads to complete
    //     for handle in handles {
    //         let result = handle.await;
    //         if let Err(e) = result {
    //             println!("Handle error during vibe check: {:?}", e);
    //         }
    //     }
    //     println!("Checked all clients!");
    //     // Remove dead clients from the list
    //     let mut changes = vec![];
    //     {
    //         let locked_dead_clients_ids = dead_client_ids.lock().await;
    //         let mut locked_clients = KNOWN_CLIENTS.lock().await;
    //         for client in locked_clients.iter() {
    //             if locked_dead_clients_ids.contains(&client.id) {
    //                 let change = NetworkChange::new(NetworkChangeType::ExitNetwork, Some(client)).unwrap();
    //                 changes.push(change);
    //             }
    //         }
    //         locked_clients.retain(|client| !locked_dead_clients_ids.contains(&client.id));
    //     }

    //     append_changes(changes).await;

    //     println!("End of vibe check!");
    // }
//}