/*- Global allowings -*/

/*- Imports -*/
use crate::{ player::Player as PlayerInner, wrapper::PlayerRedisWrapper as PlayerWrpd, ws_status, ACCOUNT_MANAGER_URL };
use serde_derive::{ Serialize, Deserialize };
use reqwest;


/*- Struct for retrieving suid data from token auth response -*/
#[derive(Serialize, Deserialize, Debug)]
struct SuidResponse {
    suid:String
}

/*- Functions -*/
pub async fn authorize_player<'a>(jwt:&'a str) -> Result<PlayerWrpd, u16> {
    let token_check_url = format!("{}profile/verify-token", &*ACCOUNT_MANAGER_URL);

    /*- Check player auth -*/
    let suid:String = match reqwest::Client::new()
        .get(token_check_url)
        .header("token", jwt).send().await {

        /*- If request succeeded -*/
        Ok(response) => {
            match response.text().await {
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
    println!("SUID: {suid}");
    let fetched_player = PlayerWrpd::fetch_player(&suid).await;
    match fetched_player {
        Some(string) => {

            /*- Deserialize the player data and wrap it in a wrapper -*/
            println!("TOOO {}", string.as_str());
            Ok(match serde_json::from_str::<PlayerInner>(string.as_str()) {
                Ok(e) => PlayerWrpd::from_inner(e),
                Err(e) => panic!("{e}")
            })
        },
        None => return Err(ws_status::PLAYER_PARSE)
    }
}
