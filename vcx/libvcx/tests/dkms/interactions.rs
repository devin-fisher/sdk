//use std::env;
//use std::fs::File;
//use std::io::Write;
//use std::thread;
//use std::time;
//use std::path::Path;
//use std::sync::Mutex;
//use std::collections::HashMap;
//use std::fmt::Debug;
//use std::time::Duration;
//use std::ops::Deref;
//use std::path::PathBuf;
//
//use serde_json::Value;
//
//
//mod utils;
//use utils::wallet;
//use utils::api_caller;
//
//use dkms::constants::{DEV_GENESIS_NODE_TXNS,
//                      config_info,
//                      wallet_entries,
//                      asset_name,
//                      ACCOUNT_CERT_SCHEMA_SEQ_NUM,
//                      ACCOUNT_CERT_DID};
//use dkms::constants;
//use dkms::actor::Actor;
//use dkms::util::{print_chapter,
//                 find_actor,
//                 pr_json,
//                 send_via_file,
//                 receive_via_file,
//                 should_print_wait_msg,
//                 gate};
//use utils::wallet::wallet_file_path;
//
//use vcx::settings::DEFAULT_GENESIS_PATH;
//use vcx::utils::libindy;
//
//fn await_state(handle: u32, state: u32, func: api_caller::fn_u32_r_u32_u32, timeout: Option<u64>) -> Result<(), u32> {
//    let timeout = timeout.unwrap_or(20 as u64);
//    let mut should_print = 0;
//    loop {
//
//        let cur_state = api_caller::u32_r_u32(handle, func)?;
//        if cur_state == state {
//            break;
//        }
//
//        should_print = should_print_wait_msg(format!("waiting for state: {} -- currently: {}", state, cur_state).as_ref(),
//                                             should_print,
//                                             4);
//        thread::sleep(Duration::from_secs(timeout/10))
//    }
//
//    println!("reached state: {}", state);
//    Ok(())
//}
//
//fn await_message(conn_handle: u32, msg_type: &str, func: api_caller::fn_u32_r_u32_str, timeout: Option<u64>) -> Result<String, u32> {
//    let timeout = timeout.unwrap_or(20 as u64);
//    let mut should_print = 0;
//    loop {
//
//        let msg = api_caller::u32_r_u32_str(conn_handle, func).unwrap();
//        let msg = msg.unwrap_or(String::from("[]"));
//
////        println!("{}", msg);
//
//        if msg.contains(msg_type) {
//            println!("received message of type: {}", msg_type);
//            return Ok(msg);
//        }
//
//        should_print = should_print_wait_msg(format!("waiting for messages with type: {}", msg_type).as_ref(),
//                                             should_print,
//                                             4);
//        thread::sleep(Duration::from_secs(timeout/10))
//    }
//}
//
//fn send_conn(other_party: &str, path: &Path) -> Result<u32, u32> {
//    let alice_h = api_caller::str_r_u32(other_party,
//                                        vcx::api::connection::vcx_connection_create
//    ).map_err(print_err)?;
//
//    println!("Connection handle: {}", alice_h);
//
//    let invite = api_caller::u32_str_r_u32_str(alice_h,
//                                               "",
//                                               vcx::api::connection::vcx_connection_connect).map_err(print_err)?;
//    let invite = invite.unwrap();
//
//    println!("Connection Details: \n");
//    pr_json(&invite);
//
//
//    println!("Sending invite via file at {:?}", path);
//    send_via_file(&invite, &path, None).unwrap();
//
//    await_state(alice_h,
//                4, //VcxStateAccepted
//                vcx::api::connection::vcx_connection_update_state,
//                None).map_err(print_err)?;
//
//    Ok(alice_h)
//}
//
//fn receive_conn(other_party: &str, path: &Path) -> Result<u32, u32> {
//    let invite = receive_via_file(path, None).unwrap();
//
//    println!("Connection Details: \n");
//    pr_json(&invite);
//
//    println!("Creating connection with {}", other_party);
//    let handle = api_caller::str_str_r_u32(other_party,
//                                           &invite,
//                                           vcx::api::connection::vcx_connection_create_with_invite
//    ).map_err(print_err)?;
//
//    println!("Connection handle: {}", handle);
//
//    println!("Connecting to connection with {}", other_party);
//    let _invite = api_caller::u32_str_r_u32_str(handle,
//                                                "",
//                                                vcx::api::connection::vcx_connection_connect
//    ).map_err(print_err)?;
//
//    await_state(handle,
//                4, //VcxStateAccepted
//                vcx::api::connection::vcx_connection_update_state,
//                None).map_err(print_err)?;
//
//    Ok(handle)
//}