/*- Imports -*/
use crate::player::Player;
use uuid::Uuid;
use redis::{ self, Commands, Connection, ToRedisArgs };
use std::default::Default;

/*- Structs, enums & unions -*/
pub struct Room<'lf, RSize: Into<u8>> {
    // Room-id for sending room specific websocket data
    pub id          : String,

    // All players in the room, including the leader
    pub players         : Vec<Player<'lf>>,

    // Player bytes
    pub _players        : Vec<u8>,

    // Should be changeable by leader. 
    pub max_players : RSize,

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

// Numbers of max players in room
#[derive(Clone, Copy)]
enum RoomSize {
    /// 2
    Duo,
    
    /// 4
    Small,
    
    /// 5
    Medium,
    
    /// 6
    Big,
    
    /// 10
    Grande
}

/*- Method implementations -*/
impl<'a, T: Into<u8> + Copy> Room<'a, T> {

    /*- Create a room with leader as input -*/
    pub fn from_leader(leader:Player<'a>, max_players:T) -> Self {
        let id:String = Uuid::new_v4().as_hyphenated().to_string();

        /*- Return -*/
        Self {
            id,
            players: vec![leader],
            _players: Vec::new(),
            max_players,
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
        if self.players.len() < <T as Into<u8>>::into(self.max_players) as usize {
            self.players.push(player);
            return Ok(())
        };

        Err(())
    }

    /*- Change room size -*/
    pub fn change_max_players(&mut self, max_players:T) -> Result<(), ()> {
        /*- If the change of max players won't fit the current amount of players in the room -*/
        if self.players.len() > max_players.into().into() {
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
            ("max-players",Self::arg(self.max_players.into())),
            ("started",    Self::arg(self.started)),
            ("private",    Self::arg(self.private)),
        ])
    }

    /*- Serialize players -*/
    pub fn serialize_players_unchecked(&mut self) -> () {
        /*- Serialize & if fail Err(_) -*/
        self._players = match bincode::serialize(&self.players) {
            Ok(e) => e,
            Err(_) => Vec::new()
        };
    }

    /*- Disbandon room -*/
    pub fn disbandon(&mut self) -> () {
        todo!()
    }
}
impl<'lf> Default for Room<'lf, u8> {
    fn default() -> Self {
        Self { 
            id: "".into(), 
            players: Vec::new(), 
            _players: Vec::new(), 
            max_players: 5, 
            leader: Player::default(), 
            started: false, 
            private: false
        }
    }
}

/*- Methods for enum RoomSize -*/
impl Into<u8> for RoomSize {
    fn into(self) -> u8 {
        match self {
            Self::Duo => 2,
            Self::Small => 4,
            Self::Medium => 6,
            Self::Big => 8,
            Self::Grande => 12
        }
    }
}
