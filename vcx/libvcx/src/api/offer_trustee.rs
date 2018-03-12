extern crate libc;

use self::libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use connection;
use recovery_shares;
use offer_trustee;
use std::thread;
use std::ptr;


///
#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_offer_trustee_create(command_handle: u32,
                                      source_id: *const c_char,
                                      cb: Option<extern fn(xcommand_handle: u32, err: u32, handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);


    let source_id_opt = if !source_id.is_null() {
        check_useful_c_str!(source_id, error::INVALID_OPTION.code_num);
        let val = source_id.to_owned();
        Some(val)
    } else { None };

    thread::spawn(move|| {
        let (rc, handle) = match offer_trustee::offer_trustee_create(source_id_opt) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };

        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_offer_trustee_send_offer(command_handle: u32,
                                          handle: u32,
                                          connection_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !offer_trustee::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    if !connection::is_valid_handle(connection_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }


    thread::spawn(move|| {
        let err = match offer_trustee::send_trustee_offer(handle, connection_handle) {
            Ok(x) => x,
            Err(x) => x,
        };

        cb(command_handle,err);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_offer_trustee_update_state(command_handle: u32,
                                            handle: u32,
                                            cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !offer_trustee::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    thread::spawn(move|| {
        offer_trustee::update_state(handle);

        cb(command_handle, error::SUCCESS.code_num, offer_trustee::get_state(handle));
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_offer_trustee_get_state(command_handle: u32,
                                         handle: u32,
                                         cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !offer_trustee::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    thread::spawn(move|| {
        cb(command_handle, error::SUCCESS.code_num, offer_trustee::get_state(handle));
    });

    error::SUCCESS.code_num
}


///
#[no_mangle]
pub extern fn vcx_offer_trustee_send_data(command_handle: u32,
                                    handle: u32,
                                    recovery_shares_handle: u32,
                                    connection_handle: u32,
                                    cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !offer_trustee::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    if !recovery_shares::is_valid_handle(recovery_shares_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }

    if !connection::is_valid_handle(connection_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }

    thread::spawn(move|| {
        let err = match offer_trustee::send_trustee_data(handle, recovery_shares_handle, connection_handle) {
            Ok(x) => x,
            Err(x) => x,
        };

        cb(command_handle,err);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_offer_trustee_serialize(command_handle: u32,
                                         handle: u32,
                                         cb: Option<extern fn(xcommand_handle: u32, err: u32, state: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !offer_trustee::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match offer_trustee::to_string(handle) {
            Ok(x) => {
                info!("serializing handle: {} with data: {}",handle, x);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num,msg.as_ptr());
            },
            Err(x) => {
                warn!("could not serialize handle {}",handle);
                cb(command_handle,x,ptr::null_mut());
            },
        };
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_offer_trustee_deserialize(command_handle: u32,
                                      data: *const c_char,
                                      cb: Option<extern fn(xcommand_handle: u32, err: u32, handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_c_str!(data, error::INVALID_OPTION.code_num);

    thread::spawn(move|| {
        let (rc, handle) = match offer_trustee::from_string(&data) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };

        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_offer_trustee_release(handle: u32) -> u32 { offer_trustee::release(handle) }


#[cfg(test)]
mod tests {
//    use super::*;

}
