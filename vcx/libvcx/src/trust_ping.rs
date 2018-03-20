extern crate rand;
extern crate serde_json;
extern crate libc;

use connection;
use rand::Rng;
use api::{ VcxStateType };
use std::sync::Mutex;
use std::collections::HashMap;
use messages;
use messages::GeneralMessage;
use messages::send_message::parse_msg_uid;
use utils::error;
use proof::generate_nonce;

use serde_json::Value;

lazy_static! {
    static ref HANDLE_MAP: Mutex<HashMap<u32, Box<Ping>>> = Default::default();
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Ping {
    source_id: String,
    handle: u32,
    msg_uid: String,
    ref_msg_id: String,
    prover_did: String,
    prover_vk: String,
    state: VcxStateType,
    nonce: String,
    remote_did: String,
    remote_vk: String,
    agent_did: String,
    agent_vk: String,
}

impl Ping {
    fn send_ping_request(&mut self, connection_handle: u32) -> Result<u32, u32> {
        if self.state != VcxStateType::VcxStateInitialized {
            warn!("ping {} has invalid state {} for sending request", self.handle, self.state as u32);
            return Err(error::NOT_READY.code_num);
        }

        info!("sending ping request: {}, and connection {}", self.handle, connection_handle);
        self.prover_did = connection::get_pw_did(connection_handle)?;
        self.agent_did = connection::get_agent_did(connection_handle)?;
        self.agent_vk = connection::get_agent_verkey(connection_handle)?;
        self.remote_vk = connection::get_their_pw_verkey(connection_handle)?;
        self.prover_vk = connection::get_pw_verkey(connection_handle)?;

        debug!("prover_did: {} -- agent_did: {} -- agent_vk: {} -- remote_vk: {} -- prover_vk: {}",
               self.prover_did,
               self.agent_did,
               self.agent_vk,
               self.remote_vk,
               self.prover_vk);

        let request = json!({"nonce": &self.nonce, "msg_type": "TRUST_PING"});
        let request = serde_json::to_string(&request).unwrap();

        let data = connection::generate_encrypted_payload(&self.prover_vk, &self.remote_vk, &request, "TRUST_PING")?;

        match messages::send_message().to(&self.prover_did)
            .to_vk(&self.prover_vk)
            .msg_type("trustPing")
            .agent_did(&self.agent_did)
            .agent_vk(&self.agent_vk)
            .edge_agent_payload(&data)
            .send_secure() {
            Ok(response) => {
                self.msg_uid = parse_msg_uid(&response[0])?;
                self.state = VcxStateType::VcxStateSent;
                return Ok(error::SUCCESS.code_num)
            },
            Err(x) => {
                warn!("could not send trustPing: {}", x);
                return Err(x);
            }
        }
    }

    fn get_pong_request_status(&mut self) -> Result<u32, u32> {
        info!("updating state for ping {}", self.handle);
        if self.state == VcxStateType::VcxStateAccepted {
            return Ok(error::SUCCESS.code_num);
        }
        else if self.state != VcxStateType::VcxStateSent || self.msg_uid.is_empty() || self.prover_did.is_empty() {
            return Ok(error::SUCCESS.code_num);
        }

        let payload = messages::get_message::get_ref_msg(&self.msg_uid, &self.prover_did, &self.prover_vk, &self.agent_did, &self.agent_vk)?;

        let pong = match parse_trust_payload(&payload) {
            Err(_) => return Ok(error::SUCCESS.code_num),
            Ok(x) => x,
        };

        if let Value::String(ref nonce) = pong["nonce"] {
            let local_nonce = &self.nonce;
            if nonce.eq(&self.nonce) {
                self.state = VcxStateType::VcxStateAccepted;
            }
        }
        Ok(error::SUCCESS.code_num)
    }

    fn update_state(&mut self) {
        self.get_pong_request_status().unwrap_or(error::SUCCESS.code_num);
    }

    fn get_state(&self) -> u32 {let state = self.state as u32; state}
}

pub fn create_ping(source_id: Option<String>) -> Result<u32, u32> {

    let new_handle = rand::thread_rng().gen::<u32>();

    let source_id_unwrap = source_id.unwrap_or("".to_string());

    let mut new_ping = Box::new(Ping {
        handle: new_handle,
        source_id: source_id_unwrap,
        msg_uid: String::new(),
        ref_msg_id: String::new(),
        prover_did: String::new(),
        prover_vk: String::new(),
        state: VcxStateType::VcxStateNone,
        nonce: generate_nonce()?,
        remote_did: String::new(),
        remote_vk: String::new(),
        agent_did: String::new(),
        agent_vk: String::new(),
    });

    new_ping.state = VcxStateType::VcxStateInitialized;

    {
        let mut m = HANDLE_MAP.lock().unwrap();
        info!("inserting handle {} into ping table", new_handle);
        m.insert(new_handle, new_ping);
    }

    Ok(new_handle)
}

pub fn is_valid_handle(handle: u32) -> bool {
    match HANDLE_MAP.lock().unwrap().get(&handle) {
        Some(_) => true,
        None => false,
    }
}

pub fn update_state(handle: u32) {
    match HANDLE_MAP.lock().unwrap().get_mut(&handle) {
        Some(t) => t.update_state(),
        None => {}
    };
}

pub fn get_state(handle: u32) -> u32 {
    match HANDLE_MAP.lock().unwrap().get(&handle) {
        Some(t) => t.get_state(),
        None => VcxStateType::VcxStateNone as u32,
    }
}

pub fn release(handle: u32) -> u32 {
    match HANDLE_MAP.lock().unwrap().remove(&handle) {
        Some(t) => error::SUCCESS.code_num,
        None => error::INVALID_OBJ_HANDLE.code_num,
    }
}

pub fn to_string(handle: u32) -> Result<String, u32> {
    match HANDLE_MAP.lock().unwrap().get(&handle) {
        Some(p) => Ok(serde_json::to_string(&p).unwrap().to_owned()),
        None => Err(error::INVALID_OBJ_HANDLE.code_num)
    }
}

pub fn from_string(data: &str) -> Result<u32, u32> {
    let derived: Ping = match serde_json::from_str(data) {
        Ok(x) => x,
        Err(y) => return Err(error::INVALID_JSON.code_num),
    };
    let new_handle = derived.handle;

    if is_valid_handle(new_handle) {return Ok(new_handle);}
    let ping = Box::from(derived);

    {
        let mut m = HANDLE_MAP.lock().unwrap();
        info!("inserting handle {} into ping table", new_handle);
        m.insert(new_handle, ping);
    }
    Ok(new_handle)
}

pub fn send_ping_request(handle: u32, connection_handle: u32) -> Result<u32,u32> {
    match HANDLE_MAP.lock().unwrap().get_mut(&handle) {
        Some(c) => Ok(c.send_ping_request(connection_handle)?),
        None => Err(error::INVALID_OBJ_HANDLE.code_num),
    }
}

fn parse_trust_payload(payload: &Vec<u8>) -> Result<Value, u32> {
    debug!("parsing pong payload: {:?}", payload);
    let data = messages::extract_json_payload(payload)?;

    match serde_json::from_str(&data) {
        Ok(x) => Ok(x),
        Err(x) => {
            warn!("invalid json {}", x);
            Err(error::INVALID_JSON.code_num)
        }
    }
}

#[cfg(test)]
mod tests {
//    use super::*;

    #[test]
    fn test_noop() {

    }


}
