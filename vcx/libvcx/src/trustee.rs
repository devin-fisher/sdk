extern crate rand;
extern crate serde_json;
extern crate libc;
extern crate serde;
extern crate rmp_serde;

use object_cache::ObjectCache;
use api::VcxStateType;
use utils::error;

use messages;
use messages::to_u8;
use messages::GeneralMessage;
use messages::send_message::parse_msg_uid;
use messages::extract_json_payload;

use messages::trustee::offer::TrusteeOffer;
use messages::trustee::request::TrusteeRequest;
use messages::trustee::data::{TrusteeData, RecoveryShare};
use messages::trustee::{MsgVersion, TrusteeMsgType};

use utils::libindy::wallet;
use utils::libindy::crypto;

use settings;
use utils::option_util::expect_ok_or;
use connection;
use serde_json::Value;

use utils::httpclient;
use utils::constants::SEND_MESSAGE_RESPONSE;

lazy_static! {
    static ref HANDLE_MAP: ObjectCache<Trustee>  = Default::default();
}

const LINK_SECRET_ALIAS: &str = "main";

impl Default for Trustee {
    fn default() -> Trustee
    {
        Trustee {
            source_id: String::new(),
            state: VcxStateType::VcxStateNone,
            agent_did: None,
            agent_vk: None,
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
            msg_uid: None,
            trustee_offer: None,
            trustee_data: None,
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Trustee {
    source_id: String,
    state: VcxStateType,
    msg_uid: Option<String>,
    trustee_offer: Option<TrusteeOffer>,
    trustee_data: Option<TrusteeData>,
    // the following 6 are pulled from the connection object
    agent_did: Option<String>,
    agent_vk: Option<String>,
    my_did: Option<String>,
    my_vk: Option<String>,
    their_did: Option<String>,
    their_vk: Option<String>,
}

impl Trustee {

    fn _generate_trustee_request(&self) -> Result<Value, u32> {
        let cap = expect_ok_or(self.trustee_offer.as_ref(),
                               "Offer must be populated",
                               10 as u32)?;
        let policy_key = settings::get_config_value(settings::CONFIG_AGENT_POLICY_VERKEY)?;

        let cap = cap.capabilities.clone();
        let rtn = TrusteeRequest{
            version: MsgVersion::v0_1,
            msg_type: TrusteeMsgType::TrusteeRequest,
            capabilities: cap,
            authorization_keys: vec![policy_key],
        };
        Ok(serde_json::to_value(&rtn).or(Err(error::INVALID_JSON.code_num))?)
    }


    fn send_request(&mut self, connection_handle: u32) -> Result<u32, u32> {
        if self.state != VcxStateType::VcxStateRequestReceived {
            warn!("Cannot send request when not in VcxStateRequestReceived state");
            return Err(error::INVALID_CONNECTION_STATE.code_num);
        }

        info!("sending trustee request via connection connection: {}", connection_handle);
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


        let request = self._generate_trustee_request()?; //

        let payload = match serde_json::to_string(&request) {
            Ok(p) => p,
            Err(_) => return Err(error::INVALID_JSON.code_num)
        };

        let data: Vec<u8> = connection::generate_encrypted_payload(local_my_vk, local_their_vk, &payload, "TRUSTEE_REQUEST")?;
//        let offer_msg_id = _value_from_json(self.trustee_offer.as_ref(), "msg_uid", "", e_code)?;
        let offer_msg_id = expect_ok_or(self.trustee_offer.as_ref(),
                                        "Expect to have a offer to send request",
                                        10 as u32)?;
        let offer_msg_id = expect_ok_or(offer_msg_id.msg_uid.as_ref(),
                                        "Expect offer to have a msg_uid",
                                        10 as u32)?;

        if settings::test_agency_mode_enabled() { httpclient::set_next_u8_response(SEND_MESSAGE_RESPONSE.to_vec()); }

        match messages::send_message().to(local_my_did)
            .to_vk(local_my_vk)
            .msg_type("trusteeReq")
            .agent_did(local_agent_did)
            .agent_vk(local_agent_vk)
            .edge_agent_payload(&data)
            .ref_msg_id(&offer_msg_id)
            .send_secure() {
            Ok(response) => {
                self.msg_uid = Some(parse_msg_uid(&response[0])?);
                self.state = VcxStateType::VcxStateOfferSent;
                return Ok(error::SUCCESS.code_num)
            },
            Err(x) => {
                warn!("could not send request: {}", x);
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
            if msg.msg_type.eq("trusteeData") {
                match msg.payload {
                    Some(ref data) => {
                        let data = to_u8(data);
                        let data = crypto::parse_msg(wallet::get_wallet_handle(), &my_vk, data.as_slice())?;

                        let trustee = extract_json_payload(&data)?;
                        let trustee: TrusteeData = serde_json::from_str(&trustee).or(Err(10)).unwrap();

                        self.trustee_data = Some(trustee);
                        info!("received trustee");
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

    fn set_offer(&mut self, offer: TrusteeOffer) {
        self.trustee_offer = Some(offer)
    }

    fn get_share(&self) -> Result<RecoveryShare, u32> {
        match self.state {
            VcxStateType::VcxStateAccepted => {
                let trustee_data = expect_ok_or(self.trustee_data.as_ref(),
                                                "Expect trustee Data is populated",
                                                error::INVALID_OPTION.code_num)?;
                let share = trustee_data.share.clone();
                Ok(share)
            },
            _ => {
                Err(error::INVALID_STATE_ERROR.code_num)
            }
        }
    }
}

//********************************************
//         HANDLE FUNCTIONS
//********************************************
fn handle_err(code_num: u32) -> u32 {
    if code_num == error::INVALID_OBJ_HANDLE.code_num {
        error::INVALID_OBJ_HANDLE.code_num // TODO make a error
    }
    else {
        code_num
    }
}

pub fn trustee_create_with_offer(source_id: Option<String>, offer: &str) -> Result<u32, u32> {
    let mut new = _create(source_id);

    let offer: TrusteeOffer = serde_json::from_str(offer).map_err(|_|error::INVALID_JSON.code_num)?;
    new.set_offer(offer);

    new.state = VcxStateType::VcxStateRequestReceived;
    info!("inserting trustee into handle map");
    Ok(HANDLE_MAP.add(new)?)
}

fn _create(source_id: Option<String>) -> Trustee {

    let mut new: Trustee = Default::default();

    new.state = VcxStateType::VcxStateInitialized;
    if let Some(s) = source_id {
        new.set_source_id(s);
    }

    new
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

pub fn list_agents(handle: u32) -> Result<String, u32> {
    HANDLE_MAP.get_mut(handle, |obj| {
        Ok(String::from("[]")) // TODO get device list
    }).map_err(handle_err)
}

pub fn revoke_key(handle: u32, agent_verkey: &str) -> Result<u32, u32> {
    HANDLE_MAP.get_mut(handle, |obj| {
        Ok(error::SUCCESS.code_num) // TODO revoke agent
    }).map_err(handle_err)
}


pub fn send_trustee_request(handle: u32, connection_handle: u32) -> Result<u32, u32> {
    HANDLE_MAP.get_mut(handle, |obj| {
        obj.send_request(connection_handle)
    }).map_err(handle_err)
}

pub fn get_share(handle: u32) -> Result<RecoveryShare, u32> {
    HANDLE_MAP.get(handle, |obj| {
        obj.get_share()
    }).map_err(handle_err)
}

pub fn new_trustee_offer_messages(connection_handle: u32, match_name: Option<&str>) -> Result<String, u32> {
    let my_did = connection::get_pw_did(connection_handle)?;
    let my_vk = connection::get_pw_verkey(connection_handle)?;
    let agent_did = connection::get_agent_did(connection_handle)?;
    let agent_vk = connection::get_agent_verkey(connection_handle)?;

    let payload = messages::get_message::get_all_message(&my_did,
                                                     &my_vk,
                                                     &agent_did,
                                                     &agent_vk)?;

    let mut messages: Vec<TrusteeOffer> = Default::default();

    for msg in payload {
        if msg.msg_type.eq("trusteeOffer") {
            let msg_data = match msg.payload {
                Some(ref data) => {
                    let data = to_u8(data);
                    crypto::parse_msg(wallet::get_wallet_handle(), &my_vk, data.as_slice())?
                },
                None => return Err(10) // TODO better error
            };

            let offer = extract_json_payload(&msg_data)?;

            println!("offer: {}", offer);

            let mut offer: TrusteeOffer = serde_json::from_str(&offer)
                .or(Err(error::INVALID_JSON.code_num))?;

            offer.msg_uid = Some(msg.uid);
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

pub fn from_string(data: &str) -> Result<u32, u32> {
    let new: Trustee = match serde_json::from_str(data) {
        Ok(x) => x,
        Err(y) => return Err(error::INVALID_JSON.code_num),
    };

    let new_handle = HANDLE_MAP.add(new)?;

    info!("inserting handle {} into proof table", new_handle);

    Ok(new_handle)
}


#[cfg(test)]
mod tests {
    extern crate serde_json;

    use super::*;
    use utils::httpclient;
    use utils::dkms_constants::*;

    #[test]
    fn full_trustee_test(){
        ::utils::logger::LoggerUtils::init();
        settings::set_defaults();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");

        let connection_h = connection::build_connection("test_send_claim_offer".to_owned()).unwrap();

        httpclient::set_next_u8_response(NEW_OFFER_RESPONSE.to_vec());

        let offers = new_trustee_offer_messages(connection_h, None).unwrap();
        println!("{}", offers);
        let offers:Value = serde_json::from_str(&offers).unwrap();
        let offers = serde_json::to_string(&offers[0]).unwrap();

        let t_h = trustee_create_with_offer(Some("trustee".to_owned()), &offers).unwrap();

        assert_eq!(3, get_state(t_h).unwrap());

        send_trustee_request(t_h, connection_h).unwrap();

        assert_eq!(2, get_state(t_h).unwrap());

        httpclient::set_next_u8_response(TRUSTEE_DATA_RESPONSE.to_vec());

        update_state(t_h);

        assert_eq!(4, get_state(t_h).unwrap());
    }
}
