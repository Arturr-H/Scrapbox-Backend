/*- Global allowings -*/
#![allow(
    dead_code,
    unused_variables
)]

/*- Imports & Modules -*/
mod player;
mod room;

use std::thread;
use websocket::sync::Server;
use websocket::OwnedMessage;
use player::Player;

/*- Constants -*/
const PROTOCOL:&str = "rust-websocket";
const ADDRESS :&str = "127.0.0.1";
const PORT 	  :u16 			= 8080;

/*- Initialize -*/
fn main() {
    /*- Start websocket server listener -*/
	let server = Server::bind(format!("{}:{}", ADDRESS, PORT)).unwrap();

	/*- Print the launch -*/
	println!("\x1b[93mLaunch successful!\x1b[0m");

    /*- Get every request isn't Err(_) -*/
	for request in server.filter_map(Result::ok) {
		
        /*- Spawn a new thread for each connection -*/
		thread::spawn(|| {

            /*- Requests need to have protocol to enter tunnel -*/
			if !request.protocols().contains(&PROTOCOL.to_string()) {
				request.reject().ok();
				return;
			};

            /*- Get client and IP -*/
			let mut client = match request.use_protocol(PROTOCOL).accept() {
                Ok(cl) => cl,
                Err(_) => return
            };
			let ip = client.peer_addr().unwrap();

			println!("Connection from {}", ip);

            /*- Send back message -*/
			let message = OwnedMessage::Text("Hello".to_string());
			client.send_message(&message).unwrap();

            /*- Get reciever and sender -*/
			let (mut recv, mut sender) = match client.split() {
                Ok(e) => e,
                Err(_) => return 
            };

            /*- Listen for client messages -*/
			for message in recv.incoming_messages() {

                /*- Get client message -*/
				let message = match message {
                    Ok(msg) => msg,
                    Err(_) => OwnedMessage::Text("".into())
                };

                /*- Match message type -*/
				match message {
					OwnedMessage::Binary(bytes) => {
						let player:Player = Player::from_bytes(&bytes).unwrap();

						println!("Client {player:?} disconnected");
						return;
					}
					_ => sender.send_message(&message).unwrap(),
				}
			}
		});
	}
}