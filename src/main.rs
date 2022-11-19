/*- Global allowings -*/
#![allow(
	dead_code,
	unused_variables,
	unused_mut,
	unused_imports
)]

/*- Imports & Modules -*/
// Player modules
#[path ="./player/player.rs"]
mod player;

#[path ="./player/wrapper.rs"]
mod wrapper;

mod room;
mod handle_req;
mod ws_status;
mod req_utils;
// ---
use tokio_tungstenite;
use tungstenite::protocol::Message;
use tokio::net::{ self, TcpListener, TcpStream };
use serde_json::{ json, Value };
use player::Player;
use room::Room;
use lazy_static::lazy_static;
use mongodb;
use dotenv::dotenv;
use handle_req::handle_req;
use responder::prelude::*;
use futures_channel::mpsc::{ unbounded, UnboundedSender };
use futures_util::{ future, pin_mut, stream::TryStreamExt, StreamExt };
use std::{
	env,
	thread,
	sync::{ Mutex, Arc },
	collections::HashMap,
	net::SocketAddr,
};

use crate::handle_req::{RequestJsonType, GeneralRequest, CreateRoomRequestData, JoinRoomRequestData};

/*- Constants -*/
const WSS_ADDRESS :&str = "127.0.0.1";
const WSS_PORT    :u16  = 8080;
const ENV_MONGO_HOST: &'static str = "MONGO_HOST_URL";
const ENV_ACCOUNT_MANAGER_URL: &'static str = "ACCOUNT_MANAGER_URL";
const ENV_MONGO_DATABASE_NAME: &'static str = "MONGO_DATABASE_NAME";

/*- Types -*/
type Tx = UnboundedSender<Message>;
pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

/*- Lazy statics -*/
lazy_static! {
	static ref MONGO_HOST: String          = env::var(ENV_MONGO_HOST).unwrap();
	static ref MONGO_DATABASE_NAME: String = env::var(ENV_MONGO_DATABASE_NAME).unwrap();

	/*- Account manager -*/
	static ref ACCOUNT_MANAGER_URL: String = env::var(ENV_ACCOUNT_MANAGER_URL).unwrap();
}

/*- Initialize -*/
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
	/*- Initialize .env k&v:s -*/
	dotenv().unwrap();

	/*- Pre-warn about missing ENV variables, because lazy static won't initialize them until read -*/
	env::var(ENV_MONGO_HOST).expect("MONGO_HOST env var missing");
	env::var(ENV_ACCOUNT_MANAGER_URL).expect("ACCOUNT_MANAGER_URL env var missing");

	/*- Pre-warn about mongodb connection -*/
	mongodb::Client::with_uri_str(&**MONGO_HOST).await
		.expect("MongoDB not up!")
		.database(&**MONGO_DATABASE_NAME);

	/*- Pre-warn about scrapbox-account-manager connection -*/
	reqwest::get(&**ACCOUNT_MANAGER_URL).await
		.expect("Account manager server not up!");

	/*- Start websocket server listener -*/
	let server = TcpListener::bind(format!("{}:{}", WSS_ADDRESS, WSS_PORT)).await.unwrap();

	/*- Print the launch -*/
	println!("Launch successful on {}:{}!", WSS_ADDRESS, WSS_PORT);

	/*- Create websocket client hashmap -*/
	let peers:PeerMap = Arc::new(Mutex::new(HashMap::new()));

    /*- Get every request isn't Err(_) -*/
	while let Ok((stream, addr)) = server.accept().await {
		tokio::spawn(handle_ws_connection(peers.clone(), stream, addr));
	};
	Ok(())
}


async fn handle_ws_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
	/*- Try accept websocket tunnel connection -*/
	let stream = match tokio_tungstenite::accept_async(raw_stream).await {
		Ok(e) => e,
		Err(_) => return
	};

	/*- Push client -*/
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx);
	let (outgoing, incoming) = stream.split();

	/*- Get incoming requests -*/
    let broadcast_incoming = incoming.try_for_each(|message| handle_req(message, &peer_map, addr));

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);

	/*- Remove connection from peer map -*/
    peer_map.lock().unwrap().remove(&addr);
}


// async fn handle_ws_connection(peer_map: PeerMap, raw_tcp_stream: TcpStream, addr: SocketAddr) {
// 	println!("1");
// 	/*- Try accept websocket tunnel connection -*/
// 	let stream = match tokio_tungstenite::accept_async(raw_tcp_stream).await {
// 		Ok(e) => e,
// 		Err(_) => return
// 	};
// 	println!("2");

// 	/*- Push client -*/
//     let (sender, reciever) = unbounded();
//     peer_map.lock().unwrap().insert(addr, sender);
// 	let (outgoing, incoming) = stream.split();
// 	println!("4");

// 	/*- Get incoming requests -*/
//     let broadcast_incoming = incoming.try_for_each(|message| handle_req(message, &peer_map, addr));
//     let receive_from_others = reciever.map(Ok).forward(outgoing);

//     pin_mut!(broadcast_incoming, receive_from_others);
//     future::select(broadcast_incoming, receive_from_others).await;

// 	/*- Open mongodb client -*/
// 	// let mut mongodb_client = match mongodb::Client::with_uri_str(MONGO_HOST) {
// 	// 	Ok(e) => e,
// 	// 	Err(_) => return
// 	// };
	
// 	/*- Remove client from redis room -*/
// 	// let _:() = match redis_client {
// 	// 	Ok(ref mut e) => {
// 	// 		let _:() = e.hdel("rooms", addr.to_string()).await.unwrap();
// 	// 		e.publish("rooms", "update").await.unwrap();
// 	// 	},
// 	// 	Err(_) => ()
// 	// };

// 	println!("{} disconnected", &addr);

// 	/*- Remove connection from peer map -*/
// 	peer_map.lock().unwrap().remove(&addr);
// }

