extern crate libc;
use self::libc::c_char;
use std::ffi::CString;
use utils::libindy::{indy_function_eval, check_str};
use utils::libindy::return_types::{ Return_I32_STR };
use utils::libindy::error_codes::{map_indy_error_code, map_string_error};
use utils::timeout::TimeoutUtils;

extern {
    fn indy_shard_msg_with_secret_and_store_shards(command_handle: i32,
                                                   wallet_handle: i32,
                                                   m: u8,
                                                   n: u8,
                                                   msg: *const c_char,
                                                   verkey: *const c_char,
                                                   cb: Option<extern fn(xcommand_handle: i32,
                                                                        err: i32,
                                                                        vk: *const c_char)>
    ) -> i32;

    fn indy_get_shards_of_verkey(command_handle: i32,
                                 wallet_handle: i32,
                                 verkey: *const c_char,
                                 cb: Option<extern fn(xcommand_handle: i32,
                                                      err: i32,
                                                      shards_json: *const c_char)>
    ) -> i32;

    fn indy_get_shard_of_verkey(command_handle: i32,
                                wallet_handle: i32,
                                verkey: *const c_char,
                                shard_number: u8,
                                cb: Option<extern fn(xcommand_handle: i32,
                                                     err: i32,
                                                     shard: *const c_char)>
    ) -> i32;

    fn indy_recover_secret_from_shards(command_handle: i32,
                                       shards_json: *const c_char,
                                       cb: Option<extern fn(xcommand_handle: i32, 
                                                            err: i32,
                                                            secret: *const c_char)>
    ) -> i32;
}

pub fn libindy_shard_msg_with_secret_and_store_shards(wallet_handle: i32,
                                                      m: u8,
                                                      n: u8,
                                                      msg: &str,
                                                      verkey: &str) -> Result<String, u32>
{
    let rtn_obj = Return_I32_STR::new()?;
    let msg = CString::new(msg).map_err(map_string_error)?;
    let verkey = CString::new(verkey).map_err(map_string_error)?;
    unsafe {
        indy_function_eval(
            indy_shard_msg_with_secret_and_store_shards(rtn_obj.command_handle,
                                                        wallet_handle as i32,
                                                        m,
                                                        n,
                                                        msg.as_ptr(),
                                                        verkey.as_ptr(),
                                                        Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    rtn_obj.receive(TimeoutUtils::some_long()).and_then(check_str)
}

pub fn libindy_get_shards_of_verkey(wallet_handle: i32,
                                    verkey: &str) -> Result<String, u32>
{
    let rtn_obj = Return_I32_STR::new()?;
    let verkey = CString::new(verkey).map_err(map_string_error)?;
    unsafe {
        indy_function_eval(
            indy_get_shards_of_verkey(rtn_obj.command_handle,
                                      wallet_handle as i32,
                                         verkey.as_ptr(),
                                         Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    rtn_obj.receive(TimeoutUtils::some_long()).and_then(check_str)
}

pub fn libindy_get_shard_of_verkey(wallet_handle: i32,
                                   verkey: &str,
                                   share_num: u8) -> Result<String, u32>
{
    let rtn_obj = Return_I32_STR::new()?;
    let verkey = CString::new(verkey).map_err(map_string_error)?;
    unsafe {
        indy_function_eval(
            indy_get_shard_of_verkey(rtn_obj.command_handle,
                                     wallet_handle as i32,
                                     verkey.as_ptr(),
                                     share_num,
                                     Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    rtn_obj.receive(TimeoutUtils::some_long()).and_then(check_str)
}

pub fn libindy_recover_secret_from_shards(shards_json: &str) -> Result<String, u32>
{
    let rtn_obj = Return_I32_STR::new()?;
    let shards_json = CString::new(shards_json).map_err(map_string_error)?;
    unsafe {
        indy_function_eval(
            indy_recover_secret_from_shards(rtn_obj.command_handle,
                                            shards_json.as_ptr(),
                                            Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    rtn_obj.receive(TimeoutUtils::some_long()).and_then(check_str)
}