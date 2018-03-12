extern crate libc;

use self::libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use connection;
use trustee;
use std::thread;
use std::ptr;


///

#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_trustee_create_with_offer(command_handle: u32,
                                          source_id: *const c_char,
                                          offer: *const c_char,
                                          cb: Option<extern fn(xcommand_handle: u32, err: u32, handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_opt_c_str!(source_id, error::INVALID_OPTION.code_num);
    check_useful_c_str!(offer, error::INVALID_OPTION.code_num);


    thread::spawn(move|| {
        let (rc, handle) = match trustee::trustee_create_with_offer(source_id, &offer) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };

        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_trustee_send_request(command_handle: u32,
                                          trustee_handle: u32,
                                          connection_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !trustee::is_valid_handle(trustee_handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    if !connection::is_valid_handle(connection_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }


    thread::spawn(move|| {
        let err = match trustee::send_trustee_request(trustee_handle, connection_handle) {
            Ok(x) => x,
            Err(x) => x,
        };

        cb(command_handle,err);
    });

    error::SUCCESS.code_num
}

///

#[no_mangle]
pub extern fn vcx_trustee_new_offers(command_handle: u32,
                                   connection_handle: u32,
                                   cb: Option<extern fn(xcommand_handle: u32, err: u32, trustee_offers: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !connection::is_valid_handle(connection_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match trustee::new_trustee_offer_messages(connection_handle, None) {
            Ok(x) => {
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("could not retrieve offers");
                cb(command_handle, x, ptr::null_mut());
            },
        };
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_trustee_update_state(command_handle: u32,
                                            trustee_handle: u32,
                                            cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !trustee::is_valid_handle(trustee_handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match trustee::update_state(trustee_handle) {
            Ok(_) => (),
            Err(e) => cb(command_handle, e, 0)
        }

        let state = match trustee::get_state(trustee_handle) {
            Ok(s) => cb(command_handle, error::SUCCESS.code_num, s),
            Err(e) => cb(command_handle, e, 0)
        };
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_trustee_get_state(command_handle: u32,
                                    handle: u32,
                                    cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {
    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !trustee::is_valid_handle(handle) {
        return error::INVALID_PROOF_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match trustee::get_state(handle) {
            Ok(s) => cb(command_handle, error::SUCCESS.code_num, s),
            Err(e) => cb(command_handle, e, 0)
        };
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_trustee_revoke_device(command_handle: u32,
                                        handle: u32,
                                        verkey: *const c_char,
                                        cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {
    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_c_str!(verkey, error::INVALID_OPTION.code_num);

    if !trustee::is_valid_handle(handle) {
        return error::INVALID_PROOF_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match trustee::revoke_key(handle, &verkey) {
            Ok(s) => cb(command_handle, error::SUCCESS.code_num),
            Err(e) => cb(command_handle, e)
        };
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_trustee_list_agents(command_handle: u32,
                                      handle: u32,
                                      cb: Option<extern fn(xcommand_handle: u32, err: u32, data: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !trustee::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match trustee::list_agents(handle) {
            Ok(x) => {
                info!("list agents handle: {} with data: {}",handle, x);
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num,msg.as_ptr());
            },
            Err(x) => {
                warn!("could not list agents from handle {}",handle);
                cb(command_handle,x,ptr::null_mut());
            },
        };
    });

    error::SUCCESS.code_num
}


///
#[no_mangle]
pub extern fn vcx_trustee_serialize(command_handle: u32,
                                    handle: u32,
                                    cb: Option<extern fn(xcommand_handle: u32, err: u32, data: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !trustee::is_valid_handle(handle) {
        return error::INVALID_OBJ_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match trustee::to_string(handle) {
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
pub extern fn vcx_trustee_deserialize(command_handle: u32,
                                      trustee_data: *const c_char,
                                      cb: Option<extern fn(xcommand_handle: u32, err: u32, handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_c_str!(trustee_data, error::INVALID_OPTION.code_num);

    thread::spawn(move|| {
        let (rc, handle) = match trustee::from_string(&trustee_data) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };

        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_trustee_release(handle: u32) -> u32 {
    match trustee::release(handle) {
        Ok(_) => error::SUCCESS.code_num,
        Err(e) => e
    }
}