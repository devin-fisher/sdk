#![allow(dead_code, non_camel_case_types)]
extern crate libc;

use std::ffi::CString;
use self::libc::c_char;

#[allow(dead_code, unused_variables, unused_assignments)]
pub extern "C" fn generic_cb(command_handle:u32, err:u32) {
    if err != 0 {panic!("api error: {}", err)}
}


type fn_str_r_u32 = extern "C" fn(u32, *const c_char, Option<extern "C" fn(u32, u32)>) -> u32;

pub fn str_r_panic(arg1: &str, func: fn_str_r_u32) {
    let arg1 = CString::new(arg1).unwrap();

    let rc = func(0,
                  arg1.as_ptr(),
                  Some(generic_cb));

    assert_eq!(rc, 0);
}

//pub fn str_r_u32(arg1: &str, func: fn_str_r_u32) -> Result<(), u32> {
////    let rtn_obj = Return_I32::new()?;
////    unsafe {
////        indy_function_eval(func(rtn_obj.command_handle,
////                                arg1,
////                                Some(rtn_obj.get_callback()))
////        ).map_err(map_indy_error_code)?;
////    }
////
////    rtn_obj.receive()
//}