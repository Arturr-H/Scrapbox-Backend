use redis::{ Connection, Commands };
use responder::{Stream, Respond};
use crate::{ REDIS_HOSTNAME, ACCOUNT_MANAGER_URL, REDIS_PASSWORD, player::Player, room::Room };
use reqwest;


/*- Instantiate room -*/
// pub fn create_room(stream:&mut Stream) -> () {
// 	let cookies = stream.get_cookies();

// 	/*- Get room -*/
// 	let (jwt, suid) = match (cookies.get("token"), cookies.get("suid")) {
// 		(Some(j), Some(s)) => (j, s),

// 		// Unauthorized
// 		(_, _) => return stream.respond_status(401)
// 	};

// 	/*- Connect non-asynchronously (won't be needed, we use threads instead) -*/
// 	let mut redis_connection:Connection = redis::Client::open(format!("redis://:{}@{}", *REDIS_PASSWORD, *REDIS_HOSTNAME))
// 		.unwrap()
// 		.get_connection()
// 		.unwrap();

// 	/*- GET JWT auth status -*/
// 	let status:u16 = Player::check_auth(&jwt);

// 	/*- Check status -*/
// 	match status {

// 		// Ok
// 		200 => {
// 			/*- Get player -*/
// 			let mut player_json_text:String;

// 			/*- Fetch account api -*/
// 			let player:Player = match reqwest::blocking::get(format!("{}profile/data/by_suid/{}", *ACCOUNT_MANAGER_URL, suid)) {
// 				Ok(response) => {

// 					/*- Get response text -*/
// 					player_json_text = response.text().unwrap_or(String::new()).clone();

// 					/*- Parse response text to Player struct -*/
// 					match serde_json::from_str::<Player>(&player_json_text) {
// 						Ok(a) => a,
// 						Err(_) => return stream.respond_status(500)
// 					}
// 				},
// 				Err(_) => return stream.respond_status(500)
// 			};

// 			/*- Get room details -*/
// 			let id:String = Room::gen_id();
// 			let room = match Room::from_leader(Player::default(), id.clone()).to_redis_hash() {
// 				Ok(e) => e,
// 				Err(_) => return stream.respond_status(405)
// 			};

// 			/*- Create room -*/
// 			let _:() = match redis_connection.hset_multiple(id.clone(), &room) {
// 				Ok(e) => e,
// 				Err(_) => return stream.respond_status(406)
// 			};

// 			/*- Redirect to room -*/
// 			stream.respond(200, Respond::new().text(&format!("{{\"id\": \"{}\"}}", id)));
// 		},

// 		// Unauthorized
// 		_ => stream.respond_status(401)
// 	};
// }
