extern crate serde_json;

use object_cache::ObjectCache;
use api::{ VcxStateType };
use utils::error;
use connection;
use messages;
use messages::GeneralMessage;
use messages::proofs::proof_message::{ProofMessage };
use messages::proofs::proof_request::{ ProofRequestMessage };
use messages::extract_json_payload;
use messages::to_u8;

use claim_def::{ RetrieveClaimDef, ClaimDefCommon };
use schema::LedgerSchema;

use utils::libindy::anoncreds;
use utils::libindy::wallet;
use utils::libindy::SigTypes;
use utils::libindy::crypto;

use utils::option_util::expect_ok_or;

use serde_json::Value;
use serde_json::Map;

use settings;
use utils::httpclient;
use utils::constants::SEND_MESSAGE_RESPONSE;

lazy_static! {
    static ref HANDLE_MAP: ObjectCache<DisclosedProof>  = Default::default();
}

impl Default for DisclosedProof {
    fn default() -> DisclosedProof
    {
        DisclosedProof {
            source_id: String::new(),
            msg_uid: None,
            my_did: None,
            my_vk: None,
            state: VcxStateType::VcxStateNone,
            proof_request: None,
            link_secret_alias: Some(String::from("main")), //TODO this should not be hardcoded
//            proof_attributes: None,
            their_did: None,
            their_vk: None,
            agent_did: None,
            agent_vk: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DisclosedProof {
    source_id: String,
    msg_uid: Option<String>,
    my_did: Option<String>,
    my_vk: Option<String>,
    state: VcxStateType,
    proof_request: Option<ProofRequestMessage>,
    link_secret_alias: Option<String>,
//    proof_attributes: Option<String>,
    their_did: Option<String>,
    their_vk: Option<String>,
    agent_did: Option<String>,
    agent_vk: Option<String>,
}


fn _match_claim(claims: &Value, id: &str) -> Option<(String, String, u32)> {
    let claims = match claims {
        &Value::Array(ref list) => list,
        _ => return None
    };
    for claim in claims.iter() {
        let claim_id = &claim["claim_uuid"];
        if let &Value::String(ref str) = claim_id {
            if str.eq(id) {

                fn get_val(val: Option<&Value>) -> Option<&String> {
                    match val {
                        Some(did) => {
                            match did {
                                &Value::String(ref s) => Some(s),
                                _ => None
                            }
                        },
                        None => None
                    }
                }
                let issuer_did = get_val(claim.get("issuer_did"));
                let issuer_did = match issuer_did {
                    Some(v) => v,
                    None => continue
                };

                let schema_seq_no = get_val(claim.get("schema_seq_no"));
                let schema_seq_no = match schema_seq_no {
                    Some(v) => v,
                    None => continue
                };
                let schema_seq_no = match schema_seq_no.parse::<u32>(){
                    Ok(i) => i,
                    Err(_) => continue
                };

                return Some((String::from(id), issuer_did.to_owned(), schema_seq_no))

            }
        }
    }
    None
}

fn claim_def_identifiers(claims: &str) -> Result<Vec<(String, String, String, u64)>,u32>{
    let mut rtn = Vec::new();

    let claims: Value = serde_json::from_str(&claims)
        .or(Err(error::INVALID_JSON.code_num))?;

    if let Value::Object(ref map) = claims["attrs"] {
        for (key, value) in map {
            if let Value::Object(ref attr_obj) = value[0] {
                let claim_uuid = match attr_obj["claim_uuid"] {
                    Value::String(ref s) => s,
                    _ => return Err(error::INVALID_JSON.code_num)
                };

                let issuer_did = match attr_obj["issuer_did"] {
                    Value::String(ref s) => s,
                    _ => return Err(error::INVALID_JSON.code_num)
                };

                let schema_seq_no = match attr_obj["schema_seq_no"] {
                    Value::Number(ref n) => match n.as_u64() {
                        Some(i) => i,
                        None => return Err(error::INVALID_JSON.code_num)
                    },
                    _ => return Err(error::INVALID_JSON.code_num)
                };

                rtn.push((key.to_owned(),
                          claim_uuid.to_owned(),
                          issuer_did.to_owned(),
                          schema_seq_no))
            }
        }
    }
    else {
        return Err(error::INVALID_JSON.code_num);
    }

    Ok(rtn)
}


impl DisclosedProof {

    fn set_proof_request(&mut self, req: ProofRequestMessage) {self.proof_request = Some(req)}

    fn get_state(&self) -> u32 {self.state as u32}
    fn set_state(&mut self, state: VcxStateType) {self.state = state}

    fn _find_schemas(&self, claims_identifers: &Vec<(String, String, String, u64)>) -> Result<String, u32> {
        let mut rtn = Map::new();

        for &(ref attr_id, ref claim_uuid, ref issuer_did, schema_seq_num) in claims_identifers {
            let schema = LedgerSchema::new_from_ledger(schema_seq_num as i32)?;
            let schema = schema.data.ok_or(10 as u32)?;

            let schema: Value = serde_json::to_value(schema)
                .or(Err(10 as u32))?;

            rtn.insert(claim_uuid.to_owned(), schema);
        }


        match rtn.is_empty() {
            false => Ok(serde_json::to_string(&Value::Object(rtn))
                .or(Err(10 as u32))?),
            true => Err(10) // DID NOT FIND NEEDED VALUES
        }
    }

    fn _find_claim_def(&self, claims_identifers: &Vec<(String, String, String, u64)>) -> Result<String, u32> {

        let mut rtn = Map::new();

        for &(ref attr_id, ref claim_uuid, ref issuer_did, schema_seq_num) in claims_identifers {
            let claim_def = RetrieveClaimDef::new()
                .retrieve_claim_def("GGBDg1j8bsKmr4h5T9XqYf",
                                    schema_seq_num as u32,
                                    Some(SigTypes::CL),
                                    &issuer_did)?;

            let claim_def: Value = serde_json::from_str(&claim_def)
                .or(Err(10 as u32))?;

            rtn.insert(claim_uuid.to_owned(), claim_def);
        }


        match rtn.is_empty() {
            false => Ok(serde_json::to_string(&Value::Object(rtn))
                            .or(Err(10 as u32))?),
            true => Err(10) // DID NOT FIND NEEDED VALUES
        }
    }

    fn _build_requested_claims(&self, claims_identifiers: &Vec<(String, String, String, u64)>) -> Result<String, u32> {
        let mut rtn: Value = json!({
              "self_attested_attributes":{},
              "requested_attrs":{},
              "requested_predicates":{}
        });
        if let Value::Object(ref mut map) = rtn["requested_attrs"] {
            for &(ref attr_id, ref claim_uuid, ref issuer_did, schema_seq_num) in claims_identifiers {
                let insert_val = json!([claim_uuid, true]);
                map.insert(attr_id.to_owned(), insert_val);
            }
        }

        let rtn = serde_json::to_string_pretty(&rtn).or(Err(error::INVALID_JSON.code_num))?;
        Ok(rtn)

    }

    fn _build_proof(&self) -> Result<ProofMessage, u32> {

        let wallet_h = wallet::get_wallet_handle();

//        let attributes = expect_ok_or(self.proof_attributes.as_ref(),
//                                      "Expect proof_attributes to not be None",
//                                      10 as u32)?;

        let proof_req = expect_ok_or(self.proof_request.as_ref(),
                                      "Expect req to not be None",
                                      10 as u32)?;
        let proof_req_data_json = serde_json::to_string(&proof_req.proof_request_data).or(Err(10 as u32))?;

        let claims = anoncreds::libindy_prover_get_claims(wallet_h,
                                                          &proof_req_data_json)?;

        let claims_identifiers = claim_def_identifiers(&claims)?;
        let requested_claims = self._build_requested_claims(&claims_identifiers)?;

        let schemas = self._find_schemas(&claims_identifiers)?;
        let master_secret = expect_ok_or(self.link_secret_alias.as_ref(),
                                         "Expect Link Secret to not be None",
                                         10 as u32)?;
        let claim_defs_json = self._find_claim_def(&claims_identifiers)?;
        let revoc_regs_json = Some("{}");

        let proof = anoncreds::libindy_prover_create_proof(wallet_h,
                                                          &proof_req_data_json,
                                                           &requested_claims,
                                                          &schemas,
                                                          master_secret,
                                                          &claim_defs_json,
                                                          revoc_regs_json)?;

        let proof: ProofMessage = serde_json::from_str(&proof)
            .or(Err(error::UNKNOWN_LIBINDY_ERROR.code_num))?;

        Ok(proof)
    }

    fn send_proof(&mut self, connection_handle: u32) -> Result<u32, u32> {
//        if self.proof_attributes.is_none(){
//            warn!("trying to send proof without setting attributes");
//            return Err(1) //TODO NEED ERROR CODE!!!
//        }

        info!("sending proof via connection connection: {}", connection_handle);
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

        let proof_req = self.proof_request.as_ref().ok_or(e_code)?;
        let ref_msg_uid = proof_req.msg_ref_id.as_ref().ok_or(e_code)?;

        let proof = match settings::test_indy_mode_enabled() {
            false => {
                let proof: ProofMessage = self._build_proof()?;
                serde_json::to_string(&proof).or(Err(error::INVALID_JSON.code_num))?
            },
            true => String::from("dummytestmodedata")
        };

        let data: Vec<u8> = connection::generate_encrypted_payload(local_my_vk, local_their_vk, &proof, "PROOF")?;

        if settings::test_agency_mode_enabled() { httpclient::set_next_u8_response(SEND_MESSAGE_RESPONSE.to_vec()); }

        match messages::send_message().to(local_my_did)
            .to_vk(local_my_vk)
            .msg_type("proof")
            .agent_did(local_agent_did)
            .agent_vk(local_agent_vk)
            .edge_agent_payload(&data)
            .ref_msg_id(ref_msg_uid)
            .send_secure() {
            Ok(response) => {
//                self.msg_uid = get_proof_details(&response[0])?;
                self.state = VcxStateType::VcxStateAccepted;
                return Ok(error::SUCCESS.code_num)
            },
            Err(x) => {
                warn!("could not send proof: {}", x);
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
        error::INVALID_DISCLOSED_PROOF_HANDLE.code_num
    }
    else {
        code_num
    }
}

pub fn create_proof(source_id: Option<String>, proof_req: &str) -> Result<u32, u32> {
    info!("creating disclosed proof with id: {}", source_id.unwrap_or("UNDEFINED".to_string()));

    let mut new_proof: DisclosedProof = Default::default();

    new_proof.set_proof_request(serde_json::from_str(proof_req)
        .map_err(|_|error::INVALID_JSON.code_num)?);

    new_proof.set_state(VcxStateType::VcxStateRequestReceived);

    Ok(HANDLE_MAP.add(new_proof)?)
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

pub fn from_string(proof_data: &str) -> Result<u32, u32> {
    let derived_proof: DisclosedProof = match serde_json::from_str(proof_data) {
        Ok(x) => x,
        Err(y) => return Err(error::INVALID_JSON.code_num),
    };

    let new_handle = HANDLE_MAP.add(derived_proof)?;

    info!("inserting handle {} into proof table", new_handle);

    Ok(new_handle)
}

pub fn release(handle: u32) -> Result<(), u32> {
    HANDLE_MAP.release(handle).map_err(handle_err)
}

pub fn send_proof(handle: u32, connection_handle: u32) -> Result<u32,u32> {
    HANDLE_MAP.get_mut(handle, |obj|{
        obj.send_proof(connection_handle)
    })
}

pub fn is_valid_handle(handle: u32) -> bool {
    HANDLE_MAP.has_handle(handle)
}

//TODO one function with claim
pub fn new_proof_requests_messages(connection_handle: u32, match_name: Option<&str>) -> Result<String, u32> {
    let my_did = connection::get_pw_did(connection_handle)?;
    let my_vk = connection::get_pw_verkey(connection_handle)?;
    let agent_did = connection::get_agent_did(connection_handle)?;
    let agent_vk = connection::get_agent_verkey(connection_handle)?;

    let payload = messages::get_message::get_all_message(&my_did,
                                                         &my_vk,
                                                         &agent_did,
                                                         &agent_vk)?;

    let mut messages: Vec<ProofRequestMessage> = Default::default();

    for msg in payload {
        if msg.sender_did.eq(&my_did){ //Do not want message sent by me
            continue;
        }

        if msg.msg_type.eq("proofReq") {
            let msg_data = match msg.payload {
                Some(ref data) => {
                    let data = to_u8(data);
                    crypto::parse_msg(wallet::get_wallet_handle(), &my_vk, data.as_slice())?
                },
                None => return Err(10) // TODO better error
            };

            let req = extract_json_payload(&msg_data)?;

            let mut req: ProofRequestMessage = serde_json::from_str(&req)
                .or(Err(error::INVALID_JSON.code_num))?;

            req.msg_ref_id = Some(msg.uid.to_owned());
            messages.push(req);
        }
    }


    Ok(serde_json::to_string_pretty(&messages).unwrap())
}

#[cfg(test)]
mod tests {
    extern crate serde_json;

    use super::*;
    use messages;
    use utils::httpclient;
    use utils::dkms_constants::*;


    fn build_proof_request() -> String {
        let mut proof_obj = messages::proof_request();
        proof_obj
            .type_version("0.1")
            .tid(1)
            .mid(9)
            .nonce("95595")
            .proof_name("Test")
            .proof_data_version("0.1")
            .requested_attrs("[{\"name\":\"person name\"}]")
            .requested_predicates("[]")
            .serialize_message().expect("Proof Request Message should work in tests")
    }

    #[test]
    #[ignore] // Need to mock indy ledger calls
    fn full_disclosed_proof_test() {
        ::utils::logger::LoggerUtils::init();
        settings::set_defaults();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
//
        let connection_h = connection::build_connection("test_send_claim_offer".to_owned()).unwrap();

        httpclient::set_next_u8_response(NEW_PROOF_REQUEST_RESPONSE.to_vec());

        let requests = new_proof_requests_messages(connection_h, None).unwrap();
        println!("{}", requests);
        let requests:Value = serde_json::from_str(&requests).unwrap();
        let requests = serde_json::to_string(&requests[0]).unwrap();


        let handle = create_proof(Some("TEST_CLAIM".to_owned()), &requests).unwrap();
        assert_eq!(3, get_state(handle).unwrap());

        send_proof(handle, connection_h).unwrap();

        assert_eq!(4, get_state(handle).unwrap());
    }

    #[test]
    fn get_state_test(){
//        ::utils::logger::LoggerUtils::init();

        let proof: DisclosedProof =  Default::default();

        assert_eq!(0, proof.get_state());


        let handle = create_proof(Some("id".to_string()),
                                  &build_proof_request()).unwrap();
        assert_eq!(1, get_state(handle).unwrap())
    }

    #[test]
    fn to_string_test() {
        let handle = create_proof(Some("id".to_string()),
                                  &build_proof_request()).unwrap();

        let serialized = to_string(handle);
        assert!(serialized.is_ok());

        assert!(from_string(serialized.unwrap().as_str()).is_ok());
    }

    #[test]
    fn claim_keys() {
        let test_json = json!(
            {
              "self_attested_attributes":{},
              "requested_attrs":{
                "sdf":[
                  "claim::e5fec91f-d03d-4513-813c-ab6db5715d55",
                  true
                ],
                "ddd":[
                  "claim::e5fec91f-d03d-4513-813c-ab6db5715d55",
                  true
                ]
              },
              "requested_predicates":{}
            }
        );
    }

}
