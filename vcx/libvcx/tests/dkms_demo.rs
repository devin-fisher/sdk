#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_json;
extern crate rand;
extern crate tempdir;
extern crate vcx;

use std::env;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time;
use std::path::Path;
use std::sync::Mutex;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;
use std::ops::Deref;
use std::path::PathBuf;

use serde_json::Value;


mod utils;
use utils::wallet;
use utils::api_caller;

mod dkms;
use dkms::constants::{DEV_GENESIS_NODE_TXNS,
                      config_info,
                      wallet_entries,
                      asset_name,
                      ACCOUNT_CERT_SCHEMA_SEQ_NUM,
                      ACCOUNT_CERT_DID};
use dkms::constants;
use dkms::actor::Actor;
use dkms::util::{print_chapter, find_actor, pr_json, send_via_file, receive_via_file, should_print_wait_msg};
use utils::wallet::wallet_file;

use vcx::settings::DEFAULT_GENESIS_PATH;

const INVITE_CUNION_ALICE_PATH: &'static str = "/tmp/cunion-alice-invite.json";
const INVITE_BOB_ALICE_PATH: &'static str = "/tmp/alice-bob-invite.json";
const INVITE_ALICE_DAKOTA_PATH: &'static str = "/tmp/alice-dakota-invite.json";

const INVITE_RECOVERY_BOB_ALICE_PATH: &'static str = "/tmp/recovery-alice-bob-invite.json";
const INVITE_RECOVERY_CUNION_ALICE_PATH: &'static str = "/tmp/recovery-alice-cunion-invite.json";

lazy_static! {
    static ref DB: Mutex<HashMap<String, String>> = Default::default();
}

fn db_put(key: &str, val: String) -> Result<(),()>{
    match DB.lock().unwrap().insert(key.to_owned(), val) {
        Some(_) => (),
        None => ()
    }

    Ok(())
}

fn db_get(key: &str) -> Option<String> {
    match DB.lock().unwrap().get(key) {
        Some(v) => Some(v.to_owned()),
        None => None
    }
}

fn db_write_file(dir: &Path) {
    let out = serde_json::to_string_pretty(&DB.lock().unwrap().deref()).unwrap();

    let file_dir = dir.join(Path::new("DB"));

    let mut f = File::create(file_dir).unwrap();
    f.write_all(out.as_bytes()).unwrap();
}

fn print_err<T: Debug>(err: T) -> T {
    println!("ERROR: {:?}", err);
    err
}

fn await_state(handle: u32, state: u32, func: api_caller::fn_u32_r_u32_u32, timeout: Option<u64>) -> Result<(), u32> {
    let timeout = timeout.unwrap_or(20 as u64);
    let mut should_print = 0;
    loop {

        let cur_state = api_caller::u32_r_u32(handle, func)?;
        if cur_state == state {
            break;
        }

        should_print = should_print_wait_msg(format!("waiting for state: {} -- currently: {}", state, cur_state).as_ref(),
                                            should_print,
                                            4);
        thread::sleep(Duration::from_secs(timeout/10))
    }

    println!("reached state: {}", state);
    Ok(())
}

fn await_message(conn_handle: u32, msg_type: &str, func: api_caller::fn_u32_r_u32_str, timeout: Option<u64>) -> Result<String, u32> {
    let timeout = timeout.unwrap_or(20 as u64);
    let mut should_print = 0;
    loop {

        let msg = api_caller::u32_r_u32_str(conn_handle, func).unwrap();
        let msg = msg.unwrap_or(String::from("[]"));

//        println!("{}", msg);

        if msg.contains(msg_type) {
            println!("received message of type: {}", msg_type);
            return Ok(msg);
        }

        should_print = should_print_wait_msg(format!("waiting for messages with type: {}", msg_type).as_ref(),
                                            should_print,
                                            4);
        thread::sleep(Duration::from_secs(timeout/10))
    }
}

fn send_conn(other_party: &str, path: &Path) -> Result<u32, u32> {
    let alice_h = api_caller::str_r_u32(other_party,
                                        vcx::api::connection::vcx_connection_create
    ).map_err(print_err)?;

    println!("Connection handle: {}", alice_h);

    let invite = api_caller::u32_str_r_u32_str(alice_h,
                                               "",
                                               vcx::api::connection::vcx_connection_connect).map_err(print_err)?;
    let invite = invite.unwrap();

    println!("Connection Details: \n");
    pr_json(&invite);


    println!("Sending invite via file at {:?}", path);
    send_via_file(&invite, &path, None).unwrap();

    await_state(alice_h,
                4, //VcxStateAccepted
                vcx::api::connection::vcx_connection_update_state,
                None).map_err(print_err)?;

    Ok(alice_h)
}

fn receive_conn(other_party: &str, path: &Path) -> Result<u32, u32> {
    let invite = receive_via_file(path, None).unwrap();

    println!("Connection Details: \n");
    pr_json(&invite);

    println!("Creating connection with {}", other_party);
    let handle = api_caller::str_str_r_u32(other_party,
                                             &invite,
                                             vcx::api::connection::vcx_connection_create_with_invite
        ).map_err(print_err)?;

    println!("Connection handle: {}", handle);

    println!("Connecting to connection with {}", other_party);
    let _invite = api_caller::u32_str_r_u32_str(handle,
                                               "",
                                               vcx::api::connection::vcx_connection_connect
    ).map_err(print_err)?;

    await_state(handle,
                4, //VcxStateAccepted
                vcx::api::connection::vcx_connection_update_state,
                None).map_err(print_err)?;

    Ok(handle)
}

fn prep_backup_file(actor: &Actor, actor_dir: &Path) -> String {
    let mut rtn: Vec<PathBuf> = Vec::new();
    db_write_file(actor_dir);
    rtn.push(actor_dir.join("DB"));
    rtn.push(actor_dir.join("config.json"));
    rtn.push(wallet_file(&asset_name(actor)).unwrap());

    serde_json::to_string(&rtn).unwrap()
}

fn chapter_1_demo(actor: &Actor) {
    print_chapter("CHAPTER ONE", None);

    match actor {
        &Actor::Alice => {
            println!("ENTER ALICE");
            let invite_path = Path::new(INVITE_CUNION_ALICE_PATH);
            let cunion_h = receive_conn("CUNION", invite_path).expect("Should connect to CUNION");


            println!("Look for claim offer");
            let offers = await_message(cunion_h,
                                       "CLAIM_OFFER",
                                       vcx::api::claim::vcx_claim_new_offers,
                                       None).unwrap();



            println!("Offers:\n{}", offers);

            let offers: Value = serde_json::from_str(&offers).unwrap();

            let account_cert_offer = &offers[0];

            let claim_h = api_caller::str_str_r_u32("bank account",
                                                    &serde_json::to_string(account_cert_offer).unwrap(),
                                                    vcx::api::claim::vcx_claim_create_with_offer).unwrap();

            api_caller::u32_u32_r_u32(claim_h,
                                      cunion_h,
                                      vcx::api::claim::vcx_claim_send_request).unwrap();



            println!("Look for claim");
            await_state(claim_h,
                        4, //VcxStateAccepted
                        vcx::api::claim::vcx_claim_update_state,
                        None).unwrap();

            println!("Time passes and Alice calls again and must be authenticated.");
            println!("Look for auth trust ping");
            let pings = await_message(cunion_h,
                                       "TRUST_PING",
                                       vcx::api::trust_pong::vcx_trust_pong_new_pings,
                                       None).unwrap();

            println!("Ping:\n{}", pings);

            let pings: Value = serde_json::from_str(&pings).unwrap();

            let ping = &pings[0];

            let pong_h = api_caller::str_str_r_u32("auth req from cunion",
                                                    &serde_json::to_string(ping).unwrap(),
                                                    vcx::api::trust_pong::vcx_trust_pong_create_with_request).unwrap();

            println!("Pong Handle: {}", pong_h);

            api_caller::u32_u32_r_u32(pong_h,
                                      cunion_h,
                                      vcx::api::trust_pong::vcx_trust_pong_send_proof).unwrap();

            await_state(pong_h,
                        4, //VcxStateAccepted
                        vcx::api::trust_pong::vcx_trust_pong_update_state,
                        None).unwrap();

            println!("Pong was send!");

            db_put("cunion_for_alice_h", format!("{}", cunion_h));

        },
        &Actor::CUnion => {
            println!("ENTER CUnion");
            let invite_path = Path::new(INVITE_CUNION_ALICE_PATH);
            let alice_h = send_conn("Alice", invite_path).expect("Should connect to CUNION");


            let claim_h = api_caller::str_u32_str_str_str_r_u32("alice_account",
                                                                ACCOUNT_CERT_SCHEMA_SEQ_NUM,
                                                                ACCOUNT_CERT_DID,
                                                                r#"{"name_on_account":["Alice"], "account_num":["8BEaoLf8TBmK4BUyX8WWnA"]}"#,
                                                                "Account Certificate",
                                                                vcx::api::issuer_claim::vcx_issuer_create_claim).unwrap();

            api_caller::u32_u32_r_u32(claim_h,
                                      alice_h,
                                      vcx::api::issuer_claim::vcx_issuer_send_claim_offer).unwrap();

            await_state(claim_h,
                        3, //VcxStateRequestReceived
                        vcx::api::issuer_claim::vcx_issuer_claim_update_state,
                        None).unwrap();

            api_caller::u32_u32_r_u32(claim_h,
                                      alice_h,
                                      vcx::api::issuer_claim::vcx_issuer_send_claim).unwrap();

            await_state(claim_h,
                        4, //VcxStateAccepted
                        vcx::api::issuer_claim::vcx_issuer_claim_update_state,
                        None).unwrap();

            println!("TIME PASSES AND Alice calls again and must be authenticated.");

            let auth_h = api_caller::str_r_u32("auth_for_alice",
                                       vcx::api::trust_ping::vcx_trust_ping_create
            ).unwrap();

            api_caller::u32_u32_r_u32(auth_h,
                                      alice_h,
                                      vcx::api::trust_ping::vcx_trust_ping_send_request
            ).unwrap();

            println!("Awaiting return of ping from Alice.");
            await_state(auth_h,
                        4, //VcxStateAccepted
                        vcx::api::trust_ping::vcx_trust_ping_update_state,
                        None).unwrap();

            println!("Alice was Authenticated!!!!");

            db_put("alice_for_cunion_h", format!("{}", alice_h));
        },
        _ => () //DOES NOT ACT IN THIS CHAPTER
    }
}

fn chapter_2_demo(actor: &Actor) {
    print_chapter("CHAPTER TWO", None);

    match actor {
        &Actor::Alice => {
            println!("ENTER ALICE");
            let invite_path = Path::new(INVITE_BOB_ALICE_PATH);
            let bob_h = send_conn("Bob", invite_path).expect("Should connect to Bob");
            println!("Connection handle: {}", bob_h);


            println!("Look for proof requests");
            let req = await_message(bob_h,
                                       "PROOF_REQUEST",
                                       vcx::api::disclosed_proof::vcx_disclosed_proof_new_requests,
                                       None).unwrap();

            println!("Requests:\n{}", req);

            let req: Value = serde_json::from_str(&req).unwrap();

            let proof_req = &req[0];

            let proof_h = api_caller::str_str_r_u32("business card",
                                                    &serde_json::to_string(proof_req).unwrap(),
                                                    vcx::api::disclosed_proof::vcx_disclosed_proof_create_with_request).unwrap();

            println!("Proof Handle: {}", proof_h);

            api_caller::u32_u32_r_u32(proof_h,
                                      bob_h,
                                      vcx::api::disclosed_proof::vcx_disclosed_proof_send_proof).unwrap();

            await_state(proof_h,
                        4, //VcxStateAccepted
                        vcx::api::disclosed_proof::vcx_disclosed_proof_update_state,
                        None).unwrap();

            db_put("bob_for_alice_h", format!("{}", bob_h));

        },
        &Actor::Bob => {
            println!("ENTER BOB");
            let invite_path = Path::new(INVITE_BOB_ALICE_PATH);
            let alice_h = receive_conn("Alice", invite_path).expect("Should connect to Alice");

            let requesting_proof = json!([
                {
                  "name":"name",

                  "schema_seq_no": constants::B_CARD_SCHEMA_SEQ_NUM
                },
                {
                  "name":"email",

                  "schema_seq_no": constants::B_CARD_SCHEMA_SEQ_NUM
                },
                {
                  "name":"business",

                  "schema_seq_no": constants::B_CARD_SCHEMA_SEQ_NUM
                },
            ]);
            let requesting_proof = serde_json::to_string_pretty(&requesting_proof).unwrap();
            println!("Requesting Proof:\n{}", requesting_proof);


            let proof_h = api_caller::str_str_str_str_r_u32("proof_of_alice",
                                                            &requesting_proof,
                                                                r#"[]"#,
                                                                "Account Certificate",
                                                                vcx::api::proof::vcx_proof_create).unwrap();
            println!("Proof Handle: {}", proof_h);


            api_caller::u32_u32_r_u32(proof_h,
                                      alice_h,
                                      vcx::api::proof::vcx_proof_send_request).unwrap();

            await_state(proof_h,
                        4, //VcxStateAccepted
                        vcx::api::proof::vcx_proof_update_state,
                        None).unwrap();

            let (proof_state, _attrs) = api_caller::u32_u32_r_u32_str(proof_h,
                                                          alice_h,
                                                          vcx::api::proof::vcx_get_proof).unwrap();

            assert_eq!(1, proof_state);

            db_put("alice_for_bob_h", format!("{}", alice_h));

        },
        _ => () //DOES NOT ACT IN THIS CHAPTER
    }
}

fn chapter_3_demo(actor: &Actor) {
    print_chapter("CHAPTER THREE", None);

    match actor {
        &Actor::Alice => {
            println!("ENTER ALICE");
            let invite_path = Path::new(INVITE_ALICE_DAKOTA_PATH);
            let bob_h = receive_conn("Bob", invite_path).expect("Should connect to Dakota");

            println!("Look for proof requests");
            let req = await_message(bob_h,
                                    "PROOF_REQUEST",
                                    vcx::api::disclosed_proof::vcx_disclosed_proof_new_requests,
                                    None).unwrap();

            println!("Requests:\n{}", req);

            let req: Value = serde_json::from_str(&req).unwrap();

            let proof_req = &req[0];

            let proof_h = api_caller::str_str_r_u32("title",
                                                    &serde_json::to_string(proof_req).unwrap(),
                                                    vcx::api::disclosed_proof::vcx_disclosed_proof_create_with_request).unwrap();

            println!("Proof Handle: {}", proof_h);

            api_caller::u32_u32_r_u32(proof_h,
                                      bob_h,
                                      vcx::api::disclosed_proof::vcx_disclosed_proof_send_proof).unwrap();

            await_state(proof_h,
                        4, //VcxStateAccepted
                        vcx::api::disclosed_proof::vcx_disclosed_proof_update_state,
                        None).unwrap();

        },
        &Actor::Dakota => {
            println!("ENTER DAKOTA");
            let invite_path = Path::new(INVITE_ALICE_DAKOTA_PATH);
            let alice_h = send_conn("Alice", invite_path).expect("Should connect to Alice");

            let requesting_proof = json!([
                {
                  "name":"vin",
                  "issuer_did": constants::V_TITLE_DID,
                  "schema_seq_no": constants::V_TITLE_SCHEMA_SEQ_NUM,
                },
            ]);
            let requesting_proof = serde_json::to_string_pretty(&requesting_proof).unwrap();
            println!("Requesting Proof:\n{}", requesting_proof);


            let proof_h = api_caller::str_str_str_str_r_u32("proof_of_title",
                                                            &requesting_proof,
                                                            r#"[]"#,
                                                            "Account Certificate",
                                                            vcx::api::proof::vcx_proof_create).unwrap();
            println!("Proof Handle: {}", proof_h);


            api_caller::u32_u32_r_u32(proof_h,
                                      alice_h,
                                      vcx::api::proof::vcx_proof_send_request).unwrap();

            await_state(proof_h,
                        4, //VcxStateAccepted
                        vcx::api::proof::vcx_proof_update_state,
                        None).unwrap();

            let (proof_state, attrs) = api_caller::u32_u32_r_u32_str(proof_h,
                                                                     alice_h,
                                                                     vcx::api::proof::vcx_get_proof).unwrap();

            assert_eq!(1, proof_state);
            let vin: Value = serde_json::from_str(&attrs.unwrap()).unwrap();
            let vin = vin[0]["value"].as_str().unwrap();
            assert_eq!(constants::DAKOTAS_VIN, vin);

            println!("UNLOCK CAR!!!!!!!!!!!!!")

        },
        _ => () //DOES NOT ACT IN THIS CHAPTER
    }
}

fn chapter_4_demo(actor: &Actor, dir_path: &Path) {
    print_chapter("CHAPTER FOUR", None);

    match actor {
        &Actor::Alice => {
            println!("ENTER ALICE");

            let bob_h: u32 = db_get("bob_for_alice_h").unwrap().parse().unwrap();
            println!("Connection handle: {}", bob_h);

            println!("Offer Trustee to Bob");


            let recovery_h = api_caller::str_u32_u32_r_u32("bob_trustee",
                                                   10,
                                                   2,
                                                   vcx::api::recovery_shares::vcx_recovery_shares_create).unwrap();

            let trustee_h = api_caller::str_r_u32("bob_trustee",
                                                   vcx::api::offer_trustee::vcx_offer_trustee_create).unwrap();

            api_caller::u32_u32_r_u32(trustee_h,
                                      bob_h,
                                      vcx::api::offer_trustee::vcx_offer_trustee_send_offer).unwrap();

            await_state(trustee_h,
                        3, //VcxStateRequestReceived
                        vcx::api::offer_trustee::vcx_offer_trustee_update_state,
                        None).unwrap();

            api_caller::u32_u32_u32_r_u32(trustee_h,
                                          recovery_h,
                                          bob_h,
                                          vcx::api::offer_trustee::vcx_offer_trustee_send_data).unwrap();

            await_state(trustee_h,
                        4, //VcxStateAccepted
                        vcx::api::offer_trustee::vcx_offer_trustee_update_state,
                        None).unwrap();


            let cunion_h: u32 = db_get("cunion_for_alice_h").unwrap().parse().unwrap();
            println!("Connection handle: {}", cunion_h);

            println!("Offer Trustee to CUnion");


            let trustee_h = api_caller::str_r_u32("cunion_trustee",
                                                  vcx::api::offer_trustee::vcx_offer_trustee_create).unwrap();

            api_caller::u32_u32_r_u32(trustee_h,
                                      cunion_h,
                                      vcx::api::offer_trustee::vcx_offer_trustee_send_offer).unwrap();

            await_state(trustee_h,
                        3, //VcxStateRequestReceived
                        vcx::api::offer_trustee::vcx_offer_trustee_update_state,
                        None).unwrap();

            api_caller::u32_u32_u32_r_u32(trustee_h,
                                          recovery_h,
                                          cunion_h,
                                          vcx::api::offer_trustee::vcx_offer_trustee_send_data).unwrap();

            await_state(trustee_h,
                        4, //VcxStateAccepted
                        vcx::api::offer_trustee::vcx_offer_trustee_update_state,
                        None).unwrap();


            println!("Alice backs up her wallet");

            api_caller::str_r_check(&prep_backup_file(actor, dir_path), vcx::api::backup::vcx_backup_do_backup).unwrap();
        },
        &Actor::Bob => {
            println!("ENTER BOB");

            let alice_h: u32 = db_get("alice_for_bob_h").unwrap().parse().unwrap();
            println!("Connection handle: {}", alice_h);


            println!("Look for trustee offer");
            let offers = await_message(alice_h,
                                       "TRUSTEE_OFFER",
                                       vcx::api::trustee::vcx_trustee_new_offers,
                                       None).unwrap();



            println!("Offers:\n{}", offers);

            let offers: Value = serde_json::from_str(&offers).unwrap();

            let trustee_offer = &offers[0];

            let trustee_h = api_caller::str_str_r_u32("trustee",
                                                    &serde_json::to_string(trustee_offer).unwrap(),
                                                    vcx::api::trustee::vcx_trustee_create_with_offer).unwrap();

            println!("Trustee handle: {}", trustee_h);

            api_caller::u32_u32_r_u32(trustee_h,
                                      alice_h,
                                      vcx::api::trustee::vcx_trustee_send_request).unwrap();



            println!("Look for Trustee Data");
            await_state(trustee_h,
                        4, //VcxStateAccepted
                        vcx::api::trustee::vcx_trustee_update_state,
                        None).unwrap();

            db_put("trustee_handle", format!("{}", trustee_h));

        },
        &Actor::CUnion => {
            println!("ENTER CUNION");

            let alice_h: u32 = db_get("alice_for_cunion_h").unwrap().parse().unwrap();
            println!("Connection handle: {}", alice_h);


            println!("Look for trustee offer");
            let offers = await_message(alice_h,
                                       "TRUSTEE_OFFER",
                                       vcx::api::trustee::vcx_trustee_new_offers,
                                       None).unwrap();



            println!("Offers:\n{}", offers);

            let offers: Value = serde_json::from_str(&offers).unwrap();

            let trustee_offer = &offers[0];

            let trustee_h = api_caller::str_str_r_u32("trustee",
                                                      &serde_json::to_string(trustee_offer).unwrap(),
                                                      vcx::api::trustee::vcx_trustee_create_with_offer).unwrap();

            println!("Trustee handle: {}", trustee_h);

            api_caller::u32_u32_r_u32(trustee_h,
                                      alice_h,
                                      vcx::api::trustee::vcx_trustee_send_request).unwrap();



            println!("Look for Trustee Data");
            await_state(trustee_h,
                        4, //VcxStateAccepted
                        vcx::api::trustee::vcx_trustee_update_state,
                        None).unwrap();

            db_put("trustee_handle", format!("{}", trustee_h));
        },
        _ => () //DOES NOT ACT IN THIS CHAPTER
    }
}

fn chapter_5_demo(actor: &Actor, dir_path: &Path) {
    print_chapter("CHAPTER FIVE", None);

    match actor {
        &Actor::Alice_New => {
            println!("ENTER ALICE's New Agent");

            println!("Recovery Connection with Bob:");
            let invite_path = Path::new(INVITE_RECOVERY_BOB_ALICE_PATH);
            let recovery_bob_h = receive_conn("Bob", invite_path).expect("Should connect to Bob");
            println!("Connection handle: {}", recovery_bob_h);

            println!("Requesting Share from Bob");


            let share_from_bob_h = api_caller::str_r_u32("recovery_share_BOB",
                                                vcx::api::request_share::vcx_request_share_create).unwrap();

            println!("Recovery Share Handle: {}", share_from_bob_h);


            api_caller::u32_u32_r_u32(share_from_bob_h,
                                      recovery_bob_h,
                                      vcx::api::request_share::vcx_request_share_send_request).unwrap();

            await_state(share_from_bob_h,
                        4, //VcxStateAccepted
                        vcx::api::request_share::vcx_request_share_update_state,
                        None).unwrap();

            println!("Recovery Connection with Cunion:");
            let invite_path = Path::new(INVITE_RECOVERY_CUNION_ALICE_PATH);
            let recovery_cunion_h = receive_conn("CUNION", invite_path).expect("Should connect to CUNION");
            println!("Connection handle: {}", recovery_cunion_h);

            println!("Requesting Share from Cunion");


            let share_from_cunion_h = api_caller::str_r_u32("recovery_share_CUNION",
                                                   vcx::api::request_share::vcx_request_share_create).unwrap();

            println!("Recovery Share Handle: {}", share_from_cunion_h);


            api_caller::u32_u32_r_u32(share_from_cunion_h,
                                      recovery_cunion_h,
                                      vcx::api::request_share::vcx_request_share_send_request).unwrap();

            await_state(share_from_cunion_h,
                        4, //VcxStateAccepted
                        vcx::api::request_share::vcx_request_share_update_state,
                        None).unwrap();


            let shares_handles = serde_json::to_string(&vec![share_from_bob_h, share_from_cunion_h]).unwrap();

            api_caller::str_r_check(&shares_handles,
                                    vcx::api::backup::vcx_backup_do_restore).unwrap();
        },
        &Actor::Bob => {
            println!("ENTER BOB");

            let trustee_h: u32 = db_get("trustee_handle").unwrap().parse().unwrap();
            println!("Trustee handle: {}", trustee_h);


            println!("Having been contacted by Alice, Bob revokes Alice's phone");

            api_caller::u32_str_r_u32(trustee_h,
                                          "asdfasdfasdf",
                                          vcx::api::trustee::vcx_trustee_revoke_device).unwrap();


            let invite_path = Path::new(INVITE_RECOVERY_BOB_ALICE_PATH);
            let recovery_alice_h = send_conn("Alice", invite_path).expect("Should connect to Alice");
            println!("Connection handle: {}", recovery_alice_h);


            println!("Look for return share requests");
            let req = await_message(recovery_alice_h,
                                    "REQUEST_SHARE",
                                    vcx::api::return_share::vcx_return_share_new_pings,
                                    None).unwrap();

            println!("Requests:\n{}", req);

            let req: Value = serde_json::from_str(&req).unwrap();

            let proof_req = &req[0];

            let return_share_h = api_caller::str_str_r_u32("alice_share_returned",
                                                    &serde_json::to_string(proof_req).unwrap(),
                                                    vcx::api::return_share::vcx_return_share_create_with_request).unwrap();

            println!("Proof Handle: {}", return_share_h);

            api_caller::u32_u32_u32_r_u32(return_share_h,
                                      recovery_alice_h,
                                      trustee_h,
                                      vcx::api::return_share::vcx_return_share_send_share).unwrap();

            await_state(return_share_h,
                        4, //VcxStateAccepted
                        vcx::api::return_share::vcx_return_share_update_state,
                        None).unwrap();
        },
        &Actor::CUnion => {
            println!("ENTER CUNION");

            let trustee_h: u32 = db_get("trustee_handle").unwrap().parse().unwrap();
            println!("Trustee handle: {}", trustee_h);

            let invite_path = Path::new(INVITE_RECOVERY_CUNION_ALICE_PATH);
            let recovery_alice_h = send_conn("Alice", invite_path).expect("Should connect to Alice");
            println!("Connection handle: {}", recovery_alice_h);


            println!("Look for return share requests");
            let req = await_message(recovery_alice_h,
                                    "REQUEST_SHARE",
                                    vcx::api::return_share::vcx_return_share_new_pings,
                                    None).unwrap();

            println!("Requests:\n{}", req);

            let req: Value = serde_json::from_str(&req).unwrap();

            let proof_req = &req[0];

            let return_share_h = api_caller::str_str_r_u32("alice_share_returned",
                                                           &serde_json::to_string(proof_req).unwrap(),
                                                           vcx::api::return_share::vcx_return_share_create_with_request).unwrap();

            println!("Proof Handle: {}", return_share_h);

            api_caller::u32_u32_u32_r_u32(return_share_h,
                                      recovery_alice_h,
                                      trustee_h,
                                      vcx::api::return_share::vcx_return_share_send_share).unwrap();

            await_state(return_share_h,
                        4, //VcxStateAccepted
                        vcx::api::return_share::vcx_return_share_update_state,
                        None).unwrap();
        },
        _ => () //DOES NOT ACT IN THIS CHAPTER
    }
}

fn init_policy(_agent_key: &str, _recovery_verkey: &str) -> Result<String, u32> {


    Ok(String::from("NOT GENERATED YET"))
}

fn init_pool() {
    let gen_file_path = Path::new(DEFAULT_GENESIS_PATH);
    println!("Writing genesis file at {:?}", gen_file_path);

    let mut gen_file = File::create(gen_file_path).unwrap();
    for line in DEV_GENESIS_NODE_TXNS {
        gen_file.write_all(line.as_bytes()).unwrap();
        gen_file.write_all("\n".as_bytes()).unwrap();
    }
    gen_file.flush().unwrap();
}

fn init_actor(actor: &Actor, dir: &Path) {
//    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
    print_chapter("INIT ACTOR", None);
    println!("Setuping {:?}'s wallet and configuration.", actor);
    wallet::add_wallet_entries(Some(asset_name(actor).as_str()),
                               wallet_entries(actor)
    ).unwrap();
    println!("Wallet Setup is done.");

    let actor_config = config_info(actor);

    let random_int: u32 = rand::random();
    let logo_url = format!("https://robohash.org/{}?set=set3", random_int);
    let wallet_name = asset_name(actor);
    let config = json!(
        {
            "pool_name":"dkms_pool",
            "wallet_name":wallet_name,
            "enterprise_name": format!("{}", actor),
            "logo_url":logo_url,
            "wallet_key": "",

            "agent_endpoint": actor_config.agent_endpoint,
            "agency_pairwise_did": actor_config.agency_pairwise_did,
            "agency_pairwise_verkey": actor_config.agency_pairwise_verkey,
            "enterprise_did_agent": actor_config.enterprise_did_agent,
            "agent_enterprise_verkey": actor_config.agent_enterprise_verkey,
            "enterprise_did": actor_config.enterprise_did,
            "enterprise_verkey": actor_config.enterprise_verkey,
            "agent_pairwise_did": actor_config.agent_pairwise_did,
            "agent_pairwise_verkey": actor_config.agent_pairwise_verkey,
//            "identity_policy_address": config.identity_policy_address,
            "agent_policy_verkey": actor_config.agent_policy_verkey,
            "recovery_verkey": actor_config.recovery_verkey,
        }
    );

    println!("{}'s configuration", actor);
    let config_data = serde_json::to_string_pretty(&config).unwrap();
    println!("{}", config_data);

    let config_file_path = dir.join("config.json");
    let mut config_file = File::create(&config_file_path).unwrap();
    config_file.write_all(config_data.as_bytes()).unwrap();
    config_file.flush().unwrap();
    println!("Config Setup is done.");


    println!("Starting VCX!");
    api_caller::str_r_check(config_file_path.to_str().unwrap(), vcx::api::vcx::vcx_init).unwrap();
    thread::sleep(time::Duration::from_millis(10));


    if let &Actor::Alice = actor {
        let policy_address = init_policy(&actor_config.agent_policy_verkey,
                                         &actor_config.recovery_verkey).unwrap();

        vcx::settings::set_config_value(vcx::settings::CONFIG_IDENTITY_POLICY_ADDRESS, &policy_address);
    }

}

fn full_demo(actor: &Actor) {
    init_pool();

    let dir = tempdir::TempDir::new(&asset_name(actor)).unwrap();
    {
        let dir_path = dir.path();
        println!("Using {:?} for {}", dir_path, actor);

        init_actor(actor, dir_path);
        chapter_1_demo(actor);
        chapter_2_demo(actor);
//        chapter_3_demo(actor);
        chapter_4_demo(actor, dir_path);
        chapter_5_demo(actor, dir_path);

        print_chapter("DONE", None);
    }
    dir.close().unwrap();
}


#[test]
fn dkms_demo(){
    match env::var("DKMS_ACTOR"){
        Ok(_) => {
            let actor = find_actor();
            println!("\nRunning as '{:?}'", actor);
            full_demo(&actor)
        },
        Err(_) => {},
    }
}
