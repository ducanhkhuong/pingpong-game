use std::io;
mod game;

use game::pingpong::{game_pingpong_run, pingpong_update, GameData, UserCommand};
use library::network::udp::UDP;

use async_std::task::spawn;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{sleep, Duration};

use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
#[tokio::main]
async fn main() -> Result<(), io::Error> {

    let mut socket = UDP::listen("0.0.0.0:7878").await.unwrap();

    let address = SocketAddrV4::new(Ipv4Addr::new(172, 16, 100, 250), 7878);

    let (tx, rx)                  : (Sender<UserCommand>, Receiver<UserCommand>) = tokio::sync::mpsc::channel(100);
    let (tx_game_data, mut rx_game_data): (Sender<GameData>   , Receiver<GameData>)    = tokio::sync::mpsc::channel(100);

    spawn(async move {
        game_pingpong_run(rx, tx_game_data.clone());
    });


    loop {
        select! {
            result = socket.read() => {
                match result {
                    Ok(data)=> {
                        if let Err(err) =   pingpong_update(tx.clone(),data).await{
                            eprintln!("Can't update data with {:?}", err);
                        }
                    },

                    Err(e) => {
                        panic!("{e}");
                    }
                }
            },

            
            Some(game_data) = rx_game_data.recv() => {
                match serde_json::to_vec(&game_data) {
                    Ok(data) => {
                        if let Err(e) = socket.send(&address, data).await {
                        //if let Err(e) = socket.broadcast(7878, data).await{
                           eprintln!("Failed to send response: {:?}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to serialize GameData to JSON: {:?}", e),
                }
            },
        }
    }
}
