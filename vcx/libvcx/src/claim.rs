extern crate rand;
extern crate serde_json;
extern crate libc;
extern crate serde;
extern crate rmp_serde;

use object_cache::ObjectCache;
use api::VcxStateType;
use utils::error;
use issuer_claim::ClaimOffer;

use claim_request::ClaimRequest;

use messages;
use messages::to_u8;
use messages::GeneralMessage;
use messages::send_message::parse_msg_uid;
use messages::extract_json_payload;

use utils::libindy::anoncreds::{libindy_prover_create_and_store_claim_req, libindy_prover_store_claim};
use utils::libindy::SigTypes;
use utils::libindy::wallet;
use utils::libindy::crypto;

use utils::option_util::expect_ok_or;

use claim_def::{ RetrieveClaimDef, ClaimDefCommon };
use connection;


use serde_json::Value;

lazy_static! {
    static ref HANDLE_MAP: ObjectCache<Claim>  = Default::default();
}

const LINK_SECRET_ALIAS: &str = "main";

impl Default for Claim {
    fn default() -> Claim
    {
        Claim {
            source_id: String::new(),
            state: VcxStateType::VcxStateNone,
            claim_name: None,
            claim_request: None,
            agent_did: None,
            agent_vk: None,
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
            claim_offer: None,
            link_secret_alias: Some(String::from("main")), //TODO this should not be hardcoded
            msg_uid: None,
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Claim {
    source_id: String,
    state: VcxStateType,
    claim_name: Option<String>,
    claim_request: Option<ClaimRequest>,
    claim_offer: Option<ClaimOffer>,
    link_secret_alias: Option<String>,
    msg_uid: Option<String>,
    // the following 6 are pulled from the connection object
    agent_did: Option<String>,
    agent_vk: Option<String>,
    my_did: Option<String>,
    my_vk: Option<String>,
    their_did: Option<String>,
    their_vk: Option<String>,
}

impl Claim {

    fn _find_claim_def(&self, issuer_did: &str, schema_seq_num: u32) -> Result<String, u32> {
        RetrieveClaimDef::new()
            .retrieve_claim_def("GGBDg1j8bsKmr4h5T9XqYf",
                                schema_seq_num,
                                Some(SigTypes::CL),
                                issuer_did)
    }

    fn _build_request(&self, my_did: &str, their_did: &str) -> Result<ClaimRequest, u32> {


        let wallet_h = wallet::get_wallet_handle();



        let prover_did = expect_ok_or(self.my_did.as_ref(),
                                      "",
                                      10 as u32)?;
        let claim_offer = expect_ok_or(self.claim_offer.as_ref(),
                                       "",
                                       10 as u32)?;

        let claim_def = self._find_claim_def(&claim_offer.issuer_did,
                                             claim_offer.schema_seq_no)?;

        let master_secret = LINK_SECRET_ALIAS;

        let claim_offer = serde_json::to_string(claim_offer).or(Err(10 as u32))?;


        let req = libindy_prover_create_and_store_claim_req(wallet_h,
                                                            &prover_did,
                                                            &claim_offer,
                                                            &claim_def,
                                                            master_secret)?;

        let mut  req : Value = serde_json::from_str(&req)
            .or_else(|e|{
                error!("Unable to create claim request - libindy error: {}", e);
                Err(error::UNKNOWN_LIBINDY_ERROR.code_num)
            })?;

        if let Value::Object(ref mut map) = req {
            map.insert(String::from("version"), Value::from("0.1"));
            map.insert(String::from("tid"), Value::from(""));
            map.insert(String::from("to_did"), Value::from(their_did));
            map.insert(String::from("from_did"), Value::from(my_did));
            map.insert(String::from("mid"), Value::from(""));
        }
        else {
            warn!("Unable to create claim request -- invalid json from libindy");
            return Err(error::UNKNOWN_LIBINDY_ERROR.code_num);
        }
        Ok(serde_json::from_value(req).or(Err(error::INVALID_JSON.code_num))?)
    }

    fn send_request(&mut self, connection_handle: u32) -> Result<u32, u32> {
        info!("sending claim offer via connection connection: {}", connection_handle);
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

        let e_code: u32 = 10; //TODO proper error handling

        let local_their_did = self.their_did.as_ref().ok_or(e_code)?;
        let local_their_vk = self.their_vk.as_ref().ok_or(e_code)?;
        let local_agent_did = self.agent_did.as_ref().ok_or(e_code)?;
        let local_agent_vk = self.agent_vk.as_ref().ok_or(e_code)?;
        let local_my_did = self.my_did.as_ref().ok_or(e_code)?;
        let local_my_vk = self.my_vk.as_ref().ok_or(e_code)?;


        let req: ClaimRequest = self._build_request(local_my_did, local_their_did)?;
        let req = serde_json::to_string(&req).or(Err(10 as u32))?;
        let data: Vec<u8> = connection::generate_encrypted_payload(local_my_vk, local_their_vk, &req, "CLAIM_REQ")?;
        let offer_msg_id = self.claim_offer.as_ref().unwrap().msg_ref_id.as_ref().ok_or(e_code)?;
////        if settings::test_agency_mode_enabled() { httpclient::set_next_u8_response(SEND_CLAIM_OFFER_RESPONSE.to_vec()); }

        match messages::send_message().to(local_my_did)
            .to_vk(local_my_vk)
            .msg_type("claimReq")
            .agent_did(local_agent_did)
            .agent_vk(local_agent_vk)
            .edge_agent_payload(&data)
            .ref_msg_id(offer_msg_id)
            .send_secure() {
            Ok(response) => {
                self.msg_uid = Some(parse_msg_uid(&response[0])?);
                self.state = VcxStateType::VcxStateOfferSent;
                return Ok(error::SUCCESS.code_num)
            },
            Err(x) => {
                warn!("could not send proof: {}", x);
                return Err(x);
            }
        }
    }

    fn _check_msg(&mut self) -> Result<(), u32> {
        let e_code: u32 = 10; //TODO proper error handling

        let agent_did = self.agent_did.as_ref().ok_or(e_code)?;
        let agent_vk = self.agent_vk.as_ref().ok_or(e_code)?;
        let my_did = self.my_did.as_ref().ok_or(e_code)?;
        let my_vk = self.my_vk.as_ref().ok_or(e_code)?;
        let msg_uid = self.msg_uid.as_ref().ok_or(e_code)?;

        let payload = messages::get_message::get_all_message(my_did,
                                                         my_vk,
                                                         agent_did,
                                                         agent_vk)?;


        for msg in payload {
            if msg.msg_type.eq("claim") {
                match msg.payload {
                    Some(ref data) => {
                        let data = to_u8(data);
                        let data = crypto::parse_msg(wallet::get_wallet_handle(), &my_vk, data.as_slice())?;

                        let claim = extract_json_payload(&data)?;
                        let claim: Value = serde_json::from_str(&claim).or(Err(10)).unwrap();

                        let wallet_h = wallet::get_wallet_handle();

                        let claim = serde_json::to_string_pretty(&claim).unwrap();
                        libindy_prover_store_claim(wallet_h, &claim)?;
                        info!("received claim");
                        self.state = VcxStateType::VcxStateAccepted;
                    },
                    None => return Err(10) // TODO better error
                };
            }
        }
        Ok(())
    }

    fn update_state(&mut self) {
        match self.state {
            VcxStateType::VcxStateOfferSent => {
                //Check for messages
                let _ = self._check_msg();
            },
            VcxStateType::VcxStateAccepted => {
                //Check for revocation
            }
            _ => {
                // NOOP there is nothing the check for a changed state
            }
        }
    }

    fn get_state(&self) -> u32 { let state = self.state as u32; state }

    fn set_source_id(&mut self, id: String) {
        self.source_id = id;
    }

    fn set_claim_offer(&mut self, offer: ClaimOffer){
        self.claim_offer = Some(offer);
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
    let mut new_claim = _claim_create(source_id);

    let offer: ClaimOffer = serde_json::from_str(offer).map_err(|_|error::INVALID_JSON.code_num)?;
    new_claim.set_claim_offer(offer);

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

pub fn send_claim_request(handle: u32, connection_handle: u32) -> Result<u32, u32> {
    HANDLE_MAP.get_mut(handle, |obj| {
        obj.send_request(connection_handle)
    }).map_err(handle_err)
}

pub fn new_claims_offer_messages(connection_handle: u32, match_name: Option<&str>) -> Result<String, u32> {
    let my_did = connection::get_pw_did(connection_handle)?;
    let my_vk = connection::get_pw_verkey(connection_handle)?;
    let agent_did = connection::get_agent_did(connection_handle)?;
    let agent_vk = connection::get_agent_verkey(connection_handle)?;

    let payload = messages::get_message::get_all_message(&my_did,
                                                     &my_vk,
                                                     &agent_did,
                                                     &agent_vk)?;

    let mut messages: Vec<ClaimOffer> = Default::default();

    for msg in payload {
        if msg.msg_type.eq("claimOffer") {
            let msg_data = match msg.payload {
                Some(ref data) => {
                    let data = to_u8(data);
                    crypto::parse_msg(wallet::get_wallet_handle(), &my_vk, data.as_slice())?
                },
                None => return Err(10) // TODO better error
            };

            let offer = extract_json_payload(&msg_data)?;

            let mut offer: ClaimOffer = serde_json::from_str(&offer)
                .or(Err(error::INVALID_JSON.code_num))?;

            offer.msg_ref_id = Some(msg.uid.to_owned());
            messages.push(offer);
        }
    }


    Ok(serde_json::to_string_pretty(&messages).unwrap())
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
