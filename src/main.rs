/*- Global allowings -*/
#![allow(
	dead_code,
	unused_variables,
	unused_mut,
	unused_imports
)]

/*- Imports & Modules -*/
mod player;
mod room;
// ---
use tungstenite;
use player::Player;
use room::Room;
use lazy_static::lazy_static;
use redis::{ self, Commands, Connection };
use dotenv::dotenv;
use std::{
	env,
	thread,
	net::TcpListener
};

/*- Constants -*/
const WSS_ADDRESS :&str = "127.0.0.1";
const WSS_PORT    :u16  = 8080;
const R_ENV_KEY_HOSTNAME: &'static str = "REDIS_HOSTNAME";
const R_ENV_KEY_PASSWORD: &'static str = "REDIS_PASSWORD";

/*- Lazy statics -*/
lazy_static! {
	static ref REDIS_HOSTNAME: String = env::var(R_ENV_KEY_HOSTNAME).unwrap();
	static ref REDIS_PASSWORD: String = env::var(R_ENV_KEY_PASSWORD).unwrap();
}

/*- Initialize -*/
fn main() {
	/*- Initialize .env k&v:s -*/
	dotenv().unwrap();
	
	/*- Start websocket server listener -*/
	let server = TcpListener::bind(format!("{}:{}", WSS_ADDRESS, WSS_PORT)).unwrap();

	/*- Pre-warn about missing ENV variables, bevcause lazy static won't initialize them until read -*/
	env::var(R_ENV_KEY_HOSTNAME).unwrap();
	env::var(R_ENV_KEY_PASSWORD).unwrap();

	/*- Print the launch -*/
	println!("\x1b[93mLaunch successful!\x1b[0m");

	/*- Connect non-asynchronously (won't be needed, we use threads instead) -*/
	let mut connection:Connection = redis::Client::open(format!("redis://:{}@{}", *REDIS_PASSWORD, *REDIS_HOSTNAME))
		.unwrap()
		.get_connection()
		.unwrap();

	let mut room:Room<u8> = Room {
		id: "awhduoip".into(),
		players: vec![
			Player::new().suid("lsuid").displayname("Di Name").username("arre21"),
			Player::new().suid("lsu12erid").displayname("Di Name 21").username("arre")
		],
		max_players: 5,
		leader: Player::new().suid("lsuid").displayname("Di Name").username("arre"),
		started: false,
		private: true,
		..Room::default()
	};

	let _:() = connection.hset_multiple("room", &room.to_redis_hash().unwrap()).unwrap();

    /*- Get every request isn't Err(_) -*/
	for request in server.incoming() {
		let request = match request {
			Ok(req) => req,
			Err(_) => continue,
		};
		
		/*- Spawn a new thread for each connection -*/
		thread::spawn(move || {

			/*- Try accept websocket tunnel connection -*/
			let mut websocket = match tungstenite::accept(request) {
				Ok(msg) => msg,
				Err(_) => return
			};

			/*- Connect non-asynchronously (won't be needed, we use threads instead) -*/
			let mut connection:Connection = redis::Client::open(format!("redis://:{}@{}", *REDIS_PASSWORD, *REDIS_HOSTNAME))
				.unwrap()
				.get_connection()
				.unwrap();

			let _:() = connection.mset_nx(&[("a", 1)]).unwrap();

			/*- Client tunnel handled here -*/
			loop {
				/*- Get client message -*/
				let message = match websocket.read_message() {
					Ok(msg) => msg,
					Err(_) => continue
				};
				
				/*- Match message type (we only accept binary) -*/
				match message {
					tungstenite::Message::Binary(bytes) => {
						let player:Player = Player::from_bytes(&bytes).unwrap();

						println!("Client {player:?} connected");
					},
					_ => ()
				};
			};
		});
	}
}
