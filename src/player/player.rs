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

/*- Game data, only lives for the lifetime of a game -*/
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct LocalGameData {
    pub games_won    : u32,
    pub games_played : u32,
    pub words_written: u32
}

/*- Method implementations -*/
impl Player {
    pub fn new() -> Self {
        Self { ..Self::default() }
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


