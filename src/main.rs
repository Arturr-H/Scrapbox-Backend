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
mod handle_req;
mod ws_status;
mod req_utils;
// ---
use tungstenite;
use player::Player;
use room::Room;
use lazy_static::lazy_static;
use redis::{ self, Commands, Connection };
use dotenv::dotenv;
use handle_req::handle_req;
use responder::prelude::*;
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

	/*- Account manager -*/
	static ref ACCOUNT_MANAGER_URL: String = env::var("ACCOUNT_MANAGER_URL").unwrap();
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

	/*- Pre-warn about redis connection -*/
	redis::Client::open(format!("redis://:{}@{}", *REDIS_PASSWORD, *REDIS_HOSTNAME)).unwrap()
		.get_connection()
		.expect("REDIS AUTH might have failed. Do CONFIG SET requirepass <pass>");

	/*- Pre-warn about scrapbox-account-manager connection -*/
	reqwest::blocking::get(&**ACCOUNT_MANAGER_URL)
		.expect("Account manager server not up!");

	/*- Print the launch -*/
	println!("Launch successful on {}:{}!", WSS_ADDRESS, WSS_PORT);

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

			/*- Client tunnel handled here -*/
			loop {
				/*- Get client message -*/
				let mut message:tungstenite::Message;
				match websocket.read_message() {
					Ok(msg) => {
						message = msg;
					},
					Err(_) => continue
				};

				/*- Match message type (we only accept binary) -*/
				match message {
					tungstenite::Message::Text(text) => handle_req(&text, &mut websocket, &mut connection),
					_ => ()
				};
			};
		});
	}
}
