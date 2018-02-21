extern crate rand;
extern crate serde_json;
extern crate libc;

use object_cache::ObjectCache;
use api::VcxStateType;
use utils::error;
//use messages;
//use settings;
//use messages::GeneralMessage;
//use messages::MessageResponseCode::{ MessageAccepted };
//use connection;
use claim_request::ClaimRequest;
//use utils::libindy::wallet;
//use utils::httpclient;
//use utils::constants::SEND_CLAIM_OFFER_RESPONSE;

lazy_static! {
    static ref HANDLE_MAP: ObjectCache<Claim>  = Default::default();
}

impl Default for Claim {
    fn default() -> Claim
    {
        Claim {
            source_id: String::new(),
            state: VcxStateType::VcxStateNone,
            claim_request: None,
            agent_did: None,
            agent_vk: None,
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Claim {
    source_id: String,
    state: VcxStateType,
    claim_request: Option<ClaimRequest>,
    // the following 6 are pulled from the connection object
    agent_did: Option<String>,
    agent_vk: Option<String>,
    my_did: Option<String>,
    my_vk: Option<String>,
    their_did: Option<String>,
    their_vk: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClaimOffer {
    msg_type: String,
    version: String,
    to_did: String,
    from_did: String,
    claim: serde_json::Map<String, serde_json::Value>,
    schema_seq_no: u32,
    issuer_did: String,
    claim_name: String,
    claim_id: String,

}

impl Claim {

//    fn get_claim_offer_status(&mut self) -> Result<u32, u32> {
//        info!("updating state for claim offer: {}", self.handle);
//        if self.state == VcxStateType::VcxStateRequestReceived {
//            return Ok(error::SUCCESS.code_num);
//        }
//            else if self.state != VcxStateType::VcxStateOfferSent || self.msg_uid.is_empty() || self.issued_did.is_empty() {
//                return Ok(error::SUCCESS.code_num);
//            }
//
//        let payload = messages::get_message::get_ref_msg(&self.msg_uid, &self.issued_did, &self.issued_vk, &self.agent_did, &self.agent_vk)?;
//
//        self.claim_request = Some(parse_claim_req_payload(&payload)?);
//        info!("received claim request for claim offer: {}", self.handle);
//        self.state = VcxStateType::VcxStateRequestReceived;
//        Ok(error::SUCCESS.code_num)
//    }

    fn update_state(&mut self) {
//        self.get_claim_offer_status().unwrap_or(error::SUCCESS.code_num);
        //There will probably be more things here once we do other things with the claim
    }

    fn get_state(&self) -> u32 { let state = self.state as u32; state }

    fn set_source_id(&mut self, id: String) {
        self.source_id = id;
    }
}

//********************************************
//         HANDLE FUNCTIONS
//********************************************
fn handle_err(code_num: u32) -> u32 {
    if code_num == error::INVALID_OBJ_HANDLE.code_num {
//        error::INVALID_CLAIM_HANDLE.code_num // TODO make a error
        10
    }
    else {
        code_num
    }
}

pub fn claim_create_with_offer(source_id: Option<String>, offer: &str) -> Result<u32, u32> {
    let new_claim = _claim_create(source_id);

    info!("inserting claim into handle map");
    Ok(HANDLE_MAP.add(new_claim)?)
}

fn _claim_create(source_id: Option<String>) -> Claim {

    let mut new_claim: Claim = Default::default();

    new_claim.state = VcxStateType::VcxStateInitialized;
    if let Some(s) = source_id {
        new_claim.set_source_id(s);
    }

    new_claim
}


pub fn update_state(handle: u32) -> Result<u32, u32> {
    HANDLE_MAP.get_mut(handle, |obj|{
        obj.update_state();
        Ok(error::SUCCESS.code_num)
    })

}

pub fn get_state(handle: u32) -> Result<u32, u32> {
    HANDLE_MAP.get(handle, |obj| {
        Ok(obj.get_state())
    }).map_err(handle_err)
}

pub fn release(handle: u32) -> Result<(), u32> {
    HANDLE_MAP.release(handle).map_err(handle_err)
}

pub fn is_valid_handle(handle: u32) -> bool {
    HANDLE_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> Result<String, u32> {
    HANDLE_MAP.get(handle, |obj|{
        serde_json::to_string(&obj).map_err(|e|{
            warn!("Unable to serialize: {:?}", e);
            error::SERIALIZATION_ERROR.code_num
        })
    })
}

pub fn from_string(claim_data: &str) -> Result<u32, u32> {
    let claim: Claim = match serde_json::from_str(claim_data) {
        Ok(x) => x,
        Err(y) => return Err(error::INVALID_JSON.code_num),
    };

    let new_handle = HANDLE_MAP.add(claim)?;

    info!("inserting handle {} into proof table", new_handle);

    Ok(new_handle)
}

//
//pub fn send_claim_offer(handle: u32, connection_handle: u32) -> Result<u32,u32> {
//    match ISSUER_CLAIM_MAP.lock().unwrap().get_mut(&handle) {
//        Some(c) => Ok(c.send_claim_offer(connection_handle)?),
//        None => Err(error::INVALID_ISSUER_CLAIM_HANDLE.code_num),
//    }
//}
//
//pub fn send_claim(handle: u32, connection_handle: u32) -> Result<u32,u32> {
//    match ISSUER_CLAIM_MAP.lock().unwrap().get_mut(&handle) {
//        Some(c) => Ok(c.send_claim(connection_handle)?),
//        None => Err(error::INVALID_ISSUER_CLAIM_HANDLE.code_num),
//    }
//}
//
//fn get_offer_details(response: &str) -> Result<String,u32> {
//    match serde_json::from_str(response) {
//        Ok(json) => {
//            let json: serde_json::Value = json;
//            let detail = match json["uid"].as_str() {
//                Some(x) => x,
//                None => {
//                    info!("response had no uid");
//                    return Err(error::INVALID_JSON.code_num)
//                },
//            };
//            Ok(String::from(detail))
//        },
//        Err(_) => {
//            info!("get_messages called without a valid response from server");
//            Err(error::INVALID_JSON.code_num)
//        },
//    }
//}
//
//pub fn set_claim_request(handle: u32, claim_request: ClaimRequest) -> Result<u32,u32>{
//    match ISSUER_CLAIM_MAP.lock().unwrap().get_mut(&handle) {
//        Some(c) => {c.set_claim_request(claim_request);
//            Ok(error::SUCCESS.code_num)},
//        None => Err(error::INVALID_ISSUER_CLAIM_HANDLE.code_num),
//    }
//}
//
//pub fn append_value(original_payload: &str,key: &str,  value: &str) -> Result<String, u32> {
//    use serde_json::Value;
//    let mut payload_json: Value = match serde_json::from_str(original_payload) {
//        Ok(s) => s,
//        Err(_) => return Err(error::INVALID_JSON.code_num),
//    };
//    payload_json[key] = json!(&value);
//    match serde_json::to_string(&payload_json) {
//        Ok(s) => Ok(s),
//        Err(_) => Err(error::INVALID_JSON.code_num),
//    }
//}
//
//pub fn convert_to_map(s:&str) -> Result<serde_json::Map<String, serde_json::Value>, u32>{
//    let v:serde_json::Map<String, serde_json::Value> = match serde_json::from_str(s) {
//        Ok(m) => m,
//        Err(_) => { warn!("{}", error::INVALID_ATTRIBUTES_STRUCTURE.message);
//            return Err(error::INVALID_ATTRIBUTES_STRUCTURE.code_num)},
//    };
//    Ok(v)
//}


#[cfg(test)]
mod tests {
    extern crate serde_json;

    #[test]
    fn noop(){
    }
}