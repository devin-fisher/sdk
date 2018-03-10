extern crate libc;

use self::libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use recovery_shares;
use connection;
use std::thread;
use std::ptr;

///
#[no_mangle]
pub extern fn vcx_recovery_shares_create(command_handle: u32,
                                    source_id: *const c_char,
                                    count: u32,
                                    threshold: u32,
                                    cb: Option<extern fn(xcommand_handle: u32, err: u32, proof_handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    let source_id_opt = if !source_id.is_null() {
        check_useful_c_str!(source_id, error::INVALID_OPTION.code_num);
        let val = source_id.to_owned();
        Some(val)
    } else { None };

    thread::spawn( move|| {
        let ( rc, handle) = match recovery_shares::create(source_id_opt, count, threshold) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };
        info!("ping creation had return code: {}", rc);
        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_recovery_shares_serialize(command_handle: u32,
                                       handle: u32,
                                       cb: Option<extern fn(xcommand_handle: u32, err: u32, state: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !recovery_shares::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    };

    thread::spawn( move|| {
        match recovery_shares::to_string(handle) {
            Ok(x) => {
                info!("serializing recovery_shares handle: {} with data: {}", handle, x);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("could not serialize recovery_shares handle {}", handle);
                cb(command_handle, x, ptr::null_mut());
            },
        };

    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_recovery_shares_deserialize(command_handle: u32,
                                         data: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: u32, err: u32, proof_handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_c_str!(data, error::INVALID_OPTION.code_num);

    thread::spawn( move|| {
        let (rc, handle) = match recovery_shares::from_string(&data) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };
        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_recovery_shares_release(handle: u32) -> u32 {
    match recovery_shares::release(handle) {
        Ok(_) => error::SUCCESS.code_num,
        Err(e) => e
    }
}


#[cfg(test)]
mod tests {
    use super::*;

}
