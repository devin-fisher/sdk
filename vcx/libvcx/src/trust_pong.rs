extern crate serde_json;

use object_cache::ObjectCache;
use api::{ VcxStateType };
use utils::error;
use connection;
use messages;
use messages::GeneralMessage;
use messages::extract_json_payload;
use messages::to_u8;

use utils::libindy::wallet;
use utils::libindy::crypto;

use utils::option_util::expect_ok_or;

use serde_json::Value;

use settings;
use utils::httpclient;
use utils::constants::SEND_MESSAGE_RESPONSE;

lazy_static! {
    static ref HANDLE_MAP: ObjectCache<Pong>  = Default::default();
}

impl Default for Pong {
    fn default() -> Pong
    {
        Pong {
            source_id: String::new(),
            msg_uid: None,
            my_did: None,
            my_vk: None,
            ping: None,
            state: VcxStateType::VcxStateNone,
            their_did: None,
            their_vk: None,
            agent_did: None,
            agent_vk: None,
        }
    }
}

pub fn _value_from_json(value: Option<&Value>, key: &str, none_warning: &str, error_val: u32)
    -> Result<String, u32> {

    let rtn = expect_ok_or(value,
                                   none_warning,
                                   error_val)?;
    let rtn = expect_ok_or(rtn.get(key),
                           none_warning,
                           error_val)?;

    match rtn {
        &Value::String(ref s) => Ok(s.to_owned()),
        _ => Err(error::INVALID_JSON.code_num)
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Pong {
    source_id: String,
    msg_uid: Option<String>,
    my_did: Option<String>,
    my_vk: Option<String>,
    ping: Option<Value>,
    state: VcxStateType,
    their_did: Option<String>,
    their_vk: Option<String>,
    agent_did: Option<String>,
    agent_vk: Option<String>,
}

impl Pong {

    fn get_state(&self) -> u32 {self.state as u32}
    fn set_state(&mut self, state: VcxStateType) {self.state = state}

    fn send_pong(&mut self, connection_handle: u32) -> Result<u32, u32> {

        info!("sending pong via connection connection: {}", connection_handle);
        self.my_did = Some(connection::get_pw_did(connection_handle)?);
        self.my_vk = Some(connection::get_pw_verkey(connection_handle)?);
        self.agent_did = Some(connection::get_agent_did(connection_handle)?);
        self.agent_vk = Some(connection::get_agent_verkey(connection_handle)?);
        self.their_did = Some(connection::get_their_pw_did(connection_handle)?);
        self.their_vk = Some(connection::get_their_pw_verkey(connection_handle)?);


        debug!("verifier_did: {:?} -- verifier_vk: {:?} -- agent_did: {:?} -- agent_vk: {:?} -- remote_vk: {:?}",
               self.my_did,
               self.agent_did,
               self.agent_vk,
               self.their_vk,
               self.my_vk);

        let e_code: u32 = 10;

        let local_their_did = self.their_did.as_ref().ok_or(e_code)?;
        let local_their_vk = self.their_vk.as_ref().ok_or(e_code)?;
        let local_agent_did = self.agent_did.as_ref().ok_or(e_code)?;
        let local_agent_vk = self.agent_vk.as_ref().ok_or(e_code)?;
        let local_my_did = self.my_did.as_ref().ok_or(e_code)?;
        let local_my_vk = self.my_vk.as_ref().ok_or(e_code)?;

//        msg_uid
        let ref_msg_uid = _value_from_json(self.ping.as_ref(), "msg_uid", "", e_code)?;
        let nonce = _value_from_json(self.ping.as_ref(), "nonce", "", e_code)?;

        let pong = json!({"nonce": nonce, "msg_type": "TRUST_PONG"});
        let pong = serde_json::to_string(&pong).or(Err(e_code))?;
        let data: Vec<u8> = connection::generate_encrypted_payload(local_my_vk, local_their_vk, &pong, "TRUST_PONG")?;
        if settings::test_agency_mode_enabled() { httpclient::set_next_u8_response(SEND_MESSAGE_RESPONSE.to_vec());}

        match messages::send_message().to(local_my_did)
            .to_vk(local_my_vk)
            .msg_type("trustPong")
            .agent_did(local_agent_did)
            .agent_vk(local_agent_vk)
            .edge_agent_payload(&data)
            .ref_msg_id(&ref_msg_uid)
            .send_secure() {
            Ok(response) => {
                self.state = VcxStateType::VcxStateAccepted;
                return Ok(error::SUCCESS.code_num)
            },
            Err(x) => {
                warn!("could not send pong: {}", x);
                return Err(x);
            }
        }
    }

    fn update_state(&mut self) {
        match self.state {
            VcxStateType::VcxStateSent => {
                //Check for messages
            },
            VcxStateType::VcxStateAccepted => {
                //Check for revocation
            }
            _ => {
                // NOOP there is nothing the check for a changed state
            }
        }
    }
}

//********************************************
//         HANDLE FUNCTIONS
//********************************************
fn handle_err(code_num: u32) -> u32 {
    if code_num == error::INVALID_OBJ_HANDLE.code_num {
        error::INVALID_OBJ_HANDLE.code_num
    }
    else {
        code_num
    }
}

pub fn create_pong(source_id: Option<String>, ping: &str) -> Result<u32, u32> {
    info!("creating ping with id: {}", source_id.unwrap_or("UNDEFINED".to_string()));

    let mut new_obj: Pong = Default::default();

    let ping = serde_json::from_str(ping)
        .map_err(|_|error::INVALID_JSON.code_num)?;

    new_obj.ping = Some(ping);

    new_obj.set_state(VcxStateType::VcxStateInitialized);

    Ok(HANDLE_MAP.add(new_obj)?)
}

pub fn get_state(handle: u32) -> Result<u32, u32> {
    HANDLE_MAP.get(handle, |obj| {
        Ok(obj.get_state())
    }).map_err(handle_err)
}

pub fn update_state(handle: u32) -> Result<u32, u32> {
    HANDLE_MAP.get_mut(handle, |obj|{
        obj.update_state();
        Ok(error::SUCCESS.code_num)
    })

}

pub fn to_string(handle: u32) -> Result<String, u32> {
    HANDLE_MAP.get(handle, |obj|{
        serde_json::to_string(&obj).map_err(|e|{
            warn!("Unable to serialize: {:?}", e);
            error::SERIALIZATION_ERROR.code_num
        })
    })
}

pub fn from_string(data: &str) -> Result<u32, u32> {
    let derived: Pong = match serde_json::from_str(data) {
        Ok(x) => x,
        Err(y) => return Err(error::INVALID_JSON.code_num),
    };

    let new_handle = HANDLE_MAP.add(derived)?;

    info!("inserting handle {} into pong table", new_handle);

    Ok(new_handle)
}

pub fn release(handle: u32) -> Result<(), u32> {
    HANDLE_MAP.release(handle).map_err(handle_err)
}

pub fn send_pong(handle: u32, connection_handle: u32) -> Result<u32,u32> {
    HANDLE_MAP.get_mut(handle, |obj|{
        obj.send_pong(connection_handle)
    })
}

pub fn is_valid_handle(handle: u32) -> bool {
    HANDLE_MAP.has_handle(handle)
}

//TODO one function with claim
pub fn new_ping_messages(connection_handle: u32, match_name: Option<&str>) -> Result<String, u32> {
    let my_did = connection::get_pw_did(connection_handle)?;
    let my_vk = connection::get_pw_verkey(connection_handle)?;
    let agent_did = connection::get_agent_did(connection_handle)?;
    let agent_vk = connection::get_agent_verkey(connection_handle)?;

    let payload = messages::get_message::get_all_message(&my_did,
                                                         &my_vk,
                                                         &agent_did,
                                                         &agent_vk)?;

    let mut messages: Vec<Value> = Default::default();

    for msg in payload {
        if msg.msg_type.eq("trustPing") {
            let msg_data = match msg.payload {
                Some(ref data) => {
                    let data = to_u8(data);
                    crypto::parse_msg(wallet::get_wallet_handle(), &my_vk, data.as_slice())?
                },
                None => return Err(10) // TODO better error
            };

            let req = extract_json_payload(&msg_data)?;

            println!("{:?}", req);

            let mut req: Value = serde_json::from_str(&req)
                .or(Err(error::INVALID_JSON.code_num))?;

            if let Value::Object(ref mut map) = req {
                let uid = json!(msg.uid);
                map.insert(String::from("msg_uid"), uid);
            }

            if req.is_object() {
                messages.push(req);
            }

        }
    }


    Ok(serde_json::to_string_pretty(&messages).unwrap())
}

#[cfg(test)]
mod tests {
//    use super::*;


    #[test]
    fn test_noop() {

    }
}
