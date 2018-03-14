extern crate libc;
use self::libc::c_char;
use std::ffi::CString;
use utils::error;
use utils::libindy::{indy_function_eval, check_str};
use utils::libindy::return_types::{ Return_I32_STR, Return_I32_BOOL, Return_I32_STR_STR, Return_I32 };
use utils::libindy::SigTypes;
use utils::libindy::error_codes::{map_indy_error_code, map_string_error};
use utils::timeout::TimeoutUtils;
use utils::libindy::option_cstring_as_ptn;

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

//
//pub fn libindy_add_new_agent_to_policy(wallet_handle: i32,
//                                       policy_address: &str,
//                                       verkey: &str,
//                                       add_commitment: bool)  -> Result<bool, u32>{
//
//    let rtn_obj = Return_I32_STR::new()?;
//    let policy_address = CString::new(policy_address).map_err(map_string_error)?;
//    let proof_json = CString::new(proof_json.to_string()).map_err(map_string_error)?;
//    let schemas_json = CString::new(schemas_json.to_string()).map_err(map_string_error)?;
//    let claim_defs_json = CString::new(claim_defs_json.to_string()).map_err(map_string_error)?;
//    let revoc_regs_json = CString::new(revoc_regs_json.to_string()).map_err(map_string_error)?;
//    unsafe {
//        indy_add_new_agent_to_policy(
//            indy_verifier_verify_proof(rtn_obj.command_handle,
//                                       proof_req_json.as_ptr(),
//                                       proof_json.as_ptr(),
//                                       schemas_json.as_ptr(),
//                                       claim_defs_json.as_ptr(),
//                                       revoc_regs_json.as_ptr(),
//                                       Some(rtn_obj.get_callback()))
//        ).map_err(map_indy_error_code)?;
//    }
//
//    rtn_obj.receive(TimeoutUtils::some_long())
//}