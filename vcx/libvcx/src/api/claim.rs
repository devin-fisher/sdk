extern crate libc;

use self::libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use connection;
use claim;
use std::thread;
use std::ptr;


/// Create a Issuer Claim object that provides a claim for an enterprise's user

#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_claim_create_with_offer(command_handle: u32,
                                          source_id: *const c_char,
                                          offer: *const c_char,
                                          cb: Option<extern fn(xcommand_handle: u32, err: u32, claim_handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_opt_c_str!(source_id, error::INVALID_OPTION.code_num);
    check_useful_c_str!(offer, error::INVALID_OPTION.code_num);


    thread::spawn(move|| {
        let (rc, handle) = match claim::claim_create_with_offer(source_id, &offer) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };

        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_claim_send_request(command_handle: u32,
                                          claim_handle: u32,
                                          connection_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !claim::is_valid_handle(claim_handle) {
        return error::INVALID_ISSUER_CLAIM_HANDLE.code_num;
    }

    if !connection::is_valid_handle(connection_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }


    thread::spawn(move|| {
        let err = match claim::send_claim_request(claim_handle, connection_handle) {
            Ok(x) => x,
            Err(x) => x,
        };

        cb(command_handle,err);
    });

    error::SUCCESS.code_num
}

///

#[no_mangle]
pub extern fn vcx_claim_new_offers(command_handle: u32,
                                   connection_handle: u32,
                                   cb: Option<extern fn(xcommand_handle: u32, err: u32, claim_offers: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !connection::is_valid_handle(connection_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match claim::new_claims_offer_messages(connection_handle, None) {
            Ok(x) => {
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("could not retrive claim offers");
                cb(command_handle, x, ptr::null_mut());
            },
        };
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_claim_update_state(command_handle: u32,
                                            claim_handle: u32,
                                            cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !claim::is_valid_handle(claim_handle) {
        return error::INVALID_ISSUER_CLAIM_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match claim::update_state(claim_handle) {
            Ok(_) => (),
            Err(e) => cb(command_handle, e, 0)
        }

        let state = match claim::get_state(claim_handle) {
            Ok(s) => cb(command_handle, error::SUCCESS.code_num, s),
            Err(e) => cb(command_handle, e, 0)
        };
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_claim_get_state(command_handle: u32,
                                  handle: u32,
                                  cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {
    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !claim::is_valid_handle(handle) {
        return error::INVALID_PROOF_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match claim::get_state(handle) {
            Ok(s) => cb(command_handle, error::SUCCESS.code_num, s),
            Err(e) => cb(command_handle, e, 0)
        };
    });

    error::SUCCESS.code_num
}


/// Takes the claim object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// handle: Claim handle that was provided during creation. Used to identify claim object
///
/// cb: Callback that provides json string of the claim's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_claim_serialize(command_handle: u32,
                                         handle: u32,
                                         cb: Option<extern fn(xcommand_handle: u32, err: u32, data: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !claim::is_valid_handle(handle) {
        return error::INVALID_ISSUER_CLAIM_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match claim::to_string(handle) {
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

/// Takes a json string representing an claim object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// claim_data: json string representing a claim object
///
///
/// cb: Callback that provides claim handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_claim_deserialize(command_handle: u32,
                                           claim_data: *const c_char,
                                           cb: Option<extern fn(xcommand_handle: u32, err: u32, handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_c_str!(claim_data, error::INVALID_OPTION.code_num);

    thread::spawn(move|| {
        let (rc, handle) = match claim::from_string(&claim_data) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };

        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

/// Releases the claim object by de-allocating memory
///
/// #Params
/// handle: Proof handle that was provided during creation. Used to access claim object
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_claim_release(handle: u32) -> u32 {
    match claim::release(handle) {
        Ok(_) => error::SUCCESS.code_num,
        Err(e) => e
    }
}