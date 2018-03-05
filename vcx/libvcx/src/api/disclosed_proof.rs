extern crate libc;

use self::libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use connection;
use disclosed_proof;
use std::thread;
use std::ptr;


///

#[no_mangle]
#[allow(unused_variables, unused_mut)]
pub extern fn vcx_disclosed_proof_create_with_request(command_handle: u32,
                                          source_id: *const c_char,
                                          req: *const c_char,
                                          cb: Option<extern fn(xcommand_handle: u32, err: u32, handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_opt_c_str!(source_id, error::INVALID_OPTION.code_num);
    check_useful_c_str!(req, error::INVALID_OPTION.code_num);


    thread::spawn(move|| {
        let (rc, handle) = match disclosed_proof::create_proof(source_id, &req){
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };

        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_disclosed_proof_send_proof(command_handle: u32,
                                          handle: u32,
                                          connection_handle: u32,
                                          cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !disclosed_proof::is_valid_handle(handle) {
        return error::INVALID_ISSUER_CLAIM_HANDLE.code_num;
    }

    if !connection::is_valid_handle(connection_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }



    thread::spawn(move|| {
        let err = match disclosed_proof::send_proof(handle, connection_handle) {
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
pub extern fn vcx_disclosed_proof_new_requests(command_handle: u32,
                                   connection_handle: u32,
                                   cb: Option<extern fn(xcommand_handle: u32, err: u32, requests: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !connection::is_valid_handle(connection_handle) {
        return error::INVALID_CONNECTION_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match disclosed_proof::new_proof_requests_messages(connection_handle, None) {
            Ok(x) => {
                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            },
            Err(x) => {
                warn!("could not retrive proof requests");
                cb(command_handle, x, ptr::null_mut());
            },
        };
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_disclosed_proof_update_state(command_handle: u32,
                                            handle: u32,
                                            cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !disclosed_proof::is_valid_handle(handle) {
        return error::INVALID_ISSUER_CLAIM_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match disclosed_proof::update_state(handle) {
            Ok(_) => (),
            Err(e) => cb(command_handle, e, 0)
        }

        match disclosed_proof::get_state(handle) {
            Ok(s) => cb(command_handle, error::SUCCESS.code_num, s),
            Err(e) => cb(command_handle, e, 0)
        };
    });

    error::SUCCESS.code_num
}

#[no_mangle]
pub extern fn vcx_disclosed_proof_get_state(command_handle: u32,
                                  handle: u32,
                                  cb: Option<extern fn(xcommand_handle: u32, err: u32, state: u32)>) -> u32 {
    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !disclosed_proof::is_valid_handle(handle) {
        return error::INVALID_PROOF_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match disclosed_proof::get_state(handle) {
            Ok(s) => cb(command_handle, error::SUCCESS.code_num, s),
            Err(e) => cb(command_handle, e, 0)
        };
    });

    error::SUCCESS.code_num
}


/// Takes the disclosed proof object and returns a json string of all its attributes
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// handle: Claim handle that was provided during creation. Used to identify the disclosed proof object
///
/// cb: Callback that provides json string of the disclosed proof's attributes and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_serialize(command_handle: u32,
                                         handle: u32,
                                         cb: Option<extern fn(xcommand_handle: u32, err: u32, data: *const c_char)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    if !disclosed_proof::is_valid_handle(handle) {
        return error::INVALID_ISSUER_CLAIM_HANDLE.code_num;
    }

    thread::spawn(move|| {
        match disclosed_proof::to_string(handle) {
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

/// Takes a json string representing an disclosed proof object and recreates an object matching the json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// data: json string representing a disclosed proof object
///
///
/// cb: Callback that provides handle and provides error status
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_deserialize(command_handle: u32,
                                           data: *const c_char,
                                           cb: Option<extern fn(xcommand_handle: u32, err: u32, handle: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);
    check_useful_c_str!(data, error::INVALID_OPTION.code_num);

    thread::spawn(move|| {
        let (rc, handle) = match disclosed_proof::from_string(&data) {
            Ok(x) => (error::SUCCESS.code_num, x),
            Err(x) => (x, 0),
        };

        cb(command_handle, rc, handle);
    });

    error::SUCCESS.code_num
}


/// Releases the disclosed proof object by de-allocating memory
///
/// #Params
/// handle: Proof handle that was provided during creation. Used to access proof object
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_disclosed_proof_release(handle: u32) -> u32 {
    match disclosed_proof::release(handle) {
        Ok(_) => error::SUCCESS.code_num,
        Err(e) => e
    }
}