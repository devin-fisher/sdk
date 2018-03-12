extern crate libc;

use self::libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use connection;
use trustee;
use return_share;
use std::thread;
use std::ptr;


///

#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_return_share_create_with_request(command_handle: u32,
                                          source_id: *const c_char,
                                          req: *const c_char,
                                          cb: Option<extern fn(xcommand_handle: u32, err: u32, handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_opt_c_str!(source_id, error::INVALID_OPTION.code_num);
    check_useful_c_str!(req, error::INVALID_OPTION.code_num);


    thread::spawn(move|| {
        let (rc, handle) = match return_share::create_return_share(source_id, &req){
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };

        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_return_share_send_share(command_handle: u32,
                                          handle: u32,
                                          connection_handle: u32,
                                          trustee_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !return_share::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    if !connection::is_valid_handle(connection_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }

    if !trustee::is_valid_handle(trustee_handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    thread::spawn(move|| {
        let err = match return_share::send_share(handle, connection_handle, trustee_handle) {
            Ok(x) => x,
            Err(x) => x,
        };

        cb(command_handle,err);
    });

    error::SUCCESS.code_num
}
//
/////
//
#[no_mangle]
pub extern fn vcx_return_share_new_pings(command_handle: u32,
                                   connection_handle: u32,
                                   cb: Option<extern fn(xcommand_handle: u32, err: u32, requests: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !connection::is_valid_handle(connection_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match return_share::new_ping_messages(connection_handle, None) {
            Ok(x) => {
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("could not retrieve trust pings");
                cb(command_handle, x, ptr::null_mut());
            },
        };
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_return_share_update_state(command_handle: u32,
                                            handle: u32,
                                            cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !return_share::is_valid_handle(handle) {
        return error::INVALID_ISSUER_CLAIM_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match return_share::update_state(handle) {
            Ok(_) => (),
            Err(e) => cb(command_handle, e, 0)
        }

        match return_share::get_state(handle) {
            Ok(s) => cb(command_handle, error::SUCCESS.code_num, s),
            Err(e) => cb(command_handle, e, 0)
        };
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_return_share_get_state(command_handle: u32,
                                  handle: u32,
                                  cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {
    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !return_share::is_valid_handle(handle) {
        return error::INVALID_PROOF_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match return_share::get_state(handle) {
            Ok(s) => cb(command_handle, error::SUCCESS.code_num, s),
            Err(e) => cb(command_handle, e, 0)
        };
    });

    error::SUCCESS.code_num
}


///
#[no_mangle]
pub extern fn vcx_return_share_serialize(command_handle: u32,
                                         handle: u32,
                                         cb: Option<extern fn(xcommand_handle: u32, err: u32, data: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !return_share::is_valid_handle(handle) {
        return error::INVALID_ISSUER_CLAIM_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match return_share::to_string(handle) {
            Ok(x) => {
                info!("serializing handle: {} with data: {}",handle, x);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num,msg.as_ptr());
            },
            Err(x) => {
                warn!("could not serialize handle {}", handle);
                cb(command_handle,x,ptr::null_mut());
            },
        };
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_return_share_deserialize(command_handle: u32,
                                           data: *const c_char,
                                           cb: Option<extern fn(xcommand_handle: u32, err: u32, handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_c_str!(data, error::INVALID_OPTION.code_num);

    thread::spawn(move|| {
        let (rc, handle) = match return_share::from_string(&data) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };

        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}


///
#[no_mangle]
pub extern fn vcx_return_share_release(handle: u32) -> u32 {
    match return_share::release(handle) {
        Ok(_) => error::SUCCESS.code_num,
        Err(e) => e
    }
}