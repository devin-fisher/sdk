extern crate libc;
extern crate serde_json;

use self::libc::c_char;
use std::ffi::CString;
use utils::libindy::{indy_function_eval, check_str};
use utils::libindy::return_types::{ Return_I32_STR };
use utils::libindy::error_codes::{map_indy_error_code, map_string_error};
use utils::timeout::TimeoutUtils;
use utils::libindy::ledger;

use serde_json::Value;

use utils::error::INVALID_JSON;


extern {
    fn indy_create_and_store_new_policy(command_handle: i32,
                                        wallet_handle: i32,
                                        cb: Option<extern fn(xcommand_handle: i32,
                                                             err: i32,
                                                             policy_address: *const c_char)>
    ) -> i32;

    fn indy_add_new_agent_to_policy(command_handle: i32,
                                    wallet_handle: i32,
                                    policy_address: *const c_char,
                                    verkey: *const c_char,
                                    add_commitment: bool,
                                    cb: Option<extern fn(xcommand_handle: i32,
                                                         err: i32,
                                                         vk: *const c_char)>
    ) -> i32;

    fn indy_update_agent_witness(command_handle: i32,
                                 wallet_handle: i32,
                                 policy_address: *const c_char,
                                 verkey: *const c_char,
                                 witness: *const c_char,
                                 cb: Option<extern fn(xcommand_handle: i32,
                                                      err: i32,
                                                      vk: *const c_char)>
    ) -> i32;

    fn indy_get_policy(command_handle: i32,
                       wallet_handle: i32,
                       policy_address: *const c_char,
                       cb: Option<extern fn(xcommand_handle: i32,
                                            err: i32,
                                            policy_json: *const c_char)>
    ) -> i32;
}

pub enum Permission {
    None = 0,
    Admin = 1,
    Prove = 2,
    ProveGrant = 4,
    ProveRevoke = 8,
}

pub fn update_verkey_in_policy(wallet_h: i32, pool_h: i32, admin_vk: &str, new_vk: &str, address: &str, auth: Permission, with_comm: bool) -> Result<(), u32> {

    let comm = match with_comm {
        true => {
            libindy_add_new_agent_to_policy(wallet_h,
                                            address,
                                            new_vk,
                                            with_comm)?;

            let updated_policy = libindy_get_policy(wallet_h,
                                                    address).unwrap();
            let updated_policy: Value = serde_json::from_str(&updated_policy).unwrap();

            //            Some(updated_policy["agents"][&new_vk]["double_commitment"]
//                    .as_str().ok_or(INVALID_JSON.code_num)?.to_string())
            use utils::dkms_constants::get_prime;
            Some(get_prime())
        },
        false => None

    };
    let comm_ref = match comm {
        Some(ref s) => Some(s.as_str()),
        None => None
    };

    let add_agent_txn = ledger::libindy_build_agent_authz_request(
        admin_vk,
        address,
        new_vk,
        auth as i32,
        comm_ref
    ).unwrap();

    println!("Add key txn: {}", add_agent_txn);



    let result = ledger::libindy_sign_and_submit_request(pool_h,
                                                         wallet_h,
                                                         admin_vk.to_owned(),
                                                         add_agent_txn)?;


    println!("**submit results: {}", result);
    Ok(())
}

pub fn libindy_create_and_store_new_policy(wallet_handle: i32)  -> Result<String, u32> {

    let rtn_obj = Return_I32_STR::new()?;

    unsafe {
        indy_function_eval(
            indy_create_and_store_new_policy(rtn_obj.command_handle,
                                             wallet_handle,
                                             Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    rtn_obj.receive(TimeoutUtils::some_long()).and_then(check_str)
}


pub fn libindy_add_new_agent_to_policy(wallet_handle: i32,
                                       policy_address: &str,
                                       verkey: &str,
                                       add_commitment: bool)  -> Result<String, u32>{

    let rtn_obj = Return_I32_STR::new()?;
    let policy_address = CString::new(policy_address).map_err(map_string_error)?;
    let verkey = CString::new(verkey).map_err(map_string_error)?;

    unsafe {
        indy_function_eval(
            indy_add_new_agent_to_policy(rtn_obj.command_handle,
                                       wallet_handle,
                                       policy_address.as_ptr(),
                                       verkey.as_ptr(),
                                       add_commitment,
                                       Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    rtn_obj.receive(TimeoutUtils::some_long()).and_then(check_str)
}


pub fn libindy_update_agent_witness(wallet_handle: i32,
                                       policy_address: &str,
                                       verkey: &str,
                                       witness: &str)  -> Result<String, u32>{

    let rtn_obj = Return_I32_STR::new()?;
    let policy_address = CString::new(policy_address).map_err(map_string_error)?;
    let verkey = CString::new(verkey).map_err(map_string_error)?;
    let witness = CString::new(witness).map_err(map_string_error)?;

    unsafe {
        indy_function_eval(
            indy_update_agent_witness(rtn_obj.command_handle,
                                      wallet_handle,
                                      policy_address.as_ptr(),
                                      verkey.as_ptr(),
                                      witness.as_ptr(),
                                      Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    rtn_obj.receive(TimeoutUtils::some_long()).and_then(check_str)
}

pub fn libindy_get_policy(wallet_handle: i32,
                          policy_address: &str)  -> Result<String, u32> {

    let rtn_obj = Return_I32_STR::new()?;
    let policy_address = CString::new(policy_address).map_err(map_string_error)?;

    unsafe {
        indy_function_eval(
            indy_get_policy(rtn_obj.command_handle,
                            wallet_handle,
                            policy_address.as_ptr(),
                            Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    rtn_obj.receive(TimeoutUtils::some_long()).and_then(check_str)
}