extern crate libc;

use self::libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use request_share;
use connection;
use std::thread;
use std::ptr;

///
#[no_mangle]
pub extern fn vcx_request_share_create(command_handle: u32,
                               source_id: *const c_char,
                               cb: Option<extern fn(xcommand_handle: u32, err: u32, proof_handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    let source_id_opt = if !source_id.is_null() {
        check_useful_c_str!(source_id, error::INVALID_OPTION.code_num);
        let val = source_id.to_owned();
        Some(val)
    } else { None };

    thread::spawn( move|| {
        let ( rc, handle) = match request_share::create_request_share(source_id_opt) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };
        info!("request share creation had return code: {}", rc);
        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_request_share_update_state(command_handle: u32,
                                     handle: u32,
                                     cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {
    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !request_share::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    thread::spawn(move|| {
        request_share::update_state(handle);

        cb(command_handle, error::SUCCESS.code_num, request_share::get_state(handle));
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_request_share_get_state(command_handle: u32,
                                     handle: u32,
                                     cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {
    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !request_share::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    thread::spawn(move|| {
        cb(command_handle, error::SUCCESS.code_num, request_share::get_state(handle));
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_request_share_serialize(command_handle: u32,
                                  handle: u32,
                                  cb: Option<extern fn(xcommand_handle: u32, err: u32, state: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !request_share::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    };

    thread::spawn( move|| {
        match request_share::to_string(handle) {
            Ok(x) => {
                info!("serializing request_share handle: {} with data: {}", handle, x);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("could not serialize request_share handle {}", handle);
                cb(command_handle, x, ptr::null_mut());
            },
        };

    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_request_share_deserialize(command_handle: u32,
                                    data: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: u32, err: u32, proof_handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_c_str!(data, error::INVALID_OPTION.code_num);

    thread::spawn( move|| {
        let (rc, handle) = match request_share::from_string(&data) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };
        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_request_share_release(handle: u32) -> u32 {
    request_share::release(handle)
}

///
#[no_mangle]
pub extern fn vcx_request_share_send_request(command_handle: u32,
                                     handle: u32,
                                     connection_handle: u32,
                                     cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !request_share::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    if !connection::is_valid_handle(connection_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }

    thread::spawn(move|| {
        let err = match request_share::send_share_request(handle, connection_handle) {
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
