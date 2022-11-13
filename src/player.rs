/*- Global allowings -*/
#![allow(
    dead_code
)]

/*- Imports -*/
use serde_derive::{ Serialize, Deserialize };
use std::default::Default;
use crate::ACCOUNT_MANAGER_URL;

/*- Structs, enums & unions -*/
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Player<'lf> {
    pub suid: Option<&'lf str>,

    // Lowercase name, can be occupied by others
    pub username: Option<&'lf str>,

    // Name with special characters etc.
    // Displayed in normal circumstances
    pub displayname: Option<&'lf str>,

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
impl<'lf> Player<'lf> {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }

    /*- Builder pattern -*/
    pub fn username(mut self, username:&'lf str) ->       Self { self.username = Some(username); self }
    pub fn displayname(mut self, displayname:&'lf str) -> Self { self.displayname = Some(displayname); self }
    pub fn suid(mut self, suid:&'lf str) ->               Self { self.suid = Some(suid); self }

    /*- Bincode deserialization for transport in websocket tunnels -*/
    pub fn from_bytes(input: &'lf [u8]) -> Result<Self, Box<bincode::ErrorKind>> {
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
    pub fn check_auth(jwt:&str) -> u16 {
            /*- GET JWT auth status -*/
        let url = format!("{}{}", &**ACCOUNT_MANAGER_URL, "profile/verify-token");
        match reqwest::blocking::Client::new()
            .get(&url)
            .header("token", jwt)
            .send()
            { Ok(e) => e, Err(_) => return 402u16 }
            .status()
            .as_u16()
    }

    /*- Fetch player data by SUID -*/
    pub fn fetch_player(suid:&str) -> Option<String> {
        println!("{}",             format!(
            "{}profile/data/by_suid/{}",
            *ACCOUNT_MANAGER_URL,
            suid
        ));
        /*- Get JSON data -*/
        let json_fetch:String = reqwest::blocking::get(
            format!(
                "{}profile/data/by_suid/{}",
                *ACCOUNT_MANAGER_URL,
                suid
            )
        ).ok()?.text().ok()?;

        /*- Return -*/
        Some(json_fetch)
    }
}

/*- Default settings -*/
impl<'f> Default for Player<'f> {
    fn default() -> Self {
        Self { suid: None, username: None, displayname: None, statistics: GameStatistics::default() }
    }
}
impl Default for GameStatistics {
    fn default() -> Self {
        GameStatistics { games_won: 0, games_played: 0, words_written: 0 }
    }
}

/*- PartialEq for checking if player is in room or not -*/
impl<'a> PartialEq for Player<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.suid == other.suid
    }
    fn ne(&self, other: &Self) -> bool {
        self.suid != other.suid
    }
}


