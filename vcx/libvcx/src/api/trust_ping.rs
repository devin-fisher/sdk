extern crate libc;

use self::libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use trust_ping;
use connection;
use std::thread;
use std::ptr;

///
#[no_mangle]
pub extern fn vcx_trust_ping_create(command_handle: u32,
                               source_id: *const c_char,
                               cb: Option<extern fn(xcommand_handle: u32, err: u32, proof_handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    let source_id_opt = if !source_id.is_null() {
        check_useful_c_str!(source_id, error::INVALID_OPTION.code_num);
        let val = source_id.to_owned();
        Some(val)
    } else { None };

    thread::spawn( move|| {
        let ( rc, handle) = match trust_ping::create_ping(source_id_opt) {
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
pub extern fn vcx_trust_ping_update_state(command_handle: u32,
                                     handle: u32,
                                     cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {
    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !trust_ping::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    thread::spawn(move|| {
        trust_ping::update_state(handle);

        cb(command_handle, error::SUCCESS.code_num, trust_ping::get_state(handle));
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_trust_ping_get_state(command_handle: u32,
                                     handle: u32,
                                     cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {
    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !trust_ping::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    thread::spawn(move|| {
        cb(command_handle, error::SUCCESS.code_num, trust_ping::get_state(handle));
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_trust_ping_serialize(command_handle: u32,
                                  handle: u32,
                                  cb: Option<extern fn(xcommand_handle: u32, err: u32, state: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !trust_ping::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    };

    thread::spawn( move|| {
        match trust_ping::to_string(handle) {
            Ok(x) => {
                info!("serializing trust_ping handle: {} with data: {}", handle, x);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("could not serialize trust_ping handle {}", handle);
                cb(command_handle, x, ptr::null_mut());
            },
        };

    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_trust_ping_deserialize(command_handle: u32,
                                    data: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: u32, err: u32, proof_handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_c_str!(data, error::INVALID_OPTION.code_num);

    thread::spawn( move|| {
        let (rc, handle) = match trust_ping::from_string(&data) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };
        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_trust_ping_release(handle: u32) -> u32 {
    trust_ping::release(handle)
}

///
#[no_mangle]
pub extern fn vcx_trust_ping_send_request(command_handle: u32,
                                     handle: u32,
                                     connection_handle: u32,
                                     cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !trust_ping::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    if !connection::is_valid_handle(connection_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }

    thread::spawn(move|| {
        let err = match trust_ping::send_ping_request(handle, connection_handle) {
            Ok(x) => x,
            Err(x) => x,
        };

        cb(command_handle,err);
    });

    error::SUCCESS.code_num
}

#[cfg(test)]
mod tests {
    use super::*;

}
