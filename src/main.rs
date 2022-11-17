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
use player::Player;
use room::Room;
use lazy_static::lazy_static;
use redis::{ self, AsyncCommands, Commands, Connection };
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

/*- Constants -*/
const WSS_ADDRESS :&str = "127.0.0.1";
const WSS_PORT    :u16  = 8080;
const R_ENV_KEY_HOSTNAME: &'static str = "REDIS_HOSTNAME";
const R_ENV_KEY_PASSWORD: &'static str = "REDIS_PASSWORD";

/*- Types -*/
type Tx = UnboundedSender<Message>;
pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

/*- Lazy statics -*/
lazy_static! {
	static ref REDIS_HOSTNAME: String = env::var(R_ENV_KEY_HOSTNAME).unwrap();
	static ref REDIS_PASSWORD: String = env::var(R_ENV_KEY_PASSWORD).unwrap();

	/*- Account manager -*/
	static ref ACCOUNT_MANAGER_URL: String = env::var("ACCOUNT_MANAGER_URL").unwrap();
}

/*- Initialize -*/
#[tokio::main]
async fn main() {
	/*- Initialize .env k&v:s -*/
	dotenv().unwrap();

	/*- Pre-warn about missing ENV variables, bevcause lazy static won't initialize them until read -*/
	env::var(R_ENV_KEY_HOSTNAME).unwrap();
	env::var(R_ENV_KEY_PASSWORD).unwrap();

	/*- Pre-warn about redis connection -*/
	redis::Client::open(format!("redis://:{}@{}", *REDIS_PASSWORD, *REDIS_HOSTNAME)).unwrap()
		.get_async_connection().await
		.expect("REDIS AUTH might have failed. Do CONFIG SET requirepass <pass>");

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
		handle_ws_connection(peers.clone(), stream, addr).await;
	};
}

async fn handle_ws_connection(peer_map: PeerMap, raw_tcp_stream: TcpStream, addr: SocketAddr) -> () {
	/*- Try accept websocket tunnel connection -*/
	let stream = match tokio_tungstenite::accept_async(raw_tcp_stream).await {
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


	// /*- Client tunnel handled here -*/
	// loop {
	// 	/*- Get client message -*/
	// 	let mut message:tungstenite::Message;
	// 	match peers.lock().unwrap()[&peer_addr].read_message() {
	// 		Ok(msg) => {
	// 			message = msg;
	// 		},
	// 		Err(_) => continue
	// 	};
	// };
}

