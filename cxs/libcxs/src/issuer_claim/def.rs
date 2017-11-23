extern crate rand;
extern crate serde_json;
extern crate libc;

use std::sync::Mutex;
use std::collections::HashMap;
use rand::Rng;
use api::CxsStateType;
use utils::error;
use messages;
use settings;
use messages::GeneralMessage;
use connection;
use claim_request::ClaimRequest;
use utils::issuer_claim::CLAIM_REQ_STRING;
use self::libc::c_char;
use utils::callback::CallbackUtils;
use std::sync::mpsc::channel;
use std::ffi::CString;
use utils::timeout::TimeoutUtils;
use utils::wallet;

lazy_static! {
    static ref ISSUER_CLAIM_MAP: Mutex<HashMap<u32, Box<IssuerClaim>>> = Default::default();
}

static DEFAULT_CLAIM_NAME: &str = "Claim";
extern {
    fn indy_issuer_create_and_store_claim_def(command_handle: i32,
                                              wallet_handle: i32,
                                              issuer_did: *const c_char,
                                              schema_json: *const c_char,
                                              signature_type: *const c_char,
                                              create_non_revoc: bool,
                                              cb: Option<extern fn(xcommand_handle: i32, err: i32,
                                                                   claim_def_json: *const c_char)>) -> i32;
    fn indy_issuer_create_claim(command_handle: i32,
                                wallet_handle: i32,
                                claim_req_json: *const c_char,
                                claim_json: *const c_char,
                                user_revoc_index: i32,
                                cb: Option<extern fn(xcommand_handle: i32, err: i32,
                                                     revoc_reg_update_json: *const c_char, //TODO must be OPTIONAL
                                                     xclaim_json: *const c_char
                                )>)-> i32;
}
#[derive(Serialize, Deserialize, Debug)]
pub struct IssuerClaim {
    source_id: String,
    handle: u32,
    claim_attributes: String,
    msg_uid: String,
    schema_seq_no: u32,
    issuer_did: String,
    issued_did: String,
    state: CxsStateType,
    claim_request: Option<ClaimRequest>,
    claim_name: String,
}

impl IssuerClaim {
    fn validate_claim_offer(&self) -> Result<u32, String> {
        //TODO: validate claim_attributes against claim_def
        Ok(error::SUCCESS.code_num)
    }

    fn send_claim_offer(&mut self, connection_handle: u32) -> Result<u32, u32> {
        if self.state != CxsStateType::CxsStateInitialized {
            warn!("claim {} has invalid state {} for sending claimOffer", self.handle, self.state as u32);
            return Err(error::NOT_READY.code_num);
        }

        if connection::is_valid_handle(connection_handle) == false {
            warn!("invalid connection handle ({}) in send_claim_offer", connection_handle);
            return Err(error::INVALID_CONNECTION_HANDLE.code_num);
        }

        //TODO: call to libindy to encrypt payload
        let to_did = connection::get_pw_did(connection_handle).unwrap();
        let from_did = settings::get_config_value(settings::CONFIG_ENTERPRISE_DID_AGENT).unwrap();
        let payload = format!("{{\"msg_type\":\"CLAIM_OFFER\",\"claim_name\":\"{}\",\"version\":\"0.1\",\"to_did\":\"{}\",\"from_did\":\"{}\",\"claim\":{},\"schema_seq_no\":{},\"issuer_did\":\"{}\"}}",self.claim_name,to_did,from_did,self.claim_attributes,self.schema_seq_no,self.issuer_did);

        //Todo: call to message class to build payload
        let added_data = r#""claim_name":"Profile detail","issuer_name":"Test Enterprise","optional_data":{"terms_of_service":"<Large block of text>","price":6}"#;
        match messages::send_message().to(&to_did).msg_type("claimOffer").edge_agent_payload(&payload).send() {
            Err(x) => {
                warn!("could not send claimOffer: {}", x);
                return Err(x);
            },
            Ok(response) => {
                self.msg_uid = match get_offer_details(&response) {
                    Ok(x) => x,
                    Err(x) => return Err(x),
                };
                self.issued_did = to_did;
                self.state = CxsStateType::CxsStateOfferSent;
                return Ok(error::SUCCESS.code_num);
            }
        }
    }
    fn create_send_claim_offer_payload(&self, to_did: &str, from_did: &str ) -> Result<String, u32> {
        #[derive(Serialize, Deserialize)]
        struct ClaimOffer {
            msg_type: String,
            version: String,
            to_did: String,
            from_did: String,
            claim: String,
            schema_seq_no: u32,
            issuer_did: String,
            claim_name: String,
        };

        let claim_offer = ClaimOffer{
            msg_type: String::from("CLAIM_OFFER"),
            version: String::from("0.1"),
            to_did: String::from(to_did),
            from_did:String::from(from_did),
            claim: self.claim_attributes.to_owned(),
            schema_seq_no: self.schema_seq_no.to_owned(),
            issuer_did: String::from(self.issuer_did.to_owned()),
            claim_name: String::from(self.claim_name.to_owned()),
        };
        match serde_json::to_string(&claim_offer)  {
            Ok(s) => { println!("\n\n{}\n\n",s);
                Ok(s)},
            Err(_) => Err(error::INVALID_JSON.code_num),
        }

    }

    fn send_claim(&mut self, connection_handle: u32) -> Result<u32, u32> {
        if self.state != CxsStateType::CxsStateRequestReceived {
            warn!("claim {} has invalid state {} for sending claim", self.handle, self.state as u32);
            return Err(error::NOT_READY.code_num);
        }

        if connection::is_valid_handle(connection_handle) == false {
            warn!("invalid connection handle ({}) in send_claim_offer", connection_handle);
            return Err(error::INVALID_CONNECTION_HANDLE.code_num);
        }

        let attrs_with_encodings = self.create_attributes_encodings()?;
        let data;
        if settings::test_mode_enabled() {
            data = String::from("dummytestmodedata");
        } else {
            data = match self.claim_request.clone() {
                Some(d) => match create_claim_payload_using_wallet(&d, &attrs_with_encodings, wallet::get_wallet_handle()) {
                    Ok(p) => p,
                    Err(e) => return Err(error::UNKNOWN_ERROR.code_num),
                },
                None => return Err(error::INVALID_CLAIM_REQUEST.code_num),
            };
        }
        let to = connection::get_pw_did(connection_handle).unwrap();
        match messages::send_message().to(&to).msg_type("claim").edge_agent_payload(&data).send() {
            Err(x) => {
                warn!("could not send claim: {}", x);
                return Err(x);
            },
            Ok(response) => {
                self.msg_uid = match get_offer_details(&response) {
                    Ok(x) => x,
                    Err(x) => {
                        info!("Error in response: {}", x);
                        return Err(x);
                    },
                };
                self.issued_did = to;
                self.state = CxsStateType::CxsStateAccepted;
                return Ok(error::SUCCESS.code_num);
            }
        }
    }

    fn create_attributes_encodings(&self) -> Result<String, u32> {
        let mut attributes: serde_json::Value = match serde_json::from_str(&self.claim_attributes) {
            Ok(x) => x,
            Err(x) => {
                warn!("Invalid Json for Attribute data");
                return Err(error::INVALID_JSON.code_num)
            }
        };

        let mut map = match attributes.as_object_mut() {
            Some(x) => x,
            None => {
                warn!("Invalid Json for Attribute data");
                return Err(error::INVALID_JSON.code_num)
            }
        };

        for (attr, mut vec) in map.iter_mut(){
            let mut list = match vec.as_array_mut() {
                Some(x) => x,
                None => {
                    warn!("Invalid Json for Attribute data");
                    return Err(error::INVALID_JSON.code_num)
                }
            };
//          FIXME This is hardcode but should have logic for finding strings and integers and
//          doing a real encoding (sha256)
            let encoded = serde_json::Value::from("1139481716457488690172217916278103335");
            list.push(encoded)
        }

        match serde_json::to_string_pretty(&map) {
            Ok(x) => Ok(x),
            Err(x) => {
                warn!("Invalid Json for Attribute data");
                Err(error::INVALID_JSON.code_num)
            }
        }
    }

    fn get_claim_req(&mut self, msg_uid: &str) {
        info!("Checking for outstanding claimReq for {} with uid: {}", self.handle, msg_uid);
        let response = match messages::get_messages().to(&self.issued_did).uid(msg_uid).send() {
            Ok(x) => x,
            Err(x) => {
                warn!("invalid response to get_messages for claim {}", self.handle);
                return
            },
        };

        let json: serde_json::Value = match serde_json::from_str(&response) {
            Ok(json) => json,
            Err(_) => {
                warn!("invalid json in get_messages for claim {}", self.handle);
                return
            },
        };

        let msgs = match json["msgs"].as_array() {
            Some(array) => array,
            None => {
                warn!("invalid msgs array returned for claim {}", self.handle);
                return
            },
        };

        for msg in msgs {
            if msg["typ"] == String::from("claimReq") {
                //get the followup-claim-req using refMsgId
                self.state = CxsStateType::CxsStateRequestReceived;

                let string_payload = match msg["edgeAgentPayload"].as_str() {
                    Some(x) => x,
                    None => {
                        warn!("claim request has no edge agent payload");
                        return
                    }
                };

                let payload: serde_json::Value = match serde_json::from_str(string_payload) {
                    Ok(x) => x,
                    Err(x) => {
                        warn!("invalid json for claim requests edgeAgentPayload");
                        return
                    },
                };

                self.claim_request = match ClaimRequest::create_from_api_msg_json(&payload) {
                    Ok(x) => Some(x),
                    Err(_) => {
                        warn!("invalid claim request for claim {}", self.handle);
                        return
                    }
                };
                return
            }
        }
    }

    fn get_claim_offer_status(&mut self) {
        if self.state == CxsStateType::CxsStateRequestReceived {
            return;
        }
        else if self.state != CxsStateType::CxsStateOfferSent || self.msg_uid.is_empty() || self.issued_did.is_empty() {
            return;
        }
        // state is "OfferSent" so check to see if there is a new claimReq
        let response = match messages::get_messages().to(&self.issued_did).uid(&self.msg_uid).send() {
            Ok(x) => x,
            Err(x) => {
                warn!("invalid response to get_messages for claim {}", self.handle);
                return
            },
        };
        let json: serde_json::Value = match serde_json::from_str(&response) {
            Ok(json) => json,
            Err(_) => {
                warn!("invalid json in get_messages for claim {}", self.handle);
                return
            },
        };

        let msgs = match json["msgs"].as_array() {
            Some(array) => array,
            None => {
                warn!("invalid msgs array returned for claim {}", self.handle);
                return
            },
        };

        for msg in msgs {
            if msg["statusCode"].to_string() == "\"MS-104\"" {
                //get the followup-claim-req using refMsgId
                let ref_msg_id = match msg["refMsgId"].as_str() {
                    Some(x) => x,
                    None => {
                        warn!("invalid message reference id for claim {}", self.handle);
                        return
                    }
                };
                self.get_claim_req(ref_msg_id);
            }
        }
    }

    fn update_state(&mut self) {
        self.get_claim_offer_status();
        //There will probably be more things here once we do other things with the claim
    }

    fn get_state(&self) -> u32 { let state = self.state as u32; state }
    fn get_offer_uid(&self) -> String { self.msg_uid.clone() }
    fn set_offer_uid(&mut self, uid: &str) {self.msg_uid = uid.to_owned();}
    fn set_claim_request(&mut self, claim_request:&ClaimRequest){
        self.claim_request = Some(claim_request.clone());
    }
    pub fn create_standard_issuer_claim() -> Result<IssuerClaim, u32> {
        let claim_req_value = &serde_json::from_str(CLAIM_REQ_STRING).unwrap();
        let issuer_claim = IssuerClaim {
            handle: 123,
            source_id: "standard_claim".to_owned(),
            schema_seq_no: 32,
            msg_uid: "1234".to_owned(),
            claim_attributes: "nothing".to_owned(),
            issuer_did: "QTrbV4raAcND4DWWzBmdsh".to_owned(),
            issued_did: "8XFh8yBzrpJQmNyZzgoTqB".to_owned(),
            state: CxsStateType::CxsStateOfferSent,
            claim_request: match ClaimRequest::create_from_api_msg_json(claim_req_value) {
                Ok(x) => Some(x.clone()),
                Err(_) => {
                    warn!("invalid claim request for claim {}", 123);
                    return Err(error::INVALID_CLAIM_REQUEST.code_num)
                }
            },
            claim_name: "Claim".to_owned(),
        };
        Ok(issuer_claim)
    }
}

pub fn create_claim_payload_using_wallet<'a>(claim_req: &ClaimRequest, claim_data: &str, wallet_handle: i32) -> Result< String, u32> {
    println!("claim data: {}", claim_data);
    println!("claim request: {:?}", serde_json::to_string(&claim_req));
    let (sender, receiver) = channel();

    let cb = Box::new(move |err, revoc_reg_update_json, xclaim_json| {
        sender.send((err, revoc_reg_update_json, xclaim_json)).unwrap();
    });
    info!("wallet_handle: {}", wallet_handle);
    let (command_handle, cb) = CallbackUtils::closure_to_issuer_create_claim_cb(cb);

    let claim_req_master_secret = match claim_req.blinded_ms.clone() {
        Some(ms) => ms,
        // TODO: need new error
        None => {
            error!("No Master Secret in the Claim Request!");
            return Err(error::UNKNOWN_ERROR.code_num);
        },
    };

    let claim_req_str = match serde_json::to_string(&claim_req) {
        Ok(s) => s,
        // TODO: need new error
        Err(x) => {
            error!("Claim Request is not properly formatted/formed: {}", x);
            return Err(error::UNKNOWN_ERROR.code_num);
        },
    };

    unsafe {
        let err = indy_issuer_create_claim(command_handle,
                                           wallet_handle,
                                           CString::new(claim_req_str).unwrap().as_ptr(),
                                           CString::new(claim_data).unwrap().as_ptr(),
                                           -1,
                                           cb);
        if err != 0 {
            error!("could not create claim: {}", err);
            return Err(error::UNKNOWN_ERROR.code_num);
        }
    }

    let (err, revoc_reg_update_json, xclaim_json) = receiver.recv_timeout(TimeoutUtils::long_timeout()).unwrap();

    if err != 0 {
        error!("could not create claim: {}", err);
        return Err(error::UNKNOWN_ERROR.code_num);
    };

    info!("xclaim_json: {}", xclaim_json);
    Ok(xclaim_json)
}

pub fn get_offer_uid(handle: u32) -> Result<String,u32> {
    match ISSUER_CLAIM_MAP.lock().unwrap().get(&handle) {
        Some(claim) => Ok(claim.get_offer_uid()),
        None => Err(error::INVALID_ISSUER_CLAIM_HANDLE.code_num),
    }
}

pub fn issuer_claim_create(schema_seq_no: u32,
                           source_id: Option<String>,
                           issuer_did: String,
                           claim_data: String) -> Result<u32, String> {

    let new_handle = rand::thread_rng().gen::<u32>();

    let source_id_unwrap = source_id.unwrap_or("".to_string());

    let mut new_issuer_claim = Box::new(IssuerClaim {
        handle: new_handle,
        source_id: source_id_unwrap,
        msg_uid: String::new(),
        claim_attributes: claim_data,
        issued_did: String::new(),
        issuer_did,
        state: CxsStateType::CxsStateNone,
        schema_seq_no,
        claim_request: None,
        claim_name: String::from("Claim"),
    });

    match new_issuer_claim.validate_claim_offer() {
        Ok(_) => info!("successfully validated issuer_claim {}", new_handle),
        Err(x) => return Err(x),
    };

    new_issuer_claim.state = CxsStateType::CxsStateInitialized;

    info!("inserting handle {} into claim_issuer table", new_handle);
    ISSUER_CLAIM_MAP.lock().unwrap().insert(new_handle, new_issuer_claim);;

    Ok(new_handle)
}

pub fn update_state(handle: u32) {
    match ISSUER_CLAIM_MAP.lock().unwrap().get_mut(&handle) {
        Some(t) => t.update_state(),
        None => {}
    };
}

pub fn get_state(handle: u32) -> u32 {
    match ISSUER_CLAIM_MAP.lock().unwrap().get(&handle) {
        Some(t) => t.get_state(),
        None => CxsStateType::CxsStateNone as u32,
    }
}

pub fn release(handle: u32) -> u32 {
    match ISSUER_CLAIM_MAP.lock().unwrap().remove(&handle) {
        Some(t) => error::SUCCESS.code_num,
        None => error::INVALID_ISSUER_CLAIM_HANDLE.code_num,
    }
}

pub fn is_valid_handle(handle: u32) -> bool {
    match ISSUER_CLAIM_MAP.lock().unwrap().get(&handle) {
        Some(_) => true,
        None => false,
    }
}

pub fn to_string(handle: u32) -> Result<String,u32> {
    match ISSUER_CLAIM_MAP.lock().unwrap().get(&handle) {
        Some(c) => Ok(serde_json::to_string(&c).unwrap().to_owned()),
        None => Err(error::INVALID_ISSUER_CLAIM_HANDLE.code_num),
    }
}

pub fn from_string(claim_data: &str) -> Result<u32,u32> {
    let derived_claim: IssuerClaim = match serde_json::from_str(claim_data) {
        Ok(x) => x,
        Err(_) => return Err(error::INVALID_JSON.code_num),
    };

    let new_handle = derived_claim.handle;

    if is_valid_handle(new_handle) {return Ok(new_handle);}
    let claim = Box::from(derived_claim);

    {
        let mut m = ISSUER_CLAIM_MAP.lock().unwrap();
        info!("inserting handle {} into claim_issuer table", new_handle);
        m.insert(new_handle, claim);
    }

    Ok(new_handle)
}

pub fn send_claim_offer(handle: u32, connection_handle: u32) -> Result<u32,u32> {
    match ISSUER_CLAIM_MAP.lock().unwrap().get_mut(&handle) {
        Some(c) => match c.send_claim_offer(connection_handle) {
            Ok(_) => Ok(error::SUCCESS.code_num),
            Err(x) => Err(x),
        },
        None => Err(error::INVALID_ISSUER_CLAIM_HANDLE.code_num),
    }
}

pub fn send_claim(handle: u32, connection_handle: u32) -> Result<u32,u32> {
    match ISSUER_CLAIM_MAP.lock().unwrap().get_mut(&handle) {
        Some(c) => match c.send_claim(connection_handle) {
            Ok(_) => Ok(error::SUCCESS.code_num),
            Err(x) => Err(x),
        },
        None => Err(error::INVALID_ISSUER_CLAIM_HANDLE.code_num),
    }
}

fn get_offer_details(response: &str) -> Result<String,u32> {
    if settings::test_mode_enabled() {return Ok("test_mode_response".to_owned());}
    match serde_json::from_str(response) {
        Ok(json) => {
            let json: serde_json::Value = json;
            let detail = match json["uid"].as_str() {
                Some(x) => x,
                None => {
                    info!("response had no uid");
                    return Err(error::INVALID_JSON.code_num)
                },
            };
            Ok(String::from(detail))
        },
        Err(_) => {
            info!("Connect called without a valid response from server");
            Err(error::UNKNOWN_ERROR.code_num)
        },
    }
}

pub fn set_claim_request(handle: u32, claim_request: &ClaimRequest) -> Result<u32,u32>{
   match ISSUER_CLAIM_MAP.lock().unwrap().get_mut(&handle) {
       Some(c) => {c.set_claim_request(claim_request);
                    Ok(error::SUCCESS.code_num)},
       None => Err(error::UNKNOWN_ERROR.code_num),
   }
}

#[cfg(test)]
#[path="./test/def_tests.rs"]
mod def_tests;

