/*- Global allowings -*/
#![allow(
    dead_code
)]

/*- Imports -*/
use serde_derive::{ Serialize, Deserialize };
use std::{default::Default, net::{SocketAddr, Ipv4Addr}};
use crate::{ ACCOUNT_MANAGER_URL, Player };

/*- Player which will be stored in the redis backend, will
    contain the player itself (Player struct) and the
    LocalGameData struct for saving things like snippets -*/
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerRedisWrapper {
    // Inner values
    pub player: Player,

    // Data stored temporary for each game
    pub local_data: LocalGameData,

    // Socket address which is stored in peer map, used to find and remove player
    // from peer map when they disconnect.
    pub socket_addr: String,
}

/*- Game data, only lives for the lifetime of a game -*/
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct LocalGameData {
    // The game start screen will have a whiteboard with all frames
    // containing the player sprites, this coordinate will be where
    // on the board this player lies
    pub board_position    : [u8; 2],
}

/*- Method implementations -*/
impl PlayerRedisWrapper {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    /*- Bincode deserialization for transport in websocket tunnels -*/
    pub fn from_bytes<'lf>(input: &'lf [u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        /*- Deserialize & if fail return -*/
        let player:PlayerRedisWrapper = bincode::deserialize(input)?;

        /*- return -*/
        Ok(player)
    }

    /*- Bincode serialization for transport -*/
    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
        /*- Serialize & if fail Err(_) -*/
        let bytes:Vec<u8> = bincode::serialize(&self)?;

        /*- Return -*/
        Ok(bytes)
    }
    pub fn to_bytes_unchecked(&self) -> Vec<u8> {
        /*- Serialize & if fail Err(_) -*/
        bincode::serialize(&self).unwrap_or(Vec::new())
    }

    /*- Check user auth status -*/
    pub async fn check_auth(jwt:&str) -> u16 {
            /*- GET JWT auth status -*/
        let url = format!("{}{}", &**ACCOUNT_MANAGER_URL, "profile/verify-token");
        match reqwest::Client::new()
            .get(&url)
            .header("token", jwt)
            .send().await
            { Ok(e) => e, Err(_) => return 402u16 }
            .status()
            .as_u16()
    }

    /*- Fetch player data by SUID -*/
    pub async fn fetch_player(suid:&str) -> Option<String> {
        /*- Get JSON data -*/
        let json_fetch:String = reqwest::get(
            format!(
                "{}profile/data/by_suid/{}",
                *ACCOUNT_MANAGER_URL,
                suid
            )
        ).await.ok()?.text().await.ok()?;

        /*- Return -*/
        Some(json_fetch)
    }

    /*- Create player wrapper using inner values -*/
    pub fn from_inner(player:Player) -> Self {
        Self { player, ..Default::default() }
    }
}

/*- Default settings -*/
impl Default for PlayerRedisWrapper {
    fn default() -> Self {
        Self {
            player: Player::default(),
            local_data: LocalGameData::default(),
            socket_addr: String::default()
        }
    }
}
impl Default for LocalGameData {
    fn default() -> Self {
        Self {
            board_position: [0, 0]
        }
    }
}

/*- PartialEq for checking if player is in room or not -*/
impl PartialEq for PlayerRedisWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.player.suid == other.player.suid
    }
    fn ne(&self, other: &Self) -> bool {
        self.player.suid != other.player.suid
    }
}


