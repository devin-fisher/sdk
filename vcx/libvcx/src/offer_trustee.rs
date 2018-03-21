extern crate rand;
extern crate serde_json;
extern crate serde;
extern crate libc;

use std::sync::Mutex;
use std::collections::HashMap;
use rand::Rng;
use api::VcxStateType;
use utils::error;
use messages;
use messages::GeneralMessage;
use messages::MessageResponseCode::{ MessageAccepted };
use messages::send_message::parse_msg_uid;
use messages::trustee::{MsgVersion, TrusteeMsgType, TrusteeCapability};
use messages::trustee::offer::TrusteeOffer;
use messages::trustee::request::TrusteeRequest;
use messages::trustee::data::{TrusteeData, RecoveryShare, RecoveryShareHint};
use connection;
use serde_json::Value;
use settings;
use recovery_shares;
use utils::option_util::expect_ok_or;

use utils::httpclient;
use utils::constants::SEND_MESSAGE_RESPONSE;


lazy_static! {
    static ref HANDLE_MAP: Mutex<HashMap<u32, Box<OfferTrustee>>> = Default::default();
}


#[derive(Serialize, Deserialize, Debug)]
pub struct OfferTrustee {
    source_id: String,
    handle: u32,
    msg_uid: String,
    state: VcxStateType,
    ref_msg_id: Option<String>,
    trustee_req: Option<TrusteeRequest>,
    // the following 6 are pulled from the connection object
    agent_did: String, //agent_did for this relationship
    agent_vk: String,
    issued_did: String, //my_pw_did for this relationship
    issued_vk: String,
    remote_did: String, //their_pw_did for this relationship
    remote_vk: String,
}



impl OfferTrustee {
    fn send_trustee_offer(&mut self, connection_handle: u32) -> Result<u32, u32> {
        info!("sending trustee offer for offer_trustee handle {} to connection handle {}", self.handle, connection_handle);
        if self.state != VcxStateType::VcxStateInitialized {
            warn!("offer {} has invalid state {} for sending trustee offer", self.handle, self.state as u32);
            return Err(error::NOT_READY.code_num);
        }

        if connection::is_valid_handle(connection_handle) == false {
            warn!("invalid connection handle ({})", connection_handle);
            return Err(error::INVALID_CONNECTION_HANDLE.code_num);
        }

        self.agent_did = connection::get_agent_did(connection_handle)?;
        self.agent_vk = connection::get_agent_verkey(connection_handle)?;
        self.issued_did = connection::get_pw_did(connection_handle)?;
        self.issued_vk = connection::get_pw_verkey(connection_handle)?;
        self.remote_vk = connection::get_their_pw_verkey(connection_handle)?;

        let payload = match settings::test_indy_mode_enabled() {
            false => {
                let offer = self._generate_trustee_offer()?;
                match serde_json::to_string(&offer) {
                    Ok(p) => p,
                    Err(_) => return Err(error::INVALID_JSON.code_num)
                }
            },
            true => String::from("dummytestmodedata")
        };

        debug!("trustee offer data: {}", payload);

        let data = connection::generate_encrypted_payload(&self.issued_vk, &self.remote_vk, &payload, "TRUSTEE_OFFER")?;

        if settings::test_agency_mode_enabled() { httpclient::set_next_u8_response(SEND_MESSAGE_RESPONSE.to_vec());}

        match messages::send_message().to(&self.issued_did)
            .to_vk(&self.issued_vk)
            .msg_type("trusteeOffer")
            .edge_agent_payload(&data)
            .agent_did(&self.agent_did)
            .agent_vk(&self.agent_vk)
            .status_code(&MessageAccepted.as_string())
            .send_secure() {
            Err(x) => {
                warn!("could not send trusteeOffer: {}", x);
                return Err(x);
            },
            Ok(response) => {
                self.msg_uid = parse_msg_uid(&response[0])?;
                self.state = VcxStateType::VcxStateSent;
                info!("sent trustee offer for: {}", self.handle);
                return Ok(error::SUCCESS.code_num);
            }
        }

    }

    fn send_trustee_data(&mut self, recovery_shares_handle: u32,  connection_handle: u32) -> Result<u32, u32> {
        info!("sending trustee data for handle {} to connection handle {}", self.handle, connection_handle);
        if self.state != VcxStateType::VcxStateRequestReceived {
            warn!("offer_trustee {} has invalid state {} for sending trustee data", self.handle, self.state as u32);
            return Err(error::NOT_READY.code_num);
        }

        if connection::is_valid_handle(connection_handle) == false {
            warn!("invalid connection handle ({}) in send_trustee_data", connection_handle);
            return Err(error::INVALID_CONNECTION_HANDLE.code_num);
        }

        let to = connection::get_pw_did(connection_handle)?;



        let data = match settings::test_indy_mode_enabled() {
            false => {
                self._add_key_to_policy()?;
                let data = self._generate_trustee_data(recovery_shares_handle)?;
                serde_json::to_string(&data).or(Err(error::INVALID_JSON.code_num))?
            },
            true => String::from("dummytestmodedata")
        };

        debug!("trustee data: {:?}", data);
        let data = connection::generate_encrypted_payload(&self.issued_vk, &self.remote_vk, &data, "TRUSTEE_DATA")?;

        if settings::test_agency_mode_enabled() { httpclient::set_next_u8_response(SEND_MESSAGE_RESPONSE.to_vec()); }

        match messages::send_message().to(&self.issued_did)
            .to_vk(&self.issued_vk)
//            .ref_msg_id(self.ref_msg_id.as_ref())
            .msg_type("trusteeData")
            .status_code((&MessageAccepted.as_string()))
            .edge_agent_payload(&data)
            .agent_did(&self.agent_did)
            .agent_vk(&self.agent_vk)
            .send_secure() {
            Err(x) => {
                warn!("could not send trustee data: {}", x);
                return Err(x);
            },
            Ok(response) => {
                self.msg_uid = parse_msg_uid(&response[0])?;
                self.state = VcxStateType::VcxStateAccepted;
                info!("trustee data sent: {}", self.handle);
                return Ok(error::SUCCESS.code_num);
            }
        }
    }


    fn get_trustee_offer_status(&mut self) -> Result<u32, u32> {
        info!("updating state for trustee offer: {}", self.handle);
        if self.state == VcxStateType::VcxStateRequestReceived {
            return Ok(error::SUCCESS.code_num);
        }
        else if self.state != VcxStateType::VcxStateSent || self.msg_uid.is_empty() || self.issued_did.is_empty() {

            return Ok(error::SUCCESS.code_num);
        }
        let payload = messages::get_message::get_ref_msg(&self.msg_uid, &self.issued_did, &self.issued_vk, &self.agent_did, &self.agent_vk)?;

        self.trustee_req = Some(parse_trustee_req_payload(&payload)?);
        info!("received trustee request for trustee offer: {}", self.handle);
        self.state = VcxStateType::VcxStateRequestReceived;
        Ok(error::SUCCESS.code_num)
    }

    fn update_state(&mut self) {
        self.get_trustee_offer_status().unwrap_or(error::SUCCESS.code_num);

    }

    fn get_state(&self) -> u32 { let state = self.state as u32; state }
    fn get_offer_uid(&self) -> String { self.msg_uid.clone() }
    fn set_offer_uid(&mut self, uid: &str) {self.msg_uid = uid.to_owned();}

    fn _generate_trustee_offer(&self) -> Result<Value, u32> {
        let rtn = TrusteeOffer{
            version: MsgVersion::v0_1,
            msg_type: TrusteeMsgType::TrusteeOffer,
            capabilities: vec![TrusteeCapability::RecoveryShare, TrusteeCapability::RevokeAuthz],
            expires: None,
            msg_uid: None,
        };
        Ok(serde_json::to_value(&rtn).or(Err(error::INVALID_JSON.code_num))?)
    }

    fn _add_key_to_policy(&self) -> Result<(), u32> {
        let address = settings::get_config_value(settings::CONFIG_IDENTITY_POLICY_ADDRESS)?;
        let req = expect_ok_or(self.trustee_req.as_ref(),
                               "must have trustee req",
                               10 as u32)?;

        for key in &req.authorization_keys {

            //TODO add key to policy
        }

        Ok(())
    }

    fn _generate_trustee_data(&self, recovery_shares_handle: u32) -> Result<TrusteeData, u32> {
        let address = settings::get_config_value(settings::CONFIG_IDENTITY_POLICY_ADDRESS)?;
        if address.is_empty() {
            warn!("Identity Address is blank");
            return Err(error::INVALID_OPTION.code_num);
        }

        if !recovery_shares::is_valid_handle(recovery_shares_handle) {
            warn!("Recovery_shares_handle is not valid");
            return Err(error::INVALID_OPTION.code_num);
        }
        let share_val = recovery_shares::consume_share(recovery_shares_handle)?;

        let rtn = TrusteeData {
            version: MsgVersion::v0_1,
            msg_type: TrusteeMsgType::TrusteeData,
            address,
            share: RecoveryShare{
                version: MsgVersion::v0_1,
                source_did: String::new(),
                tag: String::new(),
                value: share_val,
                hint: Some(RecoveryShareHint{
                    theshold: None,
                    trustees: None,
                }),
            },
        };
        Ok(rtn)
    }
}



pub fn get_offer_uid(handle: u32) -> Result<String,u32> {
    match HANDLE_MAP.lock().unwrap().get(&handle) {
        Some(obj) => Ok(obj.get_offer_uid()),
        None => Err(error::INVALID_OBJ_HANDLE.code_num),
    }
}

fn parse_trustee_req_payload(payload: &Vec<u8>) -> Result<TrusteeRequest, u32> {
    debug!("parsing trusteeReq payload: {:?}", payload);
    let data = messages::extract_json_payload(payload)?;

    let trustee_req = match serde_json::from_str(&data) {
         Ok(x) => x,
         Err(x) => {
             warn!("invalid json {}", x);
             return Err(error::INVALID_JSON.code_num);
         },
    };

    Ok(trustee_req)
}

pub fn offer_trustee_create(source_id: Option<String>) -> Result<u32, u32> {

    let new_handle = rand::thread_rng().gen::<u32>();

    let source_id_unwrap = source_id.unwrap_or("".to_string());

    let mut new_obj = Box::new(OfferTrustee {
        handle: new_handle,
        source_id: source_id_unwrap,
        msg_uid: String::new(),
        state: VcxStateType::VcxStateNone,
        ref_msg_id: None,
        issued_did: String::new(),
        issued_vk: String::new(),
        remote_did: String::new(),
        remote_vk: String::new(),
        agent_did: String::new(),
        agent_vk: String::new(),
        trustee_req: None,
    });

    new_obj.state = VcxStateType::VcxStateInitialized;

    info!("inserting handle {} into offer trustee table", new_handle);
    HANDLE_MAP.lock().unwrap().insert(new_handle, new_obj);

    Ok(new_handle)
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

pub fn is_valid_handle(handle: u32) -> bool {
    match HANDLE_MAP.lock().unwrap().get(&handle) {
        Some(_) => true,
        None => false,
    }
}

pub fn to_string(handle: u32) -> Result<String,u32> {
    match HANDLE_MAP.lock().unwrap().get(&handle) {
        Some(c) => Ok(serde_json::to_string(&c).unwrap().to_owned()),
        None => Err(error::INVALID_OBJ_HANDLE.code_num),
    }
}

pub fn from_string(data: &str) -> Result<u32,u32> {
    let derived: OfferTrustee = match serde_json::from_str(data) {
        Ok(x) => x,
        Err(_) => return Err(error::INVALID_JSON.code_num),
    };

    let new_handle = derived.handle;

    if is_valid_handle(new_handle) {return Ok(new_handle);}
    let obj = Box::from(derived);

    {
        let mut m = HANDLE_MAP.lock().unwrap();
        info!("inserting handle {} into offer trustee table", new_handle);
        m.insert(new_handle, obj);
    }

    Ok(new_handle)
}

pub fn send_trustee_offer(handle: u32, connection_handle: u32) -> Result<u32,u32> {
    match HANDLE_MAP.lock().unwrap().get_mut(&handle) {
        Some(c) => Ok(c.send_trustee_offer(connection_handle)?),
        None => Err(error::INVALID_OBJ_HANDLE.code_num),
    }
}

pub fn send_trustee_data(handle: u32, recovery_shares: u32, connection_handle: u32) -> Result<u32,u32> {
    match HANDLE_MAP.lock().unwrap().get_mut(&handle) {
        Some(c) => Ok(c.send_trustee_data(recovery_shares, connection_handle)?),
        None => Err(error::INVALID_OBJ_HANDLE.code_num),
    }
}





#[cfg(test)]
pub mod tests {
    use super::*;
    use settings;
    use utils::httpclient;
    use utils::dkms_constants::*;

    #[test]
    fn full_trustee_offer_test() {
        ::utils::logger::LoggerUtils::init();
        settings::set_defaults();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");

        settings::set_config_value(settings::CONFIG_IDENTITY_POLICY_ADDRESS, "GoHaRgzghcZ3GQ2VimymkoWSW7q9un4KR7rz3tJ24Gqr");

        let connection_h = connection::build_connection("test_send_claim_offer".to_owned()).unwrap();

        let trustee_h = offer_trustee_create(Some("Bob_trustee".to_owned())).unwrap();

        assert_eq!(1, get_state(trustee_h));

        send_trustee_offer(trustee_h, connection_h).unwrap();

        assert_eq!(2, get_state(trustee_h));

        httpclient::set_next_u8_response(TRUSTEE_REQUEST_RESPONSE.to_vec());
        httpclient::set_next_u8_response(UPDATE_TRUSTEE_OFFER_RESPONSE.to_vec());

        update_state(trustee_h);

        assert_eq!(3, get_state(trustee_h));

        let recovery_h = recovery_shares::create(Some("recovery".to_owned()), 10, 2).unwrap();

        send_trustee_data(trustee_h, recovery_h, connection_h).unwrap();

        assert_eq!(4, get_state(trustee_h));

    }


}
