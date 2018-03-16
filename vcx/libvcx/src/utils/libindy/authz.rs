extern crate libc;
use self::libc::c_char;
use std::ffi::CString;
use utils::libindy::{indy_function_eval, check_str};
use utils::libindy::return_types::{ Return_I32_STR };
use utils::libindy::error_codes::{map_indy_error_code, map_string_error};
use utils::timeout::TimeoutUtils;

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

pub fn libindy_create_and_store_new_policy(wallet_handle: i32)  -> Result<String, u32> {

    let rtn_obj = Return_I32_STR::new()?;

    unsafe {
        indy_function_eval(
            indy_create_and_store_new_policy(rtn_obj.command_handle,
                                             wallet_handle,
                                             Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    rtn_obj.receive(TimeoutUtils::some_short()).and_then(check_str)
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

    rtn_obj.receive(TimeoutUtils::some_short()).and_then(check_str)
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

    rtn_obj.receive(TimeoutUtils::some_short()).and_then(check_str)
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

    rtn_obj.receive(TimeoutUtils::some_short()).and_then(check_str)
}