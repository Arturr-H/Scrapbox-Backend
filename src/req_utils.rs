/*- Global allowings -*/

/*- Imports -*/
use crate::{ player::Player, ws_status, ACCOUNT_MANAGER_URL };
use serde_derive::{ Serialize, Deserialize };
use reqwest;


/*- Struct for retrieving suid data from token auth response -*/
#[derive(Serialize, Deserialize, Debug)]
struct SuidResponse {
    suid:String
}

/*- Functions -*/
pub fn authorize_player<'a>(jwt:&'a str) -> Result<Player, u16> {
    let token_check_url = format!("{}profile/verify-token", &*ACCOUNT_MANAGER_URL);

    /*- Check player auth -*/
    let suid:String = match reqwest::blocking::Client::new()
        .get(token_check_url)
        .header("token", jwt).send() {

        /*- If request succeeded -*/
        Ok(response) => {
            match response.text() {
                Ok(text) => {
                    /*- Parse response to SUID value -*/
                    match serde_json::from_str::<SuidResponse>(&text) {
                        Ok(suid_response) => suid_response.suid,
                        Err(_) => return Err(ws_status::PARSE_ACCOUNT_API_RES_TEXT)
                    }
                },
                Err(_) => return Err(ws_status::UNAUTHORIZED)
            }
        },
        Err(_) => return Err(ws_status::PARSE_ACCOUNT_API_RES)
    };

    /*- Get player -*/
    let fetched_player = Player::fetch_player(&suid);
    match fetched_player {
        Some(string) => {
            Ok(match serde_json::from_str::<Player>(string.as_str()) {
                Ok(e) => e,
                Err(e) => panic!("{e}")
            })
        },
        None => return Err(ws_status::PLAYER_PARSE)
    }
}
