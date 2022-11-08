/*- Imports -*/
use crate::player::Player;
use uuid::Uuid;

/*- Constants -*/


/*- Structs, enums & unions -*/
struct Room<'lf, RSize: Into<u8>> {
    // Room-id for sending room specific websocket data
    id          : String,

    // All players in the room, including the leader
    players     : Vec<Player<'lf>>,

    // Should be changeable by leader. 
    max_players : RSize,

    // The creator of the room usually gets this role,
    // however they might leave which will pass this role
    // onto the next player who joined.
    leader      : Player<'lf>,

    // Wether the room has started yet or not
    started     : bool,

    // Controls wether the room should be visible
    // in the browse rooms section or not.
    private     : bool,
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
    fn from_leader(leader:Player<'a>, max_players:T) -> Self {
        let id:String = Uuid::new_v4().as_hyphenated().to_string();

        /*- Return -*/
        Self {
            id,
            players: vec![leader],
            max_players,
            leader,
            started: false,
            private: false
        }
    }

    /*- Remove player from room -*/
    fn remove_player(&mut self, player:Player<'a>) -> Result<(), ()> {
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
    fn add_player(&mut self, player:Player<'a>) -> Result<(), ()> {
        /*- If room still fits another player -*/
        if self.players.len() < <T as Into<u8>>::into(self.max_players) as usize {
            self.players.push(player);
            return Ok(())
        };

        Err(())
    }

    /*- Change room size -*/
    fn change_max_players(&mut self, max_players:T) -> Result<(), ()> {
        /*- If the change of max players won't fit the current amount of players in the room -*/
        if self.players.len() > max_players.into().into() {
            Err(())
        }else {
            self.max_players = max_players;
            Ok(())
        }
    }

    /*- Change room visibility -*/
    fn change_room_visibility(&mut self, private:bool) -> () {
        self.private = private;
    }

    /*- Disbandon room -*/
    fn disbandon(&mut self) -> () {
        todo!()
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
