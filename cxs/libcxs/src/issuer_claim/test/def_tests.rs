extern crate mockito;

use settings;
use connection::create_connection;
use std::thread;
use std::time::Duration;
use utils::signus::SignusUtils;
use utils::wallet::init_wallet;
use utils::issuer_claim::tests::{put_claim_def_in_issuer_wallet, create_default_schema};
use super::*;

static SCHEMA: &str = r#"{{
                            "seqNo":32,
                            "data":{{
                                "name":"gvt",
                                "version":"1.0",
                                "keys":["address1","address2","city","state", "zip"]
                            }}
                         }}"#;

static CLAIM_REQ_STRING: &str =
    r#"{
           "msg_type":"CLAIM_REQUEST",
           "version":"0.1",
           "to_did":"BnRXf8yDMUwGyZVDkSENeq",
           "from_did":"GxtnGN6ypZYgEqcftSQFnC",
           "iid":"cCanHnpFAD",
           "mid":"",
           "blinded_ms":{
              "prover_did":"FQ7wPBUgSPnDGJnS1EYjTK",
              "u":"923...607",
              "ur":null
           },
           "issuer_did":"QTrbV4raAcND4DWWzBmdsh",
           "schema_seq_no":48,
           "optional_data":{
              "terms_of_service":"<Large block of text>",
              "price":6
           }
        }"#;

static CLAIM_DATA: &str =
    r#"{"address2":["101 Wilson Lane"],
        "zip":["87121"],
        "state":["UT"],
        "city":["SLC"],
        "address1":["101 Tela Lane"]
        }"#;

static X_CLAIM_JSON: &str =
    r#"{"claim":{"address1":["101 Tela Lane","1139481716457488690172217916278103335"],"address2":["101 Wilson Lane","1139481716457488690172217916278103335"],"city":["SLC","1139481716457488690172217916278103335"],"state":["UT","1139481716457488690172217916278103335"],"zip":["87121","1139481716457488690172217916278103335"]},"issuer_did":"NcYxiDXkpYi6ov5FcYDi1e","schema_seq_no":48,"signature":{"non_revocation_claim":null,"primary_claim":{"a":"","e":"","m2":"","v":""}}}"#;

fn util_put_claim_def_in_issuer_wallet(schema_seq_num: u32, wallet_handle: i32) {
    let schema = &create_default_schema(schema_seq_num);

    let stored_xclaim = String::from("");

    info!("wallet_handle: {}", wallet_handle);
    let issuer_did = &settings::get_config_value(settings::CONFIG_ENTERPRISE_DID).unwrap();

    put_claim_def_in_issuer_wallet(issuer_did, schema, wallet_handle);
}

fn set_default_and_enable_test_mode() {
    settings::set_defaults();
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
}

fn stand_up_a_wallet() -> (String, i32, String) {
    let pool_name = String::from("pool1");
    let wallet_name = String::from("wallet1");
    let wallet_type = String::from("default");
    let wallet_handle = init_wallet(&wallet_name, &pool_name, &wallet_type).unwrap();
    info!("Wallet Handle: {}", wallet_handle);
    let (did, _) = SignusUtils::create_and_store_my_did(wallet_handle, None).unwrap();
    info!("Successfully used wallet handle {} to create_and_store_my_did", wallet_handle);
    (wallet_name, wallet_handle, did)
}

fn create_standard_issuer_claim() -> IssuerClaim {
    let claim_req_value = &serde_json::from_str(CLAIM_REQ_STRING).unwrap();
    let issuer_claim = IssuerClaim {
        handle: 123,
        source_id: "standard_claim".to_owned(),
        schema_seq_no: 32,
        msg_uid: "1234".to_owned(),
        claim_attributes: CLAIM_DATA.to_owned(),
        issuer_did: "QTrbV4raAcND4DWWzBmdsh".to_owned(),
        issued_did: "8XFh8yBzrpJQmNyZzgoTqB".to_owned(),
        state: CxsStateType::CxsStateOfferSent,
        claim_name: DEFAULT_CLAIM_NAME.to_owned(),
        claim_request: match ClaimRequest::create_from_api_msg_json(claim_req_value) {
            Ok(x) => Some(x.clone()),
            Err(_) => {
                panic!("invalid claim request for claim {}", 123);
            }
        },
    };
    issuer_claim
}

fn print_error_message(e: &u32) -> () {
    use utils::error::error_message;
    ::utils::logger::LoggerUtils::init();
    info!("error message: {}", error_message(e));
}

fn normalize_claims(c1: &str, c2: &str) -> (serde_json::Value, serde_json::Value) {
    let mut v1: serde_json::Value = serde_json::from_str(c1.clone()).unwrap();
    let mut v2: serde_json::Value = serde_json::from_str(c2.clone()).unwrap();
    v1["signature"]["primary_claim"]["a"] = serde_json::to_value("".to_owned()).unwrap();
    v1["signature"]["primary_claim"]["e"] = serde_json::to_value("".to_owned()).unwrap();
    v1["signature"]["primary_claim"]["v"] = serde_json::to_value("".to_owned()).unwrap();
    v1["signature"]["primary_claim"]["m2"] = serde_json::to_value("".to_owned()).unwrap();
    v2["signature"]["primary_claim"]["a"] = serde_json::to_value("".to_owned()).unwrap();
    v2["signature"]["primary_claim"]["e"] = serde_json::to_value("".to_owned()).unwrap();
    v2["signature"]["primary_claim"]["v"] = serde_json::to_value("".to_owned()).unwrap();
    v2["signature"]["primary_claim"]["m2"] = serde_json::to_value("".to_owned()).unwrap();
    (v1, v2)
}

#[test]
fn test_issuer_claim_create_succeeds() {
    settings::set_defaults();
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
    match issuer_claim_create(0,
                              None,
                              "8XFh8yBzrpJQmNyZzgoTqB".to_owned(),
                              "{\"attr\":\"value\"}".to_owned()) {
        Ok(x) => assert!(x > 0),
        Err(_) => assert_eq!(0, 1), //fail if we get here
    }
}

#[test]
fn test_to_string_succeeds() {
    settings::set_defaults();
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
    let handle = issuer_claim_create(0,
                                     None,
                                     "8XFh8yBzrpJQmNyZzgoTqB".to_owned(),
                                     "{\"attr\":\"value\"}".to_owned()).unwrap();
    let string = to_string(handle).unwrap();
    assert!(!string.is_empty());
}

#[test]
fn test_send_claim_offer() {
    settings::set_defaults();
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
    settings::set_config_value(settings::CONFIG_AGENT_ENDPOINT, mockito::SERVER_URL);

    let connection_handle = create_connection("test_send_claim_offer".to_owned());
    connection::set_pw_did(connection_handle, "8XFh8yBzrpJQmNyZzgoTqB");

    let _m = mockito::mock("POST", "/agency/route")
        .with_status(200)
        .with_body("{\"uid\":\"6a9u7Jt\",\"typ\":\"claimOffer\",\"statusCode\":\"MS-101\"}")
        .expect(1)
        .create();

    let handle = issuer_claim_create(0,
                                     None,
                                     "8XFh8yBzrpJQmNyZzgoTqB".to_owned(),
                                     "{\"attr\":\"value\"}".to_owned()).unwrap();
    thread::sleep(Duration::from_millis(500));
    assert_eq!(send_claim_offer(handle, connection_handle).unwrap(), error::SUCCESS.code_num);
    thread::sleep(Duration::from_millis(500));
    assert_eq!(get_state(handle), CxsStateType::CxsStateOfferSent as u32);
    assert_eq!(get_offer_uid(handle).unwrap(), "6a9u7Jt");
    _m.assert();
}

#[test]
fn test_send_a_claim() {
    let test_name = "test_send_a_claim";
    settings::set_defaults();
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
    settings::set_config_value(settings::CONFIG_AGENT_ENDPOINT, mockito::SERVER_URL);
    settings::set_config_value(settings::CONFIG_ENTERPRISE_DID, "QTrbV4raAcND4DWWzBmdsh");
    wallet::tests::make_wallet(test_name);

    let claim_req_value = &serde_json::from_str(CLAIM_REQ_STRING).unwrap();
    let claim_req:ClaimRequest = match ClaimRequest::create_from_api_msg_json(&claim_req_value) {
        Ok(x) => x,
        Err(_) => panic!("error with claim request"),
    };
    let issuer_did = claim_req.issuer_did;
    let _m = mockito::mock("POST", "/agency/route")
        .with_status(200)
        .with_body("{\"uid\":\"6a9u7Jt\",\"typ\":\"claim\",\"statusCode\":\"MS-101\"}")
        .expect(1)
        .create();

    let mut claim = create_standard_issuer_claim();
    claim.state = CxsStateType::CxsStateRequestReceived;
    util_put_claim_def_in_issuer_wallet(48, wallet::get_wallet_handle());

    let connection_handle = create_connection("test_send_claim_offer".to_owned());
    connection::set_pw_did(connection_handle, "8XFh8yBzrpJQmNyZzgoTqB");

    match claim.send_claim(connection_handle) {
        Ok(_) => assert_eq!(0, 0),
        Err(x) => {
            info!("error message: {}", error::error_message(&x));
            assert_eq!(x, 0)
        },
    };
    _m.assert();
    assert_eq!(claim.state, CxsStateType::CxsStateAccepted);
    wallet::close_wallet(wallet::get_wallet_handle()).unwrap();
    wallet::delete_wallet(test_name).unwrap();
}

#[test]
fn test_from_string_succeeds() {
    set_default_and_enable_test_mode();
    let handle = issuer_claim_create(0,
                                     None,
                                     "8XFh8yBzrpJQmNyZzgoTqB".to_owned(),
                                     "{\"attr\":\"value\"}".to_owned()).unwrap();
    let string = to_string(handle).unwrap();
    assert!(!string.is_empty());
    release(handle);
    let new_handle = from_string(&string).unwrap();
    let new_string = to_string(new_handle).unwrap();
    assert_eq!(new_handle, handle);
    assert_eq!(new_string, string);
}

#[test]
fn test_update_state_with_pending_claim_request() {
    settings::set_defaults();
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
    settings::set_config_value(settings::CONFIG_AGENT_ENDPOINT, mockito::SERVER_URL);

    let response = "{\"msgs\":[{\"uid\":\"6gmsuWZ\",\"typ\":\"conReq\",\"statusCode\":\"MS-102\",\"statusMsg\":\"message sent\"},\
        {\"statusCode\":\"MS-104\",\"edgeAgentPayload\":\"{\\\"attr\\\":\\\"value\\\"}\",\"sendStatusCode\":\"MSS-101\",\"typ\":\"claimOffer\",\"statusMsg\":\"message accepted\",\"uid\":\"6a9u7Jt\",\"refMsgId\":\"CKrG14Z\"},\
        {\"msg_type\":\"CLAIM_REQUEST\",\"typ\":\"claimReq\",\"edgeAgentPayload\":\"{\\\"blinded_ms\\\":{\\\"prover_did\\\":\\\"FQ7wPBUgSPnDGJnS1EYjTK\\\",\\\"u\\\":\\\"923...607\\\",\\\"ur\\\":\\\"null\\\"},\\\"version\\\":\\\"0.1\\\",\\\"mid\\\":\\\"\\\",\\\"to_did\\\":\\\"BnRXf8yDMUwGyZVDkSENeq\\\",\\\"from_did\\\":\\\"GxtnGN6ypZYgEqcftSQFnC\\\",\\\"iid\\\":\\\"cCanHnpFAD\\\",\\\"issuer_did\\\":\\\"QTrbV4raAcND4DWWzBmdsh\\\",\\\"schema_seq_no\\\":48,\\\"optional_data\\\":{\\\"terms_of_service\\\":\\\"<Large block of text>\\\",\\\"price\\\":6}}\"}]}";
    let _m = mockito::mock("POST", "/agency/route")
        .with_status(200)
        .with_body(response)
        .expect(2)
        .create();

    let claim_req_value = &serde_json::from_str(CLAIM_REQ_STRING).unwrap();
    let mut claim = IssuerClaim {
        handle: 123,
        source_id: "test_has_pending_claim_request".to_owned(),
        schema_seq_no: 32,
        msg_uid: "1234".to_owned(),
        claim_attributes: "nothing".to_owned(),
        issuer_did: "QTrbV4raAcND4DWWzBmdsh".to_owned(),
        issued_did: "8XFh8yBzrpJQmNyZzgoTqB".to_owned(),
        state: CxsStateType::CxsStateOfferSent,
        claim_request: match ClaimRequest::create_from_api_msg_json(claim_req_value) {
            Ok(x) => Some(x.clone()),
            Err(_) => {
                panic!("invalid claim request for claim {}", 123);
            }
        },
        claim_name: DEFAULT_CLAIM_NAME.to_owned(),
    };

    claim.update_state();
    _m.assert();
    assert_eq!(claim.get_state(), CxsStateType::CxsStateRequestReceived as u32);
    let claim_request = claim.claim_request.unwrap();
    assert_eq!(claim_request.issuer_did, "QTrbV4raAcND4DWWzBmdsh");
    assert_eq!(claim_request.schema_seq_no, 48);
}

#[test]
fn test_issuer_claim_changes_state_after_being_validated() {
    set_default_and_enable_test_mode();
    let handle = issuer_claim_create(0,
                                     None,
                                     "8XFh8yBzrpJQmNyZzgoTqB".to_owned(),
                                     "{\"att\":\"value\"}".to_owned()).unwrap();
    let string = to_string(handle).unwrap();
    fn get_state_from_string(s: String) -> u32 {
        let json: serde_json::Value = serde_json::from_str(&s).unwrap();
        if json["state"].is_number() {
            return json["state"].as_u64().unwrap() as u32
        }
        0
    }
    assert_eq!(get_state_from_string(string), 1);
}

#[test]
fn test_issuer_claim_can_build_claim_from_correct_parts() {
    let test_name = "test_issuer_claim_can_build_from_correct_parts";
    ::utils::logger::LoggerUtils::init();
    let schema_str = SCHEMA;
    let mut issuer_claim = create_standard_issuer_claim();
    let issuer_did = "NcYxiDXkpYi6ov5FcYDi1e".to_owned();
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
    settings::set_config_value(settings::CONFIG_AGENT_ENDPOINT, mockito::SERVER_URL);
    settings::set_config_value(settings::CONFIG_ENTERPRISE_DID, &issuer_did);
    wallet::tests::make_wallet(test_name);
    let wallet_handle = wallet::get_wallet_handle();
    SignusUtils::create_and_store_my_did(wallet_handle, None).unwrap();
    util_put_claim_def_in_issuer_wallet(48, wallet_handle);

    // set the claim request issuer did to the correct (enterprise) did.
    let mut claim_req = issuer_claim.claim_request.clone().unwrap();
    claim_req.issuer_did = issuer_did.to_owned();
    println!("IssuerClaim: {}", serde_json::to_string_pretty(&issuer_claim).unwrap());
    issuer_claim.claim_request = Some(claim_req);
    let encoded_claim_data = issuer_claim.create_attributes_encodings().unwrap();
    let claim_payload = match create_claim_payload_using_wallet(&issuer_claim.claim_request.clone().unwrap(), &encoded_claim_data, wallet::get_wallet_handle()) {
        Ok(c) => c,
        Err(_) => panic!("Error creating claim payload"),
    };
    let claim_payload_json: serde_json::Value = serde_json::from_str(&claim_payload).unwrap();
    let x_claim_json: serde_json::Value = serde_json::from_str(X_CLAIM_JSON).unwrap();

    // remove primary claims signatures
    // as they will never match
    let (n1, n2) = normalize_claims(&claim_payload, &X_CLAIM_JSON);

    assert_eq!(serde_json::to_string(&n1).unwrap(), serde_json::to_string(&n2).unwrap());
    wallet::close_wallet(wallet_handle).unwrap();
    wallet::delete_wallet(test_name).unwrap();
}

#[test]
fn test_issuer_claim_request_changes_reflect_in_claim_payload() {
    // TODO: Is this duplicate of the above test?
    ::utils::logger::LoggerUtils::init();
    settings::set_defaults();
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
    settings::set_config_value(settings::CONFIG_ENTERPRISE_DID, "NcYxiDXkpYi6ov5FcYDi1e");
    wallet::tests::make_wallet("test_issuer_claim_request_changes_reflect_in_claim");
    let wallet_handle = wallet::get_wallet_handle();

    util_put_claim_def_in_issuer_wallet(48, wallet_handle);
    let issuer_claim = create_standard_issuer_claim();
    let mut claim_request = issuer_claim.claim_request.clone().unwrap();
    claim_request.issuer_did = String::from("NcYxiDXkpYi6ov5FcYDi1e");
    assert_eq!(claim_request.schema_seq_no, 48);
    info!("claim request: {:?}", serde_json::to_string(&claim_request));
    info!("claim data: {:?}", &CLAIM_DATA);
    let encoded = issuer_claim.create_attributes_encodings().unwrap();
    let claim_payload = match create_claim_payload_using_wallet(&claim_request, &encoded, wallet_handle) {
        Ok(c) => c,
        Err(_) => panic!("Error creating claim payload"),
    };

    let (n1, n2) = normalize_claims(&claim_payload, &X_CLAIM_JSON);
    info!("claim_payload: {}", claim_payload);
    assert_eq!(n1, n2);

    wallet::close_wallet(wallet_handle).unwrap();
    wallet::delete_wallet("test_issuer_claim_request_changes_reflect_in_claim").unwrap();
}

#[test]
fn basic_add_attribute_encoding() {
    ::utils::logger::LoggerUtils::init();
    // FIXME Make this a real test and add additional test for create_attributes_encodings
    let issuer_claim = create_standard_issuer_claim();
    match issuer_claim.create_attributes_encodings() {
        Ok(_) => assert!(true),
        Err(e) => {
            error!("Error in create_attributes_encodings test");
            assert_eq!(0, 1)
        },
    };

    let mut issuer_claim = create_standard_issuer_claim();
    match issuer_claim.claim_attributes.pop() {
        Some(brace) => assert_eq!(brace, '}'),
        None => error!("Malformed claim attributes in the issuer claim test"),
    }
    match issuer_claim.create_attributes_encodings() {
        Ok(_) => {
            error!("basic_add_attribute_encoding test should raise error.");
            assert_ne!(1, 1);
        },
        Err(e) => assert_eq!(error::INVALID_JSON.code_num, e),
    }
}

#[test]
fn test_claim_offer_has_proper_fields_for_sending_message() {
    static CORRECT_CLAIM_OFFER_PAYLOAD: &str = r#"{"msg_type":"CLAIM_OFFER","version":"0.1","to_did":"BnRXf8yDMUwGyZVDkSENeq","from_did":"GxtnGN6ypZYgEqcftSQFnC","iid":"cCanHnpFAD","mid":"","claim":{"name":["Alice"],"date_of_birth":["2000-05-17"],"height":["175"]},"schema_seq_no":103,"issuer_did":"V4SGRU86Z58d6TV7PBUe6f","nonce":"351590","claim_name":"Profiledetail","issuer_name":"TestEnterprise","optional_data":{"terms_of_service":"<Largeblockoftext>","price":6}}"#;
    println!("{:?}", json!(&CORRECT_CLAIM_OFFER_PAYLOAD));
    let issuer_claim = IssuerClaim::create_standard_issuer_claim().unwrap();
    assert_eq!(issuer_claim.claim_name, DEFAULT_CLAIM_NAME);
}

#[ignore]
#[test]
fn test_claim_offer_payload_includes_claim_name_field() {
    let mut issuer_claim = IssuerClaim::create_standard_issuer_claim().unwrap();
    issuer_claim.claim_attributes = String::from("{\"value\":\"pair\"}");
    let to_did = "FOOBAR";
    let from_did = "BARFOO";
    let payload = issuer_claim.create_send_claim_offer_payload(&to_did, &from_did).unwrap();
    let payload_json: serde_json::Value = serde_json::from_str(&payload).unwrap();
    let payload_raw = format!("{{\"msg_type\":\"CLAIM_OFFER\",\"version\":\"0.1\",\"to_did\":\"{}\",\"from_did\":\"{}\",\
                \"claim\":{},\"schema_seq_no\":{},\"issuer_did\":\"{},\"claim_name\":{}\"}}",
                              to_did, from_did, issuer_claim.claim_attributes, issuer_claim.schema_seq_no, issuer_claim.issuer_did, issuer_claim.claim_name);
    assert_eq!(payload_raw, payload_json.to_string())
}

#[test]
fn test_that_test_mode_enabled_bypasses_libindy_create_claim(){
    ::utils::logger::LoggerUtils::init();
    let test_name = "test_that_TEST_MODE_ENABLED_bypasses_libindy_create_claim";
    settings::set_defaults();
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
    settings::set_config_value(settings::CONFIG_ENTERPRISE_DID, "QTrbV4raAcND4DWWzBmdsh");

    let claim_req_value = &serde_json::from_str(CLAIM_REQ_STRING).unwrap();
    let claim_req:ClaimRequest = match ClaimRequest::create_from_api_msg_json(&claim_req_value) {
        Ok(x) => x,
        Err(_) => panic!("error with claim request"),
    };
    let issuer_did = claim_req.issuer_did;

    let mut claim = create_standard_issuer_claim();
    claim.state = CxsStateType::CxsStateRequestReceived;

    let connection_handle = create_connection("test_send_claim_offer".to_owned());
    connection::set_pw_did(connection_handle, "8XFh8yBzrpJQmNyZzgoTqB");
    match claim.send_claim(connection_handle) {
        Ok(_) => assert_eq!(0, 0),
        Err(x) => {
            info!("error message: {}", error::error_message(&x));
            assert_eq!(x, 0)
        },
    };
    assert_eq!(claim.state, CxsStateType::CxsStateAccepted);

}