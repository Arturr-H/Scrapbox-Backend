
/*- Imports -*/
use std::{
    net::{ TcpStream, SocketAddr },
    collections::{ BTreeMap, HashMap },
    sync::Mutex
};
use mongodb::{Client, Database};
use serde_json::{ json, Value };
use serde_derive::{ Serialize, Deserialize };
use futures_channel::mpsc::{ unbounded, UnboundedSender };
use futures_util::{ future, pin_mut, stream::TryStreamExt, StreamExt };
use reqwest;
use tungstenite::{ WebSocket, Message };
use crate::{
    ACCOUNT_MANAGER_URL,
    MONGO_DATABASE_NAME,
    MONGO_HOST,
    room::Room,
    player::Player as PlayerInner,
    wrapper::PlayerRedisWrapper as Player,
    ws_status,
    req_utils,
    PeerMap
};

/*- Structs & enums -*/
#[derive(Serialize, Deserialize, Debug)]
pub struct GeneralRequest<'a> { pub destination: &'a str, pub data: String }

/*- Main enum for pairing request destinations with their JSON data  -*/
#[derive(Serialize, Deserialize, Debug)]
pub enum RequestJsonType {
    CreateRoom(CreateRoomRequestData),
    JoinRoom(JoinRoomRequestData),
}

/*- Other structs for containing JSON data coupled to requests -*/
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRoomRequestData {
    jwt: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct JoinRoomRequestData {
    jwt: String,
    room_id: String
}

/*- Main -*/
pub async fn handle_req<'a>(
    msg:tungstenite::Message,
    peer_map: &PeerMap,
    current_connection: SocketAddr
) -> Result<(), tungstenite::Error> {
    /*- Get peers -*/
    let peers = match peer_map.lock() {
        Ok(e) => e,
        Err(_) => panic!("")
    };

    /*- Open database client -*/
    // let mut mongodb_connection = match mongodb::Client::with_uri_str(&**MONGO_HOST).await {
	// 	Ok(e) => e,
	// 	Err(_) => panic!("")
	// }.database(&**MONGO_DATABASE_NAME);

    /*- Which client we want to broadcast to -*/
    // let broadcast_recipients = peers
    //     .iter()
    //     // .filter(|(peer_addr, _)| peer_addr != &&addr)
    //     .map(|(_, ws_sink)| ws_sink);

    // /*- Send to all selected client -*/
    // for recp in broadcast_recipients {
    //     match recp.unbounded_send(msg.clone()) {
    //         Ok(e) => e,
    //         Err(_) => return Ok(())
    //     };
    // }

    /*- Get what type of JSON struct to use -*/
    if let Message::Text(text) = msg {
        let mut request:RequestJsonType = match serde_json::from_str::<GeneralRequest>(&text) {
            Ok(GeneralRequest { destination, data }) => {
                println!("{destination}");
                /*- Get what type of json data is to be serialized -*/
                match destination {
                    "create-room"   => RequestJsonType::CreateRoom(
                        match serde_json::from_str::<CreateRoomRequestData>(&data) { Ok(e) => e, Err(e) => panic!("{e}") }
                    ),
                    "join-room"     => RequestJsonType::JoinRoom(
                        match serde_json::from_str::<JoinRoomRequestData>(&data) { Ok(e) => e, Err(e) => panic!("{e}") }
                    ),
                    _ => panic!("")
                }
            },
            Err(_) => panic!("")
        };
        
        let websocket = peers.get(&current_connection).unwrap();

        /*- Check what request type -*/
        // match match match request {
        //     RequestJsonType::CreateRoom(data) => create_room(&data, &mut mongodb_connection).await,
        //     RequestJsonType::JoinRoom(data) => join_room(&data, &mut mongodb_connection).await,
        // } {

        //     /*- Write status to websocket tunnel -*/
        //     Ok(json)    => websocket.unbounded_send(Message::Text(json.to_string())).ok(),
        //     Err(status) => websocket.unbounded_send(Message::Text(
        //         json!({
        //             "status": status
        //         }).to_string()
        //     )).ok()
        // } {
        //     Some(e) => Ok(e),
        //     None => panic!("")
        // }
        panic!("end")
    }else {
        panic!("")
    }
}

/*- Functions -*/
pub async fn create_room(
    request:&CreateRoomRequestData,
    mongodb_connection: &mut Database
) -> Result<Value, u16> {
    /*- GET JWT auth status -*/
    let status:u16 = Player::check_auth(&request.jwt).await;

    /*- Check status -*/
    match status {
        // Unauthorized
        401 => Err(401),

        // Ok
        200 => {
            /*- Authorize player -*/
            let current_player = match req_utils::authorize_player(&request.jwt).await {
                Ok(player) => player,
                Err(status) => return Err(status)
            };

            /*- Get room details -*/
            let private_id:String = Room::gen_private_id();
            let public_id:u32     = Room::gen_public_id();
            let room_name:String  = format!("room:{}", public_id);
            let mut room          = Room::from_leader(current_player, private_id.clone(), public_id);

            /*- Add room to mongodb -*/
            mongodb_connection.collection::<Room>("rooms").insert_one(&room, None).await.unwrap();

            /*- Debug -*/
            room.quick_display("Created room");

            /*- Return -*/
            Ok(json!({
                "status": 200,
                "room": room.to_string()
            }))
        },
        _ => Err(401)
    }
}

pub async fn join_room(
    request:&JoinRoomRequestData,
    mongodb_connection: &mut Database
) -> Result<Value, u16> {
    /*- GET JWT auth status -*/
    let status:u16 = Player::check_auth(&request.jwt).await;
    dbg!(1);

    /*- Check status -*/
    match status {
        // Unauthorized
        401 => Err(401),

        // Ok
        200 => {
            dbg!(2);
            /*- Authorize player -*/
            let current_player = match req_utils::authorize_player(&request.jwt).await {
                Ok(player) => player,
                Err(status) => return Err(status)
            };
            dbg!(3);

            /*- Get room -*/
            let room_name:String = format!("room:{}", &request.room_id);
            // let room_data:BTreeMap<String, String> = match mongodb_connection. (&room_name) {
            //     Ok(e) => e,
            //     Err(_) => return Err(ws_status::CORRUPTED_ROOM)
            // };
            dbg!(4);

            /*- Get the room but return err if parse failed -*/
            // let mut room:Room = match Room::from_redis_hash(&room_data) {
            //     Some(e) => e,

            //     /*- Return -*/
            //     None => return Err(401)
            // };
            dbg!(5);

            /*- Check if player is already in room -*/
            // if room.players.contains(&current_player) {

            //     Ok(json!({
            //         "status": 200,
            //         "room": room.to_string()
            //     }))
            // }else {

            //     /*- Push player to room -*/
            //     room.players.push(current_player);

            //     /*- Convert to redis hash -*/
            //     room.serialize_players_unchecked();

            //     /*- Create room -*/
            //     let _:() = match mongodb_connection.hset(&room_name, "players", &room._players) {
            //         Ok(e) => e,
            //         Err(_) => return Err(ws_status::ROOM_UPDATE_PLAYERS),
            //     };

            //     /*- Debug -*/
            //     room.quick_display("Joined room");

            //     /*- Return -*/
            // }
            Ok(json!({
                "status": 200,
                "room": "room.to_string()"
            }))
        },
        _ => Err(401)
    }
}


