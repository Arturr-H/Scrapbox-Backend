/*- Global allowings -*/
#![allow(
    dead_code
)]

/*- Imports -*/
use serde_derive::{ Serialize, Deserialize };
use std::default::Default;
use crate::ACCOUNT_MANAGER_URL;

/*- Structs, enums & unions -*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub suid: String,

    // Lowercase name, can be occupied by others
    pub username: String,

    // Name with special characters etc.
    // Displayed in normal circumstances
    pub displayname: String,

    // The player's statistics
    pub statistics : GameStatistics
}

/*- Game statistics -*/
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct GameStatistics {
    pub games_won    : u32,
    pub games_played : u32,
    pub words_written: u32
}

/*- Method implementations -*/
impl Player {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    /*- Bincode deserialization for transport in websocket tunnels -*/
    pub fn from_bytes<'lf>(input: &'lf [u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        /*- Deserialize & if fail return -*/
        let player:Player = bincode::deserialize(input)?;

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
}

/*- Default settings -*/
impl Default for Player {
    fn default() -> Self {
        Self { suid: String::new(), username: String::new(), displayname: String::new(), statistics: GameStatistics::default() }
    }
}
impl Default for GameStatistics {
    fn default() -> Self {
        GameStatistics { games_won: 0, games_played: 0, words_written: 0 }
    }
}

/*- PartialEq for checking if player is in room or not -*/
impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.suid == other.suid
    }
    fn ne(&self, other: &Self) -> bool {
        self.suid != other.suid
    }
}


