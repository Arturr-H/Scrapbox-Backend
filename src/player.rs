/*- Global allowings -*/
#![allow(
    dead_code
)]

/*- Imports -*/
use serde_derive::{ Serialize, Deserialize };
use std::default::Default;

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
    pub fn username(&mut self, username:&'lf str) ->       &mut Self { self.username = Some(username); self }
    pub fn displayname(&mut self, displayname:&'lf str) -> &mut Self { self.displayname = Some(displayname); self }
    pub fn suid(&mut self, suid:&'lf str) ->               &mut Self { self.suid = Some(suid); self }

    /*- Bincode deserialization for transport in websocket tunnels -*/
    pub fn from_bytes(input: &'lf [u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        /*- Deserialize & if fail return -*/
        let player:Player = bincode::deserialize(input)?;

        /*- return -*/
        Ok(player)
    }

    /*- Bincode serialization for transport -*/
    pub fn to_bytes(input: Player) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
        /*- Serialize & if fail Err(_) -*/
        let bytes:Vec<u8> = bincode::serialize(&input)?;

        /*- Return -*/
        Ok(bytes)
    }
}

/*- Default settings -*/
impl<'f> Default for Player<'f> {
    fn default() -> Self {
        Self { suid: None, username: None, displayname: None }
    }
}
