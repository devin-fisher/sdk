#![allow(dead_code, non_camel_case_types)]
extern crate libc;
extern crate vcx;

use vcx::utils::timeout::TimeoutUtils;
use vcx::utils::libindy::return_types_u32;

use std::ffi::CString;

use self::libc::c_char;

pub type fn_str_r_u32 = extern "C" fn(u32, *const c_char, Option<extern "C" fn(u32, u32)>) -> u32;
pub type fn_str_u32_u32_r_u32_u32 = extern "C" fn(u32, *const c_char, u32, u32, Option<extern "C" fn(u32, u32, u32)>) -> u32;
pub type fn_u32_r_u32 = extern "C" fn(u32, u32, Option<extern "C" fn(u32, u32)>) -> u32;
pub type fn_u32_u32_r_u32 = extern "C" fn(u32, u32, u32, Option<extern "C" fn(u32, u32)>) -> u32;
pub type fn_u32_u32_u32_r_u32 = extern "C" fn(u32, u32, u32, u32, Option<extern "C" fn(u32, u32)>) -> u32;
pub type fn_u32_u32_r_u32_u32_str = extern "C" fn(u32, u32, u32, Option<extern "C" fn(u32, u32, u32, *const c_char)>) -> u32;
pub type fn_u32_r_u32_u32 = extern "C" fn(u32, u32, Option<extern "C" fn(u32, u32, u32)>) -> u32;
pub type fn_str_str_r_u32 = extern "C" fn(u32, *const c_char, *const c_char, Option<extern "C" fn(u32, u32)>) -> u32;
pub type fn_str_str_r_u32_u32 = extern "C" fn(u32, *const c_char, *const c_char, Option<extern "C" fn(u32, u32, u32)>) -> u32;
pub type fn_str_r_u32_u32 = extern "C" fn(u32, *const c_char, Option<extern "C" fn(u32, u32, u32)>) -> u32;
pub type fn_u32_str_r_u32 = extern "C" fn(u32, u32, *const c_char, Option<extern "C" fn(u32, u32)>) -> u32;
pub type fn_u32_str_r_u32_str = extern "C" fn(u32, u32, *const c_char, Option<extern "C" fn(u32, u32, *const c_char)>) -> u32;
pub type fn_u32_r_u32_str = extern "C" fn(u32, u32, Option<extern "C" fn(u32, u32, *const c_char)>) -> u32;
pub type fn_str_u32_str_str_str_r_u32_u32 = extern "C" fn(u32, *const c_char, u32, *const c_char, *const c_char, *const c_char, Option<extern "C" fn(u32, u32, u32)>) -> u32;
pub type fn_str_str_str_str_r_u32_u32 = extern "C" fn(u32, *const c_char, *const c_char, *const c_char, *const c_char, Option<extern "C" fn(u32, u32, u32)>) -> u32;

pub fn str_r_check(arg1: &str, func: fn_str_r_u32) -> Result<(), u32> {
    let rtn_obj = return_types_u32::Return_U32::new()?;

    let arg1 = CString::new(arg1).unwrap();
    let rc = func(rtn_obj.command_handle,
                  arg1.as_ptr(),
                  Some(rtn_obj.get_callback()));

    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_short())
}

pub fn str_r_u32(arg1: &str, func: fn_str_r_u32_u32) -> Result<u32, u32> {
    let rtn_obj = return_types_u32::Return_U32_U32::new()?;

    let arg1 = CString::new(arg1).unwrap();
    let rc = func(rtn_obj.command_handle,
                  arg1.as_ptr(),
                  Some(rtn_obj.get_callback()));

    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_short())
}

pub fn str_u32_u32_r_u32(arg1: &str, arg2: u32, arg3: u32, func: fn_str_u32_u32_r_u32_u32) -> Result<u32, u32> {
    let rtn_obj = return_types_u32::Return_U32_U32::new()?;

    let arg1 = CString::new(arg1).unwrap();
    let rc = func(rtn_obj.command_handle,
                  arg1.as_ptr(),
                  arg2,
                  arg3,
                  Some(rtn_obj.get_callback()));

    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_short())
}

pub fn u32_str_r_u32(arg1: u32, arg2: &str, func: fn_u32_str_r_u32) -> Result<(), u32> {
    let rtn_obj = return_types_u32::Return_U32::new()?;

    let arg2 = CString::new(arg2).unwrap();
    let rc = func(rtn_obj.command_handle, arg1,
                  arg2.as_ptr(),
                  Some(rtn_obj.get_callback()));
    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_long())
}

pub fn u32_str_r_u32_str(arg1: u32, arg2: &str, func: fn_u32_str_r_u32_str) -> Result<Option<String>, u32> {
    let rtn_obj = return_types_u32::Return_U32_STR::new()?;

    let arg2 = CString::new(arg2).unwrap();
    let rc = func(rtn_obj.command_handle, arg1,
                  arg2.as_ptr(),
                  Some(rtn_obj.get_callback()));
    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_long())
}

pub fn str_str_r_check(arg1: &str, arg2: &str, func: fn_str_str_r_u32) -> Result<(), u32> {
    let rtn_obj = return_types_u32::Return_U32::new()?;

    let arg1 = CString::new(arg1).unwrap();
    let arg2 = CString::new(arg2).unwrap();
    let rc = func(rtn_obj.command_handle,
                  arg1.as_ptr(),
                  arg2.as_ptr(),
                  Some(rtn_obj.get_callback()));

    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_short())
}

pub fn str_str_r_u32(arg1: &str, arg2: &str, func: fn_str_str_r_u32_u32) -> Result<u32, u32> {
    let rtn_obj = return_types_u32::Return_U32_U32::new()?;

    let arg1 = CString::new(arg1).unwrap();
    let arg2 = CString::new(arg2).unwrap();
    let rc = func(rtn_obj.command_handle,
                  arg1.as_ptr(),
                  arg2.as_ptr(),
                  Some(rtn_obj.get_callback()));

    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_short())
}

pub fn u32_r_u32(arg1: u32, func: fn_u32_r_u32_u32) -> Result<u32, u32> {
    let rtn_obj = return_types_u32::Return_U32_U32::new()?;

    let rc = func(rtn_obj.command_handle,
                  arg1,
                  Some(rtn_obj.get_callback()));

    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_medium())
}

pub fn str_u32_str_str_str_r_u32(arg1: &str, arg2: u32, arg3: &str, arg4: &str, arg5: &str, func: fn_str_u32_str_str_str_r_u32_u32) -> Result<u32, u32> {
    let rtn_obj = return_types_u32::Return_U32_U32::new()?;

    let arg1 = CString::new(arg1).unwrap();
    let arg3 = CString::new(arg3).unwrap();
    let arg4 = CString::new(arg4).unwrap();
    let arg5 = CString::new(arg5).unwrap();


    let rc = func(rtn_obj.command_handle,
                  arg1.as_ptr(),
                  arg2,
                  arg3.as_ptr(),
                  arg4.as_ptr(),
                  arg5.as_ptr(),
                  Some(rtn_obj.get_callback()));

    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_short())
}


pub fn str_str_str_str_r_u32(arg1: &str, arg2: &str, arg3: &str, arg4: &str, func: fn_str_str_str_str_r_u32_u32) -> Result<u32, u32> {
    let rtn_obj = return_types_u32::Return_U32_U32::new()?;

    let arg1 = CString::new(arg1).unwrap();
    let arg2 = CString::new(arg2).unwrap();
    let arg3 = CString::new(arg3).unwrap();
    let arg4 = CString::new(arg4).unwrap();


    let rc = func(rtn_obj.command_handle,
                  arg1.as_ptr(),
                  arg2.as_ptr(),
                  arg3.as_ptr(),
                  arg4.as_ptr(),
                  Some(rtn_obj.get_callback()));

    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_short())
}


pub fn u32_u32_r_u32(arg1: u32, arg2: u32, func: fn_u32_u32_r_u32) -> Result<(), u32> {
    let rtn_obj = return_types_u32::Return_U32::new()?;

    let rc = func(rtn_obj.command_handle,
                  arg1,
                  arg2,
                  Some(rtn_obj.get_callback()));

    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_medium())
}

pub fn u32_u32_u32_r_u32(arg1: u32, arg2: u32, arg3: u32, func: fn_u32_u32_u32_r_u32) -> Result<(), u32> {
    let rtn_obj = return_types_u32::Return_U32::new()?;

    let rc = func(rtn_obj.command_handle,
                  arg1,
                  arg2,
                  arg3,
                  Some(rtn_obj.get_callback()));

    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_medium())
}

pub fn u32_r_u32_str(arg1: u32, func: fn_u32_r_u32_str) -> Result<Option<String>, u32> {

    let rtn_obj = return_types_u32::Return_U32_STR::new()?;

    let rc = func(rtn_obj.command_handle,
                  arg1,
                  Some(rtn_obj.get_callback()));

    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_medium())
}


pub fn u32_u32_r_u32_str(arg1: u32, arg2: u32, func: fn_u32_u32_r_u32_u32_str) -> Result<(u32, Option<String>), u32> {
    let rtn_obj = return_types_u32::Return_U32_U32_STR::new()?;

    let rc = func(rtn_obj.command_handle,
                  arg1,
                  arg2,
                  Some(rtn_obj.get_callback()));

    if rc != 0 {
        return Err(rc);
    }

    rtn_obj.receive(TimeoutUtils::some_medium())
}