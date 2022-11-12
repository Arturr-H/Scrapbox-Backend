
/*- Imports -*/
use std::{net::TcpStream, collections::BTreeMap};
use redis::{ Connection, Commands };
use serde::{ Deserialize, Serialize };
use serde_json::{json, Value};
use serde_derive::{ Serialize, Deserialize };
use reqwest;
use tungstenite::{ WebSocket, Message };
use crate::{ ACCOUNT_MANAGER_URL, room::Room, player::Player };

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
            println!("{destination}");
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
            let id:String = Room::gen_id();
            let mut room = Room::from_leader(Player::default(), id.clone());
            

            let redis_room = match room.to_redis_hash() {
                Ok(e) => e,
                Err(_) => return Err(404)
            };

            /*- Create room -*/
            let _:() = match redis_connection.hset_multiple(id.clone(), &redis_room) {
                Ok(e) => e,
                Err(_) => return Err(404)
            };

            /*- Return -*/
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
            /*- Get room -*/
            println!("{}", request.room_id);
            let room_data:BTreeMap<String, String> = match redis_connection.hgetall(&request.room_id) {
                Ok(e) => e,
                Err(_) => return Err(404)
            };

            println!("AOMOA:::::::::: {:?}", Room::from_redis_hash(&room_data));

            /*- Get room details -*/
            let id:String = Room::gen_id();
            let room = match Room::from_leader(Player::default(), id.clone()).to_redis_hash() {
                Ok(e) => e,
                Err(_) => return Err(404)
            };

            /*- Return -*/
            Ok(json!({
                "status": 200,
                // "room": room.to_string()
            }))
        },
        _ => Err(401)
    }
}


