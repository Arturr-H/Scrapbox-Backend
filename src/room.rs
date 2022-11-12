/*- Imports -*/
use crate::player::Player;
use serde_derive::{Serialize, Deserialize};
use uuid::Uuid;
use redis::{ self, Commands, Connection, ToRedisArgs };
use std::{default::Default, collections::BTreeMap};

/*- Structs, enums & unions -*/
#[derive(Serialize, Debug)]
pub struct Room<'lf> {
    // Room-id for sending room specific websocket data
    pub id          : String,

    // All players in the room, including the leader
    pub players         : Vec<Player<'lf>>,

    // Player bytes
    pub _players        : Vec<u8>,

    // Should be changeable by leader. 
    pub max_players : u8,

    // The creator of the room usually gets this role,
    // however they might leave which will pass this role
    // onto the next player who joined.
    pub leader      : Player<'lf>,

    // Wether the room has started yet or not
    pub started     : bool,

    // Controls wether the room should be visible
    // in the browse rooms section or not.
    pub private     : bool,
}

/*- Method implementations -*/
impl<'a> Room<'a> {

    /*- Create a room with leader as input -*/
    pub fn from_leader(leader:Player<'a>, id:String) -> Self {
        /*- Return -*/
        Self {
            id,
            players: vec![leader],
            _players: Vec::new(),
            max_players: 5,
            leader,
            started: false,
            private: false
        }
    }

    /*- Remove player from room -*/
    pub fn remove_player(&mut self, player:Player<'a>) -> Result<(), ()> {
        /*- First check if player is leader -*/
        if player.suid == self.leader.suid {
            self.players.remove(0);

            /*- Get next player as leader -*/
            self.leader = match self.players.iter().next() {
                Some(player) => *player,
                None => return Err(self.disbandon())
            };

            Ok(())
        }else {
            /*- Iterate over players -*/
            for (index, p) in self.players.iter().enumerate() {
                if p.suid == player.suid {
                    self.players.remove(index);
                    return Ok(())
                };
            };

            Err(())
        }
    }

    /*- Add player to room -*/
    pub fn add_player(&mut self, player:Player<'a>) -> Result<(), ()> {
        /*- If room still fits another player -*/
        if self.players.len() < self.max_players as usize {
            self.players.push(player);
            return Ok(())
        };

        Err(())
    }

    /*- Change room size -*/
    pub fn change_max_players(&mut self, max_players:u8) -> Result<(), ()> {
        /*- If the change of max players won't fit the current amount of players in the room -*/
        if self.players.len() > max_players as usize {
            Err(())
        }else {
            self.max_players = max_players;
            Ok(())
        }
    }

    /*- Change room visibility -*/
    pub fn change_room_visibility(&mut self, private:bool) -> () {
        self.private = private;
    }

    /*- To redis Hash -*/
    pub fn arg<F: ToRedisArgs>(f:F) -> Vec<Vec<u8>> { f.to_redis_args() }
    pub fn to_redis_hash(&mut self) -> Result<[(impl ToRedisArgs, impl ToRedisArgs); 6], ()> {
        self.serialize_players_unchecked();

        Ok([
            ("id",         Self::arg(self.id.as_str())),
            ("leader",     Self::arg(self.leader.to_bytes_unchecked())),
            ("players",    Self::arg(&self._players)),
            ("max-players",Self::arg(self.max_players)),
            ("started",    Self::arg(self.started)),
            ("private",    Self::arg(self.private)),
        ])
    }

    /*- From redis hash -*/
    pub fn make_bool(inner:Option<&String>) -> Option<bool> {
        match inner {
            Some(e) => {
                match e.parse::<u8>() {
                    Ok(e) => {
                        if e == 0 { Some(false) }
                        else { Some(true) }
                    },
                    Err(_) => return None
                }
            },
            None => return None
        }
    }
    pub fn from_redis_hash<'w: 'a>(hash:&'w BTreeMap<String, String>) -> Option<Self> {
        let players = match bincode::deserialize::<Vec<Player>>(
            match &hash.get("players") {
                Some(e) => e,
                None => return None
            }.as_bytes()
        ) {
            Ok(e) => e,
            Err(_) => return None
        };

        /*- Return -*/
        Some(Self {
            id: hash.get("id").unwrap().to_string(),
            players,
            _players: Vec::new(),
            max_players: hash.get("max-players").unwrap().to_string().parse::<u8>().ok().unwrap(),
            leader: Player::from_bytes(&hash.get("leader").unwrap().as_bytes()).ok().unwrap(),
            started: Self::make_bool(hash.get("started"))?,
            private: Self::make_bool(hash.get("private"))?
        })
    }

    /*- Serialize players -*/
    pub fn serialize_players_unchecked(&mut self) -> () {
        /*- Serialize & if fail Err(_) -*/
        self._players = match bincode::serialize(&self.players) {
            Ok(e) => e,
            Err(_) => Vec::new()
        };
    }

    /*- Create ID for room -*/
    pub fn gen_id() -> String {
        Uuid::new_v4().as_hyphenated().to_string()
    }

    /*- Disbandon room -*/
    pub fn disbandon(&mut self) -> () {
        todo!()
    }
}
impl<'lf> Default for Room<'lf> {
    fn default() -> Self {
        Self { 
            id: String::new(), 
            players: Vec::new(), 
            _players: Vec::new(), 
            max_players: 5, 
            leader: Player::default(), 
            started: false, 
            private: false
        }
    }
}
impl ToString for Room<'_> {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or(String::new())
    }
}
