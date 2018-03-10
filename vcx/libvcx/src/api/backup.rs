extern crate libc;

use self::libc::c_char;
use utils::cstring::CStringUtils;
use utils::error;
use std::thread;

use backup;

///
#[no_mangle]
pub extern fn vcx_backup_do_backup(command_handle: u32,
                                   file_list: *const c_char,
                                   cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    check_useful_c_str!(file_list, error::INVALID_OPTION.code_num);



    thread::spawn(move|| {
        let err = match backup::do_backup(&file_list) {
            Ok(x) => x,
            Err(x) => x,
        };

        cb(command_handle,err);
    });

    error::SUCCESS.code_num
}

///
#[no_mangle]
pub extern fn vcx_backup_do_restore(command_handle: u32,
                                   share_handles: *const c_char,
                                   cb: Option<extern fn(xcommand_handle: u32, err: u32)>) -> u32 {

    check_useful_c_callback!(cb, error::INVALID_OPTION.code_num);

    check_useful_c_str!(share_handles, error::INVALID_OPTION.code_num);

    thread::spawn(move|| {
        let err = match backup::do_restore(&share_handles) {
            Ok(x) => x,
            Err(x) => x,
        };

        cb(command_handle,err);
    });

    error::SUCCESS.code_num
}