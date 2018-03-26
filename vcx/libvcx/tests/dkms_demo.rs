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
                      ACCOUNT_CERT_DID,
                      B_CARD_SCHEMA_SEQ_NUM,
                      V_TITLE_DID,
                      V_TITLE_SCHEMA_SEQ_NUM,
                      DAKOTAS_VIN};

use dkms::actor::Actor;
use dkms::util::{print_chapter,
                 find_actor,
                 pr_json,
                 send_via_file,
                 receive_via_file,
                 should_print_wait_msg,
                 gate};
use utils::wallet::wallet_file_path;

use vcx::settings::DEFAULT_GENESIS_PATH;
use vcx::utils::libindy;

const INVITE_CUNION_ALICE_PATH: &'static str = "/tmp/cunion-alice-invite.json";
const INVITE_BOB_ALICE_PATH: &'static str = "/tmp/alice-bob-invite.json";
const INVITE_ALICE_DAKOTA_PATH: &'static str = "/tmp/alice-dakota-invite.json";

const INVITE_RECOVERY_BOB_ALICE_PATH: &'static str = "/tmp/recovery-alice-bob-invite.json";
const INVITE_RECOVERY_CUNION_ALICE_PATH: &'static str = "/tmp/recovery-alice-cunion-invite.json";

const POOL_NAME: &'static str = "dkms_pool";

const USE_GATES: bool = false;

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

fn state_name(state: u32) -> &'static str {
    match state {
        0 => "None",
        1 => "Initialized",
        2 => "Sent",
        3 => "Received",
        4 => "Accepted",
        _ => "Unknown"
    }
}

fn await_state(await_for: &str, handle: u32, state: u32, func: api_caller::fn_u32_r_u32_u32, timeout: Option<u64>) -> Result<(), u32> {
    let timeout = timeout.unwrap_or(20 as u64);
    let mut should_print = 0;
    println!("{} waiting for {} state", await_for, state_name(state));
    loop {

        let cur_state = api_caller::u32_r_u32(handle, func)?;
        if cur_state == state {
            break;
        }

        should_print = should_print_wait_msg(format!("{} waiting for state: {} -- currently: {}",await_for, state_name(state), state_name(cur_state)).as_ref(),
                                            should_print,
                                            4);
        thread::sleep(Duration::from_secs(timeout/10))
    }

    println!("{} reached state: {}", await_for, state_name(state));
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
//            println!("received message of type: {}", msg_type);
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

    println!("Connecting to {}", other_party);
    println!("Connection handle: {}", alice_h);

    let invite = api_caller::u32_str_r_u32_str(alice_h,
                                               "",
                                               vcx::api::connection::vcx_connection_connect).map_err(print_err)?;
    let invite = invite.unwrap();

//    println!("Connection Details: \n");
//    pr_json(&invite);

    let gate_msg = format!("Send Connection Invite to {}? [Yes]:", other_party);
    gate(Some(gate_msg.as_str()), USE_GATES);

    println!("Sending invite via file at {:?}", path);
    send_via_file(&invite, &path, None).unwrap();

    await_state("Connection",
                alice_h,
                4, //VcxStateAccepted
                vcx::api::connection::vcx_connection_update_state,
                None).map_err(print_err)?;

    println!("Connection Complete");
    Ok(alice_h)
}

fn receive_conn(other_party: &str, path: &Path) -> Result<u32, u32> {
    let invite = receive_via_file(path, None).unwrap();

    println!("Connecting to {}", other_party);
//    println!("Connection Details: \n");
//    pr_json(&invite);

    let handle = api_caller::str_str_r_u32(other_party,
                                             &invite,
                                             vcx::api::connection::vcx_connection_create_with_invite
        ).map_err(print_err)?;

    println!("Connection handle: {}", handle);

    let gate_msg = format!("Accept Connection Invite from {}? [Yes]:", other_party);
    gate(Some(gate_msg.as_str()), USE_GATES);

    println!("Connecting to connection with {}", other_party);
    let _invite = api_caller::u32_str_r_u32_str(handle,
                                               "",
                                               vcx::api::connection::vcx_connection_connect
    ).map_err(print_err)?;

    await_state("Connection",
                handle,
                4, //VcxStateAccepted
                vcx::api::connection::vcx_connection_update_state,
                None).map_err(print_err)?;

    println!("Connection Complete");
    Ok(handle)
}

fn prep_backup_file(actor: &Actor, actor_dir: &Path) -> String {
    let mut rtn: Vec<PathBuf> = Vec::new();
    db_write_file(actor_dir);
    rtn.push(actor_dir.join("DB"));
    rtn.push(actor_dir.join("config.json"));
    rtn.push(wallet_file_path(&asset_name(actor)).unwrap());

    serde_json::to_string(&rtn).unwrap()
}

fn chapter_1_demo(actor: &Actor) {
    match actor {
        &Actor::Alice => {
            print_chapter("CHAPTER ONE", None);
            println!("ENTER ALICE");
            gate(None, USE_GATES);

            let invite_path = Path::new(INVITE_CUNION_ALICE_PATH);
            let cunion_h = receive_conn("CUNION", invite_path).expect("Should connect to CUNION");


            println!("Alice looks for claim offers from CUnion");
            let offers = await_message(cunion_h,
                                       "CLAIM_OFFER",
                                       vcx::api::claim::vcx_claim_new_offers,
                                       None).unwrap();



            println!("Received offers:\n{}", offers);

            let offers: Value = serde_json::from_str(&offers).unwrap();

            let account_cert_offer = &offers[0];
            let account_cert_offer = serde_json::to_string(account_cert_offer).unwrap();

            let claim_h = api_caller::str_str_r_u32("bank account",
                                                    &account_cert_offer,
                                                    vcx::api::claim::vcx_claim_create_with_offer).unwrap();
            println!("Claim handle: {}", claim_h);

            gate(Some("Accept Claim from CUnion? [Yes]:"), USE_GATES);

            api_caller::u32_u32_r_u32(claim_h,
                                      cunion_h,
                                      vcx::api::claim::vcx_claim_send_request).unwrap();

            println!("Waiting for claim from CUnion");
            await_state("Claim",
                        claim_h,
                        4, //VcxStateAccepted
                        vcx::api::claim::vcx_claim_update_state,
                        None).unwrap();

            println!("Time passes and Alice calls again and must be authenticated.");
            println!("Alice Looks for DID auth request");
            let pings = await_message(cunion_h,
                                       "TRUST_PING",
                                       vcx::api::trust_pong::vcx_trust_pong_new_pings,
                                       None).unwrap();

            println!("Did Auth requests:\n{}", pings);

            let pings: Value = serde_json::from_str(&pings).unwrap();

            let ping = &pings[0];
            let ping = serde_json::to_string(ping).unwrap();

            let pong_h = api_caller::str_str_r_u32("auth req from cunion",
                                                    &ping,
                                                    vcx::api::trust_pong::vcx_trust_pong_create_with_request).unwrap();

            println!("DID Auth Handle: {}", pong_h);

            gate(Some("Reply to DID Auth request from CUnion? [Yes]:"), USE_GATES);

            api_caller::u32_u32_r_u32(pong_h,
                                      cunion_h,
                                      vcx::api::trust_pong::vcx_trust_pong_send).unwrap();

            await_state("DID Auth",
                        pong_h,
                        4, //VcxStateAccepted
                        vcx::api::trust_pong::vcx_trust_pong_update_state,
                        None).unwrap();

            println!("DID auth challange met!");

            db_put("cunion_for_alice_h", format!("{}", cunion_h)).unwrap();

        },
        &Actor::CUnion => {
            print_chapter("CHAPTER ONE", None);
            println!("ENTER CUnion");
            gate(None, USE_GATES);

            let invite_path = Path::new(INVITE_CUNION_ALICE_PATH);
            let alice_h = send_conn("Alice", invite_path).expect("Should connect to CUNION");

            let claim_data = r#"{"name_on_account":["Alice"], "account_num":["8BEaoLf8TBmK4BUyX8WWnA"]}"#;

            println!("Sending Account Certificate offer to Alice:");
            pr_json(claim_data);

            let claim_h = api_caller::str_u32_str_str_str_r_u32("alice_account",
                                                                ACCOUNT_CERT_SCHEMA_SEQ_NUM,
                                                                ACCOUNT_CERT_DID,
                                                                claim_data,
                                                                "Account Certificate",
                                                                vcx::api::issuer_claim::vcx_issuer_create_claim).unwrap();

            api_caller::u32_u32_r_u32(claim_h,
                                      alice_h,
                                      vcx::api::issuer_claim::vcx_issuer_send_claim_offer).unwrap();

            await_state("Claim",
                        claim_h,
                        3, //VcxStateRequestReceived
                        vcx::api::issuer_claim::vcx_issuer_claim_update_state,
                        None).unwrap();

            println!("Alice has replyed to Claim Request");

            gate(Some("Send signed claim to Alice? [Yes]:"), USE_GATES);
            api_caller::u32_u32_r_u32(claim_h,
                                      alice_h,
                                      vcx::api::issuer_claim::vcx_issuer_send_claim).unwrap();

            await_state("Claim",
                        claim_h,
                        4, //VcxStateAccepted
                        vcx::api::issuer_claim::vcx_issuer_claim_update_state,
                        None).unwrap();

            println!("Claim issuance is complete!");
            println!();


            println!("TIME PASSES AND Alice calls again and must be authenticated.");

            gate(Some("Send DID Auth request to Alice? [Yes]:"), USE_GATES);

            let auth_h = api_caller::str_r_u32("auth_for_alice",
                                       vcx::api::trust_ping::vcx_trust_ping_create
            ).unwrap();

            api_caller::u32_u32_r_u32(auth_h,
                                      alice_h,
                                      vcx::api::trust_ping::vcx_trust_ping_send_request
            ).unwrap();

            println!("Awaiting return of ping from Alice.");
            await_state("DID Auth",
                        auth_h,
                        4, //VcxStateAccepted
                        vcx::api::trust_ping::vcx_trust_ping_update_state,
                        None).unwrap();

            println!("Alice was Authenticated!!!!");

            db_put("alice_for_cunion_h", format!("{}", alice_h)).unwrap();
        },
        _ => () //DOES NOT ACT IN THIS CHAPTER
    }
}

fn chapter_2_demo(actor: &Actor) {
    match actor {
        &Actor::Alice => {
            print_chapter("CHAPTER TWO", None);
            println!("ENTER ALICE");
            gate(None, USE_GATES);

            let invite_path = Path::new(INVITE_BOB_ALICE_PATH);
            let bob_h = send_conn("Bob", invite_path).expect("Should connect to Bob");
            println!("Connection handle: {}", bob_h);


            println!("Alice agrees to provide business card info to Bob");
            println!("Alice Looks for proof requests from Bob");
            let req = await_message(bob_h,
                                       "PROOF_REQUEST",
                                       vcx::api::disclosed_proof::vcx_disclosed_proof_new_requests,
                                       None).unwrap();

            println!("Proof requests:\n{}", req);

            let req: Value = serde_json::from_str(&req).unwrap();

            let proof_req = &req[0];

            let proof_h = api_caller::str_str_r_u32("business card",
                                                    &serde_json::to_string(proof_req).unwrap(),
                                                    vcx::api::disclosed_proof::vcx_disclosed_proof_create_with_request).unwrap();

            println!("Proof Handle: {}", proof_h);


            gate(Some("Send business card proof to Bob? [Yes]:"), USE_GATES);
            api_caller::u32_u32_r_u32(proof_h,
                                      bob_h,
                                      vcx::api::disclosed_proof::vcx_disclosed_proof_send_proof).unwrap();

            println!("Proof sent successfully");


            await_state("Proof",
                        proof_h,
                        4, //VcxStateAccepted
                        vcx::api::disclosed_proof::vcx_disclosed_proof_update_state,
                        None).unwrap();

            println!("Proof successfully sent!");

            db_put("bob_for_alice_h", format!("{}", bob_h)).unwrap();

        },
        &Actor::Bob => {
            print_chapter("CHAPTER TWO", None);
            println!("ENTER BOB");
            gate(None, USE_GATES);

            let invite_path = Path::new(INVITE_BOB_ALICE_PATH);
            let alice_h = receive_conn("Alice", invite_path).expect("Should connect to Alice");

            let requesting_proof = json!([
                {
                  "name":"name",

                  "schema_seq_no": B_CARD_SCHEMA_SEQ_NUM
                },
                {
                  "name":"email",

                  "schema_seq_no": B_CARD_SCHEMA_SEQ_NUM
                },
                {
                  "name":"business",

                  "schema_seq_no": B_CARD_SCHEMA_SEQ_NUM
                },
            ]);
            let requesting_proof = serde_json::to_string_pretty(&requesting_proof).unwrap();
            println!("Request for business card proof:\n{}", requesting_proof);

            let proof_h = api_caller::str_str_str_str_r_u32("proof_of_alice",
                                                            &requesting_proof,
                                                                r#"[]"#,
                                                                "Account Certificate",
                                                                vcx::api::proof::vcx_proof_create).unwrap();
            println!("Proof Handle: {}", proof_h);

            gate(Some("Send request for business card proof to Alice? [Yes]:"), USE_GATES);
            api_caller::u32_u32_r_u32(proof_h,
                                      alice_h,
                                      vcx::api::proof::vcx_proof_send_request).unwrap();

            await_state("Proof",
                        proof_h,
                        4, //VcxStateAccepted
                        vcx::api::proof::vcx_proof_update_state,
                        None).unwrap();

            let (proof_state, _attrs) = api_caller::u32_u32_r_u32_str(proof_h,
                                                          alice_h,
                                                          vcx::api::proof::vcx_get_proof).unwrap();

            assert_eq!(1, proof_state);

            pr_json(&_attrs.expect("Expect proof attrs"));

            println!("Bob has received proof from Alice!");

            db_put("alice_for_bob_h", format!("{}", alice_h)).unwrap();

        },
        _ => () //DOES NOT ACT IN THIS CHAPTER
    }
}

fn chapter_3_demo(actor: &Actor) {
    match actor {
        &Actor::Alice => {
            print_chapter("CHAPTER THREE", None);
            println!("ENTER ALICE");
            gate(None, USE_GATES);

            let invite_path = Path::new(INVITE_ALICE_DAKOTA_PATH);
            let dakota_h = receive_conn("Dakota", invite_path).expect("Should connect to Dakota");

            println!("Alice wants to prove to her car that she owns it.");
            println!("Alice Looks for proof requests from Dakota");
            let req = await_message(dakota_h,
                                    "PROOF_REQUEST",
                                    vcx::api::disclosed_proof::vcx_disclosed_proof_new_requests,
                                    None).unwrap();

            println!("Proof Requests:\n{}", req);

            let req: Value = serde_json::from_str(&req).unwrap();

            let proof_req = &req[0];

            let proof_h = api_caller::str_str_r_u32("title",
                                                    &serde_json::to_string(proof_req).unwrap(),
                                                    vcx::api::disclosed_proof::vcx_disclosed_proof_create_with_request).unwrap();

            println!("Proof Handle: {}", proof_h);

            gate(Some("Send Auto Title proof to Dakota? [Yes]:"), USE_GATES);
            api_caller::u32_u32_r_u32(proof_h,
                                      dakota_h,
                                      vcx::api::disclosed_proof::vcx_disclosed_proof_send_proof).unwrap();

            await_state("Proof",
                        proof_h,
                        4, //VcxStateAccepted
                        vcx::api::disclosed_proof::vcx_disclosed_proof_update_state,
                        None).unwrap();

            println!("Proof successfully sent!");
        },
        &Actor::Dakota => {
            print_chapter("CHAPTER THREE", None);
            println!("ENTER DAKOTA");
            gate(None, USE_GATES);

            let invite_path = Path::new(INVITE_ALICE_DAKOTA_PATH);
            let alice_h = send_conn("Alice", invite_path).expect("Should connect to Alice");

            println!("Alice wants to unlock me but Dakota wants proof frist.");
            let requesting_proof = json!([
                {
                  "name":"vin",
                  "issuer_did": V_TITLE_DID,
                  "schema_seq_no": V_TITLE_SCHEMA_SEQ_NUM,
                },
            ]);
            let requesting_proof = serde_json::to_string_pretty(&requesting_proof).unwrap();
            println!("Requesting Proof:\n{}", requesting_proof);


            let proof_h = api_caller::str_str_str_str_r_u32("proof_of_title",
                                                            &requesting_proof,
                                                            r#"[]"#,
                                                            "Auto Title",
                                                            vcx::api::proof::vcx_proof_create).unwrap();
            println!("Proof Handle: {}", proof_h);

            gate(Some("Send request for Auto Title proof to Alice? [Yes]:"), USE_GATES);
            api_caller::u32_u32_r_u32(proof_h,
                                      alice_h,
                                      vcx::api::proof::vcx_proof_send_request).unwrap();

            await_state("Proof",
                        proof_h,
                        4, //VcxStateAccepted
                        vcx::api::proof::vcx_proof_update_state,
                        None).unwrap();

            let (proof_state, attrs) = api_caller::u32_u32_r_u32_str(proof_h,
                                                                     alice_h,
                                                                     vcx::api::proof::vcx_get_proof).unwrap();

            assert_eq!(1, proof_state);
            let vin: Value = serde_json::from_str(&attrs.unwrap()).unwrap();
            let vin = vin[0]["value"].as_str().unwrap();
            assert_eq!(DAKOTAS_VIN, vin);

            println!("VIN in proof matches Dakota's VIN");
            println!("UNLOCK CAR!")

        },
        _ => () //DOES NOT ACT IN THIS CHAPTER
    }
}

fn _offer_trustee(other_party: &str, c_h: u32, r_h: u32) -> Result<u32, u32> {

    let trustee_h = api_caller::str_r_u32(&format!("{}_trustee", other_party),
                                          vcx::api::offer_trustee::vcx_offer_trustee_create).unwrap();
    println!("Trustee Handle: {}", trustee_h);

    api_caller::u32_u32_r_u32(trustee_h,
                              c_h,
                              vcx::api::offer_trustee::vcx_offer_trustee_send_offer).unwrap();

    println!("Sending trustee offer to {}", other_party);
    await_state("Trustee",
                trustee_h,
                3, //VcxStateRequestReceived
                vcx::api::offer_trustee::vcx_offer_trustee_update_state,
                None).unwrap();
    println!("Received trustee accept request from {}", other_party);

    let gate_msg = format!("Send Trustee Data to {}? [Yes]:", other_party);
    gate(Some(gate_msg.as_str()), USE_GATES);
    api_caller::u32_u32_u32_r_u32(trustee_h,
                                  r_h,
                                  c_h,
                                  vcx::api::offer_trustee::vcx_offer_trustee_send_data).unwrap();



    await_state("Trustee",
                trustee_h,
                4, //VcxStateAccepted
                vcx::api::offer_trustee::vcx_offer_trustee_update_state,
                None).unwrap();

    Ok(trustee_h)
}

fn _accept_trustee(other_party: &str, c_h: u32) -> Result<u32, u32> {
    println!("Looking for trustee offers from {}", other_party);
    let offers = await_message(c_h,
                               "TRUSTEE_OFFER",
                               vcx::api::trustee::vcx_trustee_new_offers,
                               None).unwrap();



    println!("Offers:\n{}", offers);

    let offers: Value = serde_json::from_str(&offers).unwrap();

    let trustee_offer = &offers[0];
    let trustee_offer = serde_json::to_string(trustee_offer).unwrap();

    let trustee_h = api_caller::str_str_r_u32("trustee",
                                              &trustee_offer,
                                              vcx::api::trustee::vcx_trustee_create_with_offer
    ).unwrap();

    println!("Trustee handle: {}", trustee_h);

    let gate_msg = format!("Accept Trustee offer from {}? [Yes]:", other_party);
    gate(Some(gate_msg.as_str()), USE_GATES);
    api_caller::u32_u32_r_u32(trustee_h,
                              c_h,
                              vcx::api::trustee::vcx_trustee_send_request).unwrap();



    await_state("Trustee Data",
                trustee_h,
                4, //VcxStateAccepted
                vcx::api::trustee::vcx_trustee_update_state,
                None).unwrap();

    Ok(trustee_h)
}

fn chapter_4_demo(actor: &Actor, dir_path: &Path) {
    match actor {
        &Actor::Alice => {
            print_chapter("CHAPTER FOUR", None);
            println!("ENTER ALICE");
            gate(None, USE_GATES);

            println!("Alice wants to protect her Digital Identity.");

            println!("Alice shards her recovery key into multiple shares.");
            let recovery_h = api_caller::str_u32_u32_r_u32("recovery_shares",
                                                           10,
                                                           2,
                                                           vcx::api::recovery_shares::vcx_recovery_shares_create
            ).unwrap();

            let bob_h: u32 = db_get("bob_for_alice_h").unwrap().parse().unwrap();
//            println!("Connection handle: {}", bob_h);

            println!("Alice chooses Bob to be Trustee.");
            let _trustee_h_bob = _offer_trustee("Bob",
                                                bob_h,
                                                recovery_h
            ).unwrap();


            let cunion_h: u32 = db_get("cunion_for_alice_h").unwrap().parse().unwrap();
//            println!("Connection handle: {}", cunion_h);

            println!("Alice chooses CUnion to be Trustee.");
            let _trustee_h_cunion = _offer_trustee("CUnion",
                                           cunion_h,
                                           recovery_h
            ).unwrap();


            let _offers = api_caller::u32_r_u32_str(bob_h,
                                                   vcx::api::trustee::vcx_trustee_new_offers
            ).unwrap(); //TESTING FOR A BUG


            println!("Alice creates an encrypted backup of her identity data (wallet and other data).");

            api_caller::str_r_check(&prep_backup_file(actor, dir_path), vcx::api::backup::vcx_backup_do_backup).unwrap();

            println!("Alice digital identity is secure!");
        },
        &Actor::Bob => {
            print_chapter("CHAPTER FOUR", None);
            println!("ENTER BOB");
            gate(None, USE_GATES);

            let alice_h: u32 = db_get("alice_for_bob_h").unwrap().parse().unwrap();
//            println!("Connection handle: {}", alice_h);

            let trustee_h = _accept_trustee("Alice", alice_h).unwrap();

            db_put("trustee_handle", format!("{}", trustee_h)).unwrap();

            println!("Bob is now Alice's Trustee!");

        },
        &Actor::CUnion => {
            print_chapter("CHAPTER FOUR", None);
            println!("ENTER CUNION");
            gate(None, USE_GATES);

            let alice_h: u32 = db_get("alice_for_cunion_h").unwrap().parse().unwrap();
//            println!("Connection handle: {}", alice_h);

            let trustee_h = _accept_trustee("Alice", alice_h).unwrap();

            db_put("trustee_handle", format!("{}", trustee_h)).unwrap();

            println!("CUnion is now Alice's Trustee!");
        },
        _ => () //DOES NOT ACT IN THIS CHAPTER
    }
}

fn _request_share(other_party: &str, c_h: u32) -> Result<u32, u32> {
    println!("Requesting Share from {}", other_party);


    let share_id = format!("recovery_share_{}", other_party);
    let share_h = api_caller::str_r_u32(&share_id,
                                        vcx::api::request_share::vcx_request_share_create
    ).unwrap();

    println!("Recovery Share Handle: {}", share_h);

    let gate_msg = format!("Send request for share to {}? [Yes]:", other_party);
    gate(Some(gate_msg.as_str()), USE_GATES);
    api_caller::u32_u32_r_u32(share_h,
                              c_h,
                              vcx::api::request_share::vcx_request_share_send_request
    ).unwrap();

    await_state("Return Share",
                share_h,
                4, //VcxStateAccepted
                vcx::api::request_share::vcx_request_share_update_state,
                None).unwrap();

    println!("Share from {} has been returned.", other_party);

    Ok(share_h)
}

fn _return_share(other_party:&str, c_h: u32, trustee_h: u32) -> Result<u32, u32> {
    println!("Looking for request to return share from {}", other_party);
    let req = await_message(c_h,
                            "REQUEST_SHARE",
                            vcx::api::return_share::vcx_return_share_new_request,
                            None
    ).unwrap();

    println!("Requests:\n{}", req);

    let req: Value = serde_json::from_str(&req).unwrap();

    let req = &req[0];
    let req = serde_json::to_string(req).unwrap();

    let id = format!("{}_share_returned", other_party);
    let return_share_h = api_caller::str_str_r_u32(&id,
                                                   &req,
                                                   vcx::api::return_share::vcx_return_share_create_with_request
    ).unwrap();

    println!("Handle: {}", return_share_h);

    let gate_msg = format!("Send share back to {}? [Yes]:", other_party);
    gate(Some(gate_msg.as_str()), USE_GATES);
    api_caller::u32_u32_u32_r_u32(return_share_h,
                                  c_h,
                                  trustee_h,
                                  vcx::api::return_share::vcx_return_share_send_share
    ).unwrap();

    await_state("Share Request",
                return_share_h,
                4, //VcxStateAccepted
                vcx::api::return_share::vcx_return_share_update_state,
                None).unwrap();

    println!("Share from {} has been returned.", other_party);

    Ok(return_share_h)
}

fn chapter_5_demo(actor: &Actor, _dir_path: &Path) {
    match actor {
        &Actor::AliceNew => {
            print_chapter("CHAPTER FIVE", None);
            println!("ENTER ALICE's New Agent");
            gate(None, USE_GATES);

            println!("Recovery Connection with Bob:");
            let invite_path = Path::new(INVITE_RECOVERY_BOB_ALICE_PATH);
            let recovery_bob_h = receive_conn("Bob", invite_path).expect("Should connect to Bob");
            println!("Connection handle: {}", recovery_bob_h);

            let share_from_bob_h = _request_share("Bob", recovery_bob_h).unwrap();

            println!("Recovery Connection with Cunion:");
            let invite_path = Path::new(INVITE_RECOVERY_CUNION_ALICE_PATH);
            let recovery_cunion_h = receive_conn("CUNION", invite_path).expect("Should connect to CUNION");
            println!("Connection handle: {}", recovery_cunion_h);

            let share_from_cunion_h = _request_share("CUnion", recovery_cunion_h).unwrap();

            let shares_handles = serde_json::to_string(&vec![share_from_bob_h, share_from_cunion_h]).unwrap();

            println!("Alice has her shares back.");
            println!("Alice can now restore her Agent");

            api_caller::str_r_check(&shares_handles,
                                    vcx::api::backup::vcx_backup_do_restore).unwrap();

            println!("Alice has restored her Agent!");
        }
        &Actor::Bob => {
            print_chapter("CHAPTER FIVE", None);
            println!("ENTER BOB");
            gate(None, USE_GATES);

            let trustee_h: u32 = db_get("trustee_handle").unwrap().parse().unwrap();
            println!("Trustee handle: {}", trustee_h);


            println!("Having been contacted by Alice, Bob revokes Alice's phone");

            let agent_list = api_caller::u32_r_u32_str(trustee_h,
                                      vcx::api::trustee::vcx_trustee_list_agents).unwrap().unwrap();

            println!("Bob has the following agent that he can revoke");
            println!("{}", agent_list);

            let gate_msg = format!("Revoke Alice's 'Phone'? [Yes]:");
            gate(Some(gate_msg.as_str()), USE_GATES);

            let agent_verkey: Value = serde_json::from_str(&agent_list).unwrap();
            let agent_verkey = agent_verkey[0]["verkey"].as_str().unwrap();

            api_caller::u32_str_r_u32(trustee_h,
                                      agent_verkey,
                                          vcx::api::trustee::vcx_trustee_revoke_device).unwrap();

            println!("While in contact with Alice, Bob creates a recovery connection.");

            let invite_path = Path::new(INVITE_RECOVERY_BOB_ALICE_PATH);
            let recovery_alice_h = send_conn("Alice", invite_path).expect("Should connect to Alice");
            println!("Connection handle: {}", recovery_alice_h);


            let _return_share_h = _return_share("Alice",
                                               recovery_alice_h,
                                               trustee_h
            ).unwrap();

            println!("Bob has finished helping Alice recover!");

        },
        &Actor::CUnion => {
            print_chapter("CHAPTER FIVE", None);
            println!("ENTER CUNION");
            gate(None, USE_GATES);

            let trustee_h: u32 = db_get("trustee_handle").unwrap().parse().unwrap();
            println!("Trustee handle: {}", trustee_h);


            println!("Alice visits a CUnion branch, CUnion creates a recovery connection.");

            let invite_path = Path::new(INVITE_RECOVERY_CUNION_ALICE_PATH);
            let recovery_alice_h = send_conn("Alice", invite_path).expect("Should connect to Alice");
            println!("Connection handle: {}", recovery_alice_h);


            let _return_share_h = _return_share("Alice",
                                                                recovery_alice_h,
                                                                trustee_h
            ).unwrap();

            println!("CUnion has finished helping Alice recover!");
        },
        &Actor::Alice => {
            print_chapter("CHAPTER FIVE", None);
            println!("ENTER Fake Alice");


            println!("Alice's phone has been stolen!!");
            println!("This agent is no longer acting on Alice's behalf!");
        }
        _ => () //DOES NOT ACT IN THIS CHAPTER
    }
}


fn _get_current_policy(w_h: i32, p_h: i32, verkey: &str, address: &str) -> Result<Value, u32> {
    let get_policy_txn = libindy::ledger::libindy_build_get_agent_authz_request(
        &verkey,
        address
    ).expect("Building txn for current policy");

    let result = libindy::ledger::libindy_sign_and_submit_request(p_h,
                                                                  w_h,
                                                                  verkey.to_owned(),
                                                                  get_policy_txn
    ).expect("submitting txn to ledger for current policy");

    let result: Value = serde_json::from_str(&result).or(Err(88 as u32))
        .expect("Result from ledger(current policy) to be valid json");
    Ok(result)
}


fn refresh_policy(w_h: i32, p_h: i32, verkey: &str, address: &str) -> Result<(), u32> {


    let policy: Value = _get_current_policy(w_h, p_h, verkey, address)?;

    println!("\nCleaning Policy for demo");
//    println!("Dirty Entries:");

    let data = policy["result"]["data"].as_array().unwrap();

    for auth in data {
        let agent_verkey = auth[0].as_str().ok_or(88 as u32)?;
        let auth_level = auth[1].as_u64().ok_or(88 as u32)?;
        if auth_level == 0 {
            continue;
        }
        if agent_verkey.eq(verkey) {
            continue;
        }

//        println!("{}", serde_json::to_string_pretty(&auth).unwrap());

        let update_txn = libindy::ledger::libindy_build_agent_authz_request(
            verkey,
            address,
            agent_verkey,
            0,
            None
        ).unwrap();

//        println!("{}", update_txn);

        let _result = libindy::ledger::libindy_sign_and_submit_request(p_h,
                                                                      w_h,
                                                                      verkey.to_owned(),
                                                                      update_txn).unwrap();
//        println!("Result from ledger: \n{}", _result);
    }


//    let policy: Value = _get_current_policy(w_h, p_h, verkey, address).unwrap();
//    println!("Clean Policy: \n{}", serde_json::to_string_pretty(&policy).unwrap());

    println!();
    Ok(())

}

fn init_policy(address: &str) -> Result<(), u32> {
    let wallet_name = "CLEAN_POLICY";

    let p_h = vcx::utils::libindy::pool::libindy_open_pool_ledger(POOL_NAME, None).unwrap();
    match vcx::utils::libindy::wallet::create_wallet(POOL_NAME,
                                                     wallet_name,
                                                     None,
                                                     None,
                                                     None) {
        Ok(_) => (),
        Err(_) => {
            vcx::utils::libindy::wallet::delete_wallet(wallet_name).unwrap();
            vcx::utils::libindy::wallet::create_wallet(POOL_NAME,
                                                       wallet_name,
                                                       None,
                                                       None,
                                                       None
            ).unwrap();
        }
    }
    let w_h = vcx::utils::libindy::wallet::open_wallet(wallet_name, None, None).unwrap();

    let recovery_vk = vcx::utils::libindy::crypto::libindy_create_key(w_h,
                                                    r#"{"seed": "4F7BsTMVPKFshM1MwLf6y23cid6fL3xMpfDaTHZtNLTr"}"#
    ).unwrap();


    refresh_policy(w_h, p_h, &recovery_vk, address)?;

    vcx::utils::libindy::wallet::close_wallet(w_h).unwrap();
    vcx::utils::libindy::wallet::delete_wallet(wallet_name).unwrap();
    vcx::utils::libindy::pool::libindy_close_pool_ledger(p_h).unwrap();

    Ok(())
}

fn init_pool() {
    println!("Setup ledger connection");
    let gen_file_path = Path::new(DEFAULT_GENESIS_PATH);
    println!("Writing genesis file at {:?}", gen_file_path);

    let mut gen_file = File::create(gen_file_path).unwrap();
    for line in DEV_GENESIS_NODE_TXNS {
        gen_file.write_all(line.as_bytes()).unwrap();
        gen_file.write_all("\n".as_bytes()).unwrap();
    }
    gen_file.flush().unwrap();
    println!("Complete ledger setup");
    println!();
}

fn init_actor(actor: &Actor, dir: &Path) {
    print_chapter("INIT ACTOR", None);
    println!("Setting up {:?}'s wallet and configuration.", actor);
    wallet::add_wallet_entries(Some(asset_name(actor).as_str()),
                               Some(POOL_NAME),
                               wallet_entries(actor)
    ).unwrap();
    println!("Wallet Setup is done.");

    let actor_config = config_info(actor);

    let random_int: u32 = rand::random();
    let logo_url = format!("https://robohash.org/{}?set=set3", random_int);
    let wallet_name = asset_name(actor);

    if let &Actor::Alice = actor {
        init_policy(&actor_config.identity_policy_address).unwrap();
    }

    let config = json!(
        {
            "pool_name": POOL_NAME,
            "wallet_name": wallet_name,
            "enterprise_name": format!("{}", actor),
            "logo_url": logo_url,
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
            "identity_policy_address": actor_config.identity_policy_address,
//            "agent_policy_verkey": actor_config.agent_policy_verkey,
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

    println!("Complete Setup!");
    println!();

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
        chapter_3_demo(actor);
        chapter_4_demo(actor, dir_path);
        chapter_5_demo(actor, dir_path);

        print_chapter("DONE", None);
        gate(None, USE_GATES);
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