/*- Global allowings -*/
#![allow(
    dead_code,
    unused_variables
)]

/*- Imports & Modules -*/
mod player;
mod room;
// ---
use tungstenite;
use player::Player;
use std::{
	thread,
	net::TcpListener
};

/*- Constants -*/
const PROTOCOL:&str = "rust-websocket";
const ADDRESS :&str = "127.0.0.1";
const PORT 	  :u16 			= 8080;

/*- Initialize -*/
fn main() {
    /*- Start websocket server listener -*/
	let server = TcpListener::bind(format!("{}:{}", ADDRESS, PORT)).unwrap();

	/*- Print the launch -*/
	println!("\x1b[93mLaunch successful!\x1b[0m");

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