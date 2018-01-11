extern crate libc;

use self::libc::c_char;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use utils::libindy::next_command_handle;
use utils::libindy::callback;
use utils::libindy::error_codes::map_indy_error;
use utils::timeout::TimeoutUtils;
use utils::error;
use std::sync::mpsc::channel;
use std::fmt::Display;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Mutex;
use std::ops::Deref;

fn log_error<T: Display>(e: T) {
    warn!("Unable to send through libindy callback in cxs: {}", e);
}

fn insert_closure<T>(closure: T, map: &Mutex<HashMap<i32, T>>) -> i32 {
    let command_handle = next_command_handle();
    {
        let mut callbacks = map.lock().expect(callback::POISON_MSG);
        callbacks.insert(command_handle, closure);
    }
    command_handle
}

fn receive<T>(receiver: &Receiver<T>, timeout: Option<Duration>) -> Result<T, u32>{
    let timeout_val = timeout.unwrap_or(TimeoutUtils::medium_timeout());

    match receiver.recv_timeout(timeout_val) {
        Ok(t) => Ok(t),
        Err(e) => match e {
            RecvTimeoutError::Timeout => {
                warn!("Timed Out waiting for libindy to call back");
                Err(error::TIMEOUT_LIBINDY_ERROR.code_num)
            },
            RecvTimeoutError::Disconnected => {
                warn!("Channel to libindy was disconnected unexpectedly");
                Err(error::TIMEOUT_LIBINDY_ERROR.code_num)
            }
        }
    }
}

#[allow(non_camel_case_types)]
pub struct Return_I32 {
    pub command_handle: i32,
    receiver: Receiver<i32>,
}

impl Return_I32 {
    pub fn new() -> Result<Return_I32, u32> {
        let (sender, receiver) = channel();
        let closure: Box<FnMut(i32) + Send> = Box::new(move |err | {
            sender.send(err).unwrap_or_else(log_error);
        });

        let command_handle = insert_closure(closure, callback::CALLBACKS_I32.deref());

        Ok(Return_I32 {
            command_handle,
            receiver,
        })
    }

    pub fn get_callback(&self) -> extern fn(command_handle: i32, arg1: i32) {
        callback::call_cb_i32
    }

    pub fn receive(&self, timeout: Option<Duration>) -> Result<(), u32> {
        let err = receive(&self.receiver, timeout)?;

        map_indy_error((), err)
    }
}

#[allow(non_camel_case_types)]
pub struct Return_I32_I32 {
    pub command_handle: i32,
    receiver: Receiver<(i32, i32)>,
}
impl Return_I32_I32 {
    pub fn new() -> Result<Return_I32_I32, u32> {
        let (sender, receiver) = channel();
        let closure: Box<FnMut(i32, i32) + Send> = Box::new(move |err, arg1 | {
            sender.send((err, arg1)).unwrap_or_else(log_error);
        });

        let command_handle = insert_closure(closure, callback::CALLBACKS_I32_I32.deref());

        Ok(Return_I32_I32 {
            command_handle,
            receiver,
        })
    }

    pub fn get_callback(&self) -> extern fn (command_handle: i32, arg1: i32, arg2: i32) {
        callback::call_cb_i32_i32
    }

    pub fn receive(&self, timeout: Option<Duration>) -> Result<i32, u32> {
        let (err, arg1) = receive(&self.receiver, timeout)?;

        map_indy_error(arg1, err)
    }
}


#[allow(non_camel_case_types)]
pub struct Return_I32_STR {
    pub command_handle: i32,
    receiver: Receiver<(i32, Option<String>)>,
}
impl Return_I32_STR {
    pub fn new() -> Result<Return_I32_STR, u32> {
        let (sender, receiver) = channel();
        let closure:Box<FnMut(i32, Option<String>) + Send> = Box::new(move |err, str | {
            sender.send((err, str)).unwrap_or_else(log_error);
        });

        let command_handle = insert_closure(closure, callback::CALLBACKS_I32_STR.deref());

        Ok(Return_I32_STR {
            command_handle,
            receiver,
        })
    }

    pub fn get_callback(&self) -> extern fn(command_handle: i32, arg1: i32, arg2: *const c_char) {
        callback::call_cb_i32_str
    }

    pub fn receive(&self, timeout: Option<Duration>) -> Result<Option<String>, u32> {
        let (err, str1) = receive(&self.receiver, timeout)?;

        map_indy_error(str1, err)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    fn cstring(str_val: &String) -> CString {
        CString::new(str_val.clone()).unwrap()
    }

    #[test]
    fn test_return_i32() {
        let rtn = Return_I32::new().unwrap();
        rtn.get_callback()(rtn.command_handle, 0);
        let val = rtn.receive(None);
        assert!(val.is_ok());

        let rtn = Return_I32::new().unwrap();
        rtn.get_callback()(rtn.command_handle, 123);
        let val = rtn.receive(None);
        assert!(val.is_err());
    }

    #[test]
    fn test_return_i32_i32() {
        let test_val = 23455;

        let rtn = Return_I32_I32::new().unwrap();
        rtn.get_callback()(rtn.command_handle, 0, test_val);
        let val = rtn.receive(None);
        assert!(val.is_ok());
        assert_eq!(val.unwrap(), test_val);

        let rtn = Return_I32_I32::new().unwrap();
        rtn.get_callback()(rtn.command_handle, 123, test_val);
        let val = rtn.receive(None);
        assert!(val.is_err());
    }

    #[test]
    fn test_return_i32_str() {
        let test_str = "Journey before destination".to_string();

        let rtn = Return_I32_STR::new().unwrap();
        rtn.get_callback()(rtn.command_handle, 0, cstring(&test_str).as_ptr());
        let val = rtn.receive(None);
        assert!(val.is_ok());
        assert_eq!(val.unwrap(), Some(test_str.clone()));

        let rtn = Return_I32_STR::new().unwrap();
        rtn.get_callback()(rtn.command_handle, 0, ptr::null());
        let val = rtn.receive(None);
        assert!(val.is_ok());
        assert_eq!(val.unwrap(), None);

        let rtn = Return_I32_STR::new().unwrap();
        rtn.get_callback()(rtn.command_handle, 123, cstring(&test_str).as_ptr());
        let val = rtn.receive(None);
        assert!(val.is_err());
    }

}