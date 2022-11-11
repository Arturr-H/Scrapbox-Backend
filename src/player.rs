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
            /*- GET JWS auth status -*/
        let url = format!("{}{}", &**ACCOUNT_MANAGER_URL, "profile/verify-token");
        match reqwest::blocking::Client::new()
            .get(&url)
            .header("token", jwt)
            .send()
            { Ok(e) => e, Err(_) => return 402u16 }
            .status()
            .as_u16()
    }
}

/*- Default settings -*/
impl<'f> Default for Player<'f> {
    fn default() -> Self {
        Self { suid: None, username: None, displayname: None }
    }
}
