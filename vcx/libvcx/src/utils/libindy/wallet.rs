extern crate libc;

use self::libc::c_char;
use std::ffi::CString;
use settings;
use std::ptr::null;
use utils::libindy::{indy_function_eval};
use utils::libindy::return_types::{ Return_I32, Return_I32_I32};
use utils::libindy::error_codes::{map_indy_error_code, map_string_error};
use utils::timeout::TimeoutUtils;
use utils::libindy::option_cstring_as_ptn;
use utils::error;
use utils::libindy::crypto;
use utils::libindy::pool;
use utils::libindy::authz::{update_verkey_in_policy, Permission};

pub static mut WALLET_HANDLE: i32 = 0;

extern {
    fn indy_create_wallet(command_handle: i32,
                          pool_name: *const c_char,
                          name: *const c_char,
                          xtype: *const c_char,
                          config: *const c_char,
                          credentials: *const c_char,
                          cb: Option<extern fn(xcommand_handle: i32, err: i32)>) -> i32;

    fn indy_open_wallet(command_handle: i32,
                        name: *const c_char,
                        runtime_config: *const c_char,
                        credentials: *const c_char,
                        cb: Option<extern fn(xcommand_handle: i32, err: i32, handle: i32)>) -> i32;

    fn indy_close_wallet(command_handle: i32,
                         handle: i32,
                         cb: Option<extern fn(xcommand_handle: i32, err: i32)>) -> i32;

    fn indy_delete_wallet(command_handle: i32,
                          name: *const c_char,
                          credentials: *const c_char,
                          cb: Option<extern fn(xcommand_handle: i32, err: i32)>) -> i32;

    fn indy_create_and_store_my_did(command_handle: i32,
                                    wallet_handle: i32,
                                    did_json: *const c_char,
                                    cb: Option<extern fn(xcommand_handle: i32, err: i32,
                                                         did: *const c_char,
                                                         verkey: *const c_char,
                                                         pk: *const c_char)>) -> i32;

    fn indy_store_their_did(command_handle: i32,
                            wallet_handle: i32,
                            identity_json: *const c_char,
                            cb: Option<extern fn(xcommand_handle: i32, err: i32)>) -> i32;
}

pub fn get_wallet_handle() -> i32 { unsafe { WALLET_HANDLE } }

pub fn init_wallet(wallet_name: &str) -> Result<i32, u32> {
    if settings::test_indy_mode_enabled() {
        unsafe {WALLET_HANDLE = 1;}
        return Ok(1);
    }
    let pool_name = match settings::get_config_value(settings::CONFIG_POOL_NAME) {
        Ok(x) => x,
        Err(_) => "pool1".to_owned(),
    };

    let wallet_type = match settings::get_config_value(settings::CONFIG_WALLET_TYPE) {
        Ok(x) => x,
        Err(_) => "default".to_owned(),
    };
    let credentials = match settings::get_wallet_credentials() {
        Some(x) => {
            info!("using key for indy wallet");
            Some(x)
        },
        None => None,
    };

    match create_wallet(pool_name.as_str(),
                  wallet_name,
                  Some(wallet_type.as_str()),
                  None,
                  credentials.as_ref().map(String::as_str)) {
        Ok(_) => (),
        Err(e) => {
            if e != error::WALLET_ALREADY_EXISTS.code_num {
                return Err(e);
            }
        },
    }

    let wallet_handle = match open_wallet(wallet_name,
                None,
                credentials.as_ref().map(String::as_str)){
        Ok(h) => h,
        Err(e) => {
            if e == error::WALLET_ALREADY_OPEN.code_num {
                return Ok(get_wallet_handle());
            }
            else {
                return Err(e);
            }
        },
    };

    //** DKMS ADDITION **
    match settings::get_config_value(settings::CONFIG_AGENT_POLICY_VERKEY) {
        Ok(_) => warn!("Has CONFIG_AGENT_POLICY_VERKEY, will not create one"),
        Err(_) => {
            let agent_verkey = crypto::libindy_create_key(wallet_handle, "{}").map_err(|x|{
                x as u32
            })?;
            warn!("Creating agent key: {}", agent_verkey);
            settings::set_config_value(settings::CONFIG_AGENT_POLICY_VERKEY, &agent_verkey);
            match settings::get_config_value(settings::CONFIG_IDENTITY_POLICY_ADDRESS) {
                Ok(addr) => {
                    if !addr.is_empty() {
                        warn!("Adding Agent key to Policy via recovery key");
                        let p_h = pool::get_pool_handle().unwrap();

                        let recovery_vk = settings::get_config_value(
                            settings::CONFIG_RECOVERY_VERKEY).expect("Expected RECOVERY_VERKEY");

                        update_verkey_in_policy(wallet_handle,
                                                p_h,
                                                &recovery_vk,
                                                &agent_verkey,
                                                &addr,
                                                Permission::Prove,
                                                true)?;
                    }
                },
                Err(_) =>()
            }
        },

    };


    unsafe {
        WALLET_HANDLE = wallet_handle; //TODO this is a bad idea, consider Option<std::sync::atomic::AtomicI32>
    }

    Ok(wallet_handle)
}

pub fn create_wallet(pool_name: &str, wallet_name: &str, xtype: Option<&str>, config: Option<&str>, credentials: Option<&str>) -> Result<(), u32> {
    let rtn_obj = Return_I32::new()?;

    let pool_name = CString::new(pool_name).map_err(map_string_error)?;
    let wallet_name = CString::new(wallet_name).map_err(map_string_error)?;
    let xtype = match xtype {
        Some(s) => Some(CString::new(s).map_err(map_string_error)?),
        None => None
    };
    let config = match config {
        Some(s) => Some(CString::new(s).map_err(map_string_error)?),
        None => None
    };
    let credentials = match credentials {
        Some(s) => Some(CString::new(s).map_err(map_string_error)?),
        None => None
    };

    unsafe {
        indy_function_eval(
            indy_create_wallet(rtn_obj.command_handle,
                              pool_name.as_ptr(),
                               wallet_name.as_ptr(),
                               option_cstring_as_ptn(&xtype),
                               option_cstring_as_ptn(&config),
                               option_cstring_as_ptn(&credentials),
                              Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }
    rtn_obj.receive(TimeoutUtils::some_medium())
}

pub fn open_wallet(wallet_name: &str, config: Option<&str>, credentials: Option<&str>) -> Result<i32, u32> {
    let rtn_obj = Return_I32_I32::new()?;

    let wallet_name = CString::new(wallet_name).map_err(map_string_error)?;
    let config = match config {
        Some(s) => Some(CString::new(s).map_err(map_string_error)?),
        None => None
    };
    let credentials = match credentials {
        Some(s) => Some(CString::new(s).map_err(map_string_error)?),
        None => None
    };

    unsafe {
        indy_function_eval(
            indy_open_wallet(rtn_obj.command_handle,
                               wallet_name.as_ptr(),
                               option_cstring_as_ptn(&config),
                               option_cstring_as_ptn(&credentials),
                               Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }
    rtn_obj.receive(TimeoutUtils::some_medium())
}

pub fn close_wallet(wallet_handle: i32) -> Result<(), u32> {
    let rtn_obj = Return_I32::new()?;

    unsafe {
        indy_function_eval(
            indy_close_wallet(rtn_obj.command_handle,
                              wallet_handle,
                             Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }
    rtn_obj.receive(TimeoutUtils::some_long())
}

pub fn delete_wallet(wallet_name: &str) -> Result<(), u32> {
    if settings::test_indy_mode_enabled() {
        unsafe { WALLET_HANDLE = 0;}
        return Ok(())
    }

    let rtn_obj = Return_I32::new()?;
    let wallet_name = CString::new(wallet_name).map_err(map_string_error)?;

    unsafe {
        indy_function_eval(
            indy_delete_wallet(rtn_obj.command_handle,
                              wallet_name.as_ptr(),
                              null(),
                              Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }
    rtn_obj.receive(TimeoutUtils::some_long())
}

pub fn store_their_did(identity_json: &str) -> Result<(), u32> {

    let identity_json = CString::new(identity_json.to_string()).map_err(map_string_error)?;
    let wallet_handle = get_wallet_handle();

    let rtn_obj = Return_I32::new()?;

    unsafe {
        indy_function_eval(
            indy_store_their_did(rtn_obj.command_handle,
                                 wallet_handle,
                                 identity_json.as_ptr(),
                                 Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }
    rtn_obj.receive(TimeoutUtils::some_long())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use utils::error;
    use std::thread;
    use std::time::Duration;
    use utils::libindy::signus::SignusUtils;

    #[test]
    fn test_wallet() {
        settings::set_defaults();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE,"false");
        let wallet_name = String::from("walletUnique");
        let mut wallet_handle = init_wallet(&wallet_name).unwrap();
        assert!( wallet_handle > 0);
        assert_eq!(error::UNKNOWN_LIBINDY_ERROR.code_num, init_wallet(&String::from("")).unwrap_err());

        thread::sleep(Duration::from_secs(1));
        delete_wallet("walletUnique").unwrap();
        let handle = get_wallet_handle();
        let wallet_name2 = String::from("wallet2");
        wallet_handle = init_wallet(&wallet_name2).unwrap();
        assert!(wallet_handle > 0);

        thread::sleep(Duration::from_secs(1));
        assert_ne!(handle, get_wallet_handle());
        delete_wallet("wallet2").unwrap();
    }

    #[test]
    fn test_wallet_with_credentials() {
        settings::set_defaults();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE,"false");
        settings::set_config_value(settings::CONFIG_WALLET_KEY,"pass");

        let handle = init_wallet("password_wallet").unwrap();

        SignusUtils::create_and_store_my_did(handle,None).unwrap();
        delete_wallet("password_wallet").unwrap();
    }
}