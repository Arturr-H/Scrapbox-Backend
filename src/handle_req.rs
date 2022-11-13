
/*- Imports -*/
use std::{net::TcpStream, collections::BTreeMap};
use redis::{ Connection, Commands };
use serde::{ Deserialize, Serialize };
use serde_json::{json, Value};
use serde_derive::{ Serialize, Deserialize };
use reqwest;
use tungstenite::{ WebSocket, Message };
use crate::{ ACCOUNT_MANAGER_URL, room::Room, player::Player, ws_status, req_utils };

/*- Structs & enums -*/
#[derive(Serialize, Deserialize, Debug)]
struct GeneralRequest<'a> { pub destination: &'a str, pub data: String }

/*- Main enum for pairing request destinations with their JSON data  -*/
#[derive(Serialize, Deserialize, Debug)]
enum RequestJsonType {
    CreateRoom(CreateRoomRequestData),
    JoinRoom(JoinRoomRequestData),
}

/*- Other structs for containing JSON data coupled to requests -*/
#[derive(Serialize, Deserialize, Debug)]
struct CreateRoomRequestData {
    jwt: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct JoinRoomRequestData {
    jwt: String,
    room_id: String
}

/*- Main -*/
pub fn handle_req<'a>(
    text:&String,
    websocket:&mut WebSocket<TcpStream>,
    redis_connection: &mut Connection
) -> () {

    /*- Get what type of JSON struct to use -*/
    let mut request:RequestJsonType = match serde_json::from_str::<GeneralRequest>(text) {
        Ok(GeneralRequest { destination, data }) => {
            /*- Get what type of json data is to be serialized -*/
            match destination {
                "create-room"   => RequestJsonType::CreateRoom(
                    match serde_json::from_str::<CreateRoomRequestData>(&data) { Ok(e) => e, Err(_) => return }
                ),
                "join-room"     => RequestJsonType::JoinRoom(
                    match serde_json::from_str::<JoinRoomRequestData>(&data) { Ok(e) => e, Err(_) => return }
                ),
                _ => return
            }
        },
        Err(_) => return
    };

    /*- Check what request type -*/
    match match request {
        RequestJsonType::CreateRoom(data) => create_room(&data, websocket, redis_connection),
        RequestJsonType::JoinRoom(data) => join_room(&data, websocket, redis_connection),
    } {

        /*- Write status to websocket tunnel -*/
        Ok(json)    => websocket.write_message(Message::Text(json.to_string())).ok(),
        Err(status) => websocket.write_message(Message::Text(
            json!({
                "status": status
            }).to_string()
        )).ok()
    };
}

/*- Functions -*/
fn create_room(
    request:&CreateRoomRequestData,
    websocket:&mut WebSocket<TcpStream>,
    redis_connection: &mut Connection
) -> Result<Value, u16> {
    /*- GET JWT auth status -*/
    let status:u16 = Player::check_auth(&request.jwt);

    /*- Check status -*/
    match status {
        // Unauthorized
        401 => Err(401),

        // Ok
        200 => {
            /*- Get room details -*/
            let private_id:String = Room::gen_private_id();
            let public_id:u32     = Room::gen_public_id();
            let room_name:String  = format!("room:{}", public_id);
            let mut room          = Room::from_leader(Player::default(), private_id.clone(), public_id);

            /*- Convert to redis hash -*/
            let redis_room = match room.to_redis_hash() {
                Ok(e) => e,
                Err(_) => return Err(404)
            };

            /*- Create room -*/
            let _:() = match redis_connection.hset_multiple(room_name, &redis_room) {
                Ok(e) => e,
                Err(_) => return Err(404)
            };

            /*- Return -*/
            println!("CREATE: {room:?}");
            Ok(json!({
                "status": 200,
                "room": room.to_string()
            }))
        },
        _ => Err(401)
    }
}

fn join_room(
    request:&JoinRoomRequestData,
    websocket:&mut WebSocket<TcpStream>,
    redis_connection: &mut Connection
) -> Result<Value, u16> {
    /*- GET JWT auth status -*/
    let status:u16 = Player::check_auth(&request.jwt);

    /*- Check status -*/
    match status {
        // Unauthorized
        401 => Err(401),

        // Ok
        200 => {
            /*- Authorize player -*/
            let current_player = match req_utils::authorize_player(&request.jwt) {
                Ok(player) => player,
                Err(status) => return Err(status)
            };

            /*- Get room -*/
            let room_name:String = format!("room:{}", &request.room_id);
            let room_data:BTreeMap<String, String> = match redis_connection.hgetall(&room_name) {
                Ok(e) => e,
                Err(_) => return Err(ws_status::CORRUPTED_ROOM)
            };

            /*- Get the room but return err if parse failed -*/
            let mut room:Room = match Room::from_redis_hash(&room_data) {
                Some(e) => e,

                /*- Return -*/
                None => return Err(401)
            };

            /*- Check if player is already in room -*/
            if room.players.contains(&current_player) {
                Ok(json!({
                    "status": 200,
                    "room": room.to_string()
                }))
            }else {

                /*- Push player to room -*/
                room.players.push(current_player);

                /*- Convert to redis hash -*/
                room.serialize_players_unchecked();

                /*- Create room -*/
                let _:() = match redis_connection.hset(&room_name, "players", &room._players) {
                    Ok(e) => e,
                    Err(_) => return Err(ws_status::ROOM_UPDATE_PLAYERS),
                };

                /*- Return -*/
                println!("JOIN: {room:?}");
                Ok(json!({
                    "status": 200,
                    "room": room.to_string()
                }))
            }
        },
        _ => Err(401)
    }
}


