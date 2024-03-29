use std::collections::HashMap;
use std::fmt;

// **** DEFINE NEW ERRORS HERE ****
// STEP 1: create new public static instance of Error, assign it a new unused number and
// give it a human readable error message
// STEP 2: Add Error to the static MAP (used for getting messages to wrappers)
// STEP 3: create a test making sure that your message can be retrieved

pub static SUCCESS: Error = Error{code_num:0, message:"Success"};
pub static UNKNOWN_ERROR: Error = Error{code_num:1001, message:"Unknown Error"};
pub static CONNECTION_ERROR: Error = Error{code_num:1002, message:"Error with Connection"};
pub static INVALID_CONNECTION_HANDLE: Error = Error{code_num:1003, message:"Invalid Connection Handle"};
pub static INVALID_CONFIGURATION: Error = Error{code_num:1004, message:"Invalid Configuration"};
pub static NOT_READY: Error = Error{code_num:1005, message:"Object not ready for specified action"};
pub static NO_ENDPOINT: Error = Error{code_num:1006, message:"No Endpoint set for Connection Object"};
pub static INVALID_OPTION: Error = Error{code_num:1007, message:"Invalid Option"};
pub static INVALID_DID: Error = Error{code_num:1008, message:"Invalid DID"};
pub static INVALID_VERKEY: Error = Error{code_num:1009, message:"Invalid VERKEY"};
pub static POST_MSG_FAILURE: Error = Error{code_num:1010, message:"Message failed in post"};
pub static INVALID_NONCE: Error = Error{code_num:1011, message:"Invalid NONCE"};
pub static INVALID_KEY_DELEGATE: Error = Error{code_num:1012, message:"Invalid DELEGATE"};
pub static INVALID_URL: Error = Error{code_num:1013, message:"Invalid URL"};
pub static NOT_BASE58: Error = Error{code_num:1014, message:"Value needs to be base58"};
pub static INVALID_ISSUER_CLAIM_HANDLE: Error = Error{code_num:1015, message:"Invalid Claim Issuer Handle"};

lazy_static! {
    static ref ERROR_MESSAGES: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        insert_message(&mut m, &SUCCESS);
        insert_message(&mut m, &UNKNOWN_ERROR);
        insert_message(&mut m, &CONNECTION_ERROR);
        insert_message(&mut m, &INVALID_CONNECTION_HANDLE);
        insert_message(&mut m, &INVALID_CONFIGURATION);
        insert_message(&mut m, &INVALID_OPTION);
        insert_message(&mut m, &NOT_READY);
        insert_message(&mut m, &NO_ENDPOINT);
        m
    };


}

// ******* END *******




// Helper function for static defining of error messages. Does limited checking that it can.
fn insert_message(map: &mut HashMap<u32, &'static str>, error: &Error) {
    if map.contains_key(&error.code_num) {
        panic!("Error Code number was repeated which is not allowed! (likely a copy/paste error)")
    }
    map.insert(error.code_num, error.message);

}

pub struct Error {
    pub code_num: u32,
    pub message: &'static str
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = error_message(&self.code_num);
        write!(f, "{}: (Error Num:{})", msg, &self.code_num)
    }
}

/// Finds a static string message for a unique Error code_num. This function allows for finding
/// this message without having the original Error struct.
///
/// Intended for use with wrappers that receive an error code without a message through a
/// c-callable interface.
pub fn error_message(code_num:&u32) -> &'static str {
    match ERROR_MESSAGES.get(code_num) {
        Some(msg) => msg,
        None => UNKNOWN_ERROR.message
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_has_error(){
        let e = &UNKNOWN_ERROR;
        assert_eq!(e.code_num, 1001);
    }

    #[test]
    fn test_display_error(){
        let msg = format!("{}",UNKNOWN_ERROR);
        assert_eq!(msg, "Unknown Error: (Error Num:1001)")
    }

    #[test]
    fn test_error_message(){
        let msg = error_message(&1);
        assert_eq!(msg, "Unknown Error");

        let msg = error_message(&1002);
        assert_eq!(msg, "Error with Connection");
    }

    #[test]
    fn test_unknown_error(){
        assert_eq!(error_message(&UNKNOWN_ERROR.code_num), UNKNOWN_ERROR.message);
    }

    #[test]
    fn test_connection_error(){
        assert_eq!(error_message(&CONNECTION_ERROR.code_num), CONNECTION_ERROR.message);
    }

    #[test]
    fn test_success_error(){
        assert_eq!(error_message(&SUCCESS.code_num), SUCCESS.message);
    }

    #[test]
    fn test_no_endpoint_error(){
        assert_eq!(error_message(&NO_ENDPOINT.code_num), NO_ENDPOINT.message);
    }

    #[test]
    fn test_invalid_option_error(){
        assert_eq!(error_message(&INVALID_OPTION.code_num), INVALID_OPTION.message);
    }
}
