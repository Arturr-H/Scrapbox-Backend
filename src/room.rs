/*- Imports -*/
use crate::{ player::Player as PlayerInner, wrapper::PlayerRedisWrapper as PlayerWrpd };
use rand::Rng;
use serde_derive::{ Serialize, Deserialize };
use uuid::Uuid;
use mongodb;
use std::{ default::Default, collections::{BTreeMap, btree_map::Range}, net::SocketAddr };

/*- Constants -*/
mod private_room_id_range {
    pub const MAX: u32 = 100_000;
    pub const MIN: u32 = 10_000;

    pub const RANGE:std::ops::Range<u32> = 10_000..100_000;
}

/*- Structs, enums & unions -*/
#[derive(Serialize, Debug)]
pub struct Room {
    // Room-id for sending room specific websocket data. Players
    // will recieve this id upon joining a room and connecting
    // to the websocket server via the id.
    pub private_id      : String,

    // Room-id for easy access to rooms. Will be used for joining rooms via URL:S
    pub public_id       : u32,

    // All players in the room, including the leader
    pub players         : Vec<PlayerWrpd>,

    // Player bytes
    pub _players        : Vec<u8>,

    // Should be changeable by leader. 
    pub max_players : u8,

    // The creator of the room usually gets this role,
    // however they might leave which will pass this role
    // onto the next player who joined.
    pub leader      : PlayerWrpd,

    // Wether the room has started yet or not
    pub started     : bool,

    // Controls wether the room should be visible
    // in the browse rooms section or not.
    pub private     : bool,

    // A vector containing players' socket addresses. Will make searching
    // for players in the room easier because there won't be any need for
    // deserializing the players vector. And iterating over it. If adress
    // is not found in this vector, the player is not in the room. And no
    // need to iterate over the vector.
    pub player_adresses : Vec<String>,
}

/*- Method implementations -*/
impl Room {

    /*- Create a room with leader as input -*/
    pub fn from_leader(leader:PlayerWrpd, private_id:String, public_id:u32) -> Self {

        /*- Return -*/
        Self {
            private_id,
            public_id,
            players: vec![leader.clone()],
            _players: Vec::new(),
            max_players: 5,
            leader: leader.clone(),
            started: false,
            private: false,
            player_adresses: vec![leader.socket_addr],
        }
    }

    /*- Remove player from room -*/
    pub fn remove_player(&mut self, player:PlayerWrpd) -> Result<(), ()> {
        /*- First check if player is leader -*/
        if player.player.suid == self.leader.player.suid {
            self.players.remove(0);

            /*- Get next player as leader -*/
            self.leader = match self.players.iter().next() {
                Some(player) => player.clone(),
                None => return Err(self.disbandon())
            };

            Ok(())
        }else {
            /*- Iterate over players -*/
            for (index, p) in self.players.iter().enumerate() {
                if p.player.suid == player.player.suid {
                    self.players.remove(index);
                    return Ok(())
                };
            };

            Err(())
        }
    }

    /*- Add player to room -*/
    pub fn add_player(&mut self, player:PlayerWrpd) -> Result<(), ()> {
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
    // pub fn arg<F: ToRedisArgs>(f:F) -> Vec<Vec<u8>> { f.to_redis_args() }
    // pub fn to_redis_hash(&mut self) -> Result<[(impl ToRedisArgs, impl ToRedisArgs); 8], ()> {
    //     self.serialize_players_unchecked();

    //     Ok([
    //         ("private_id", Self::arg(self.private_id.as_str())),
    //         ("public_id",  Self::arg(self.public_id.to_string())),
    //         ("leader",     Self::arg(self.leader.to_bytes_unchecked())),
    //         ("players",    Self::arg(&self._players)),
    //         ("max-players",Self::arg(self.max_players)),
    //         ("started",    Self::arg(self.started)),
    //         ("private",    Self::arg(self.private)),
    //         ("player_adresses", Self::arg(self.player_adresses)),
    //     ])
    // }

    /*- From redis hash -*/
    // pub fn make_bool(inner:Option<&String>) -> Option<bool> {
    //     match inner {
    //         Some(e) => {
    //             match e.parse::<u8>() {
    //                 Ok(e) => {
    //                     if e == 0 { Some(false) }
    //                     else { Some(true) }
    //                 },
    //                 Err(_) => return None
    //             }
    //         },
    //         None => return None
    //     }
    // }
    // pub fn from_redis_hash<'w>(hash:&'w BTreeMap<String, String>) -> Option<Self> {
    //     if hash.is_empty() { return None };
    //     let players = match bincode::deserialize::<Vec<PlayerWrpd>>(
    //         match &hash.get("players") {
    //             Some(e) => e,
    //             None => return None
    //         }.as_bytes()
    //     ) {
    //         Ok(e) => e,
    //         Err(a) => return None
    //     };

    //     /*- Return -*/
    //     Some(Self {
    //         private_id: hash.get("private_id")?.to_string(),
    //         public_id:  hash.get("public_id")?.parse::<u32>().ok()?,
    //         players,
    //         _players: Vec::new(),
    //         max_players: hash.get("max-players")?.to_string().parse::<u8>().ok()?,
    //         leader: PlayerWrpd::from_bytes(&hash.get("leader")?.as_bytes()).ok()?,
    //         started: Self::make_bool(hash.get("started"))?,
    //         private: Self::make_bool(hash.get("private"))?,
    //         player_adresses: Vec::new(),
    //     })
    // }

    /*- Serialize players -*/
    pub fn serialize_players_unchecked(&mut self) -> () {
        /*- Serialize & if fail Err(_) -*/
        self._players = match bincode::serialize(&self.players) {
            Ok(e) => e,
            Err(_) => Vec::new()
        };
    }

    /*- Create ID for room -*/
    pub fn gen_private_id() -> String {
        Uuid::new_v4().as_hyphenated().to_string()
    }
    pub fn gen_public_id() -> u32 {
        rand::thread_rng().gen_range(private_room_id_range::RANGE)
    }

    /*- Debugging -*/
    pub fn quick_display(&self, title:&str) -> () {
        let s:usize = 28 + self.leader.player.username.len() + self.public_id.to_string().len() + self.players.len().to_string().len();
        println!("┌{}┐", "─".repeat(s));
        println!("│{:^width$}│", title, width = s);
        println!("├{}┤", "─".repeat(s));
        println!("│ {}'s room | room:{} | {} players │", self.leader.player.username, self.public_id, self.players.len());
        println!("└{}┘", "─".repeat(s));
    }

    /*- Disbandon room -*/
    pub fn disbandon(&mut self) -> () {
        todo!()
    }
}
impl Default for Room {
    fn default() -> Self {
        Self { 
            private_id: String::new(), 
            public_id: 10_000, 
            players: Vec::new(), 
            _players: Vec::new(), 
            max_players: 5, 
            leader: PlayerWrpd::default(), 
            started: false, 
            private: false,
            player_adresses: Vec::new()
        }
    }
}
impl ToString for Room {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or(String::new())
    }
}
