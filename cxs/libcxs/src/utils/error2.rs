use std::collections::HashMap;
use std::fmt;
use std::result;
use std::string::String;

pub type CxsResult<T> = result::Result<T, u32>;

// **** DEFINE NEW ERRORS HERE ****
// STEP 1: create new public static instance of Error, assign it a new unused number and
// give it a human readable error message
// STEP 2: Add Error to the static MAP (used for getting messages to wrappers)
// STEP 3: create a test making sure that your message can be retrieved

#[allow(non_camel_case_types)]
pub enum Error2 {
    SUCCESS,
    UNKNOWN_ERROR,
    CONNECTION_ERROR,
    INVALID_CONNECTION_HANDLE,
    INVALID_CONFIGURATION,
    NOT_READY,
    NO_ENDPOINT,
    INVALID_OPTION,
    INVALID_DID,
    INVALID_VERKEY,
    POST_MSG_FAILURE,
    INVALID_NONCE,
    INVALID_KEY_DELEGATE,
    INVALID_URL,
    NOT_BASE58,
    INVALID_ISSUER_CLAIM_HANDLE,
    INVALID_JSON,
    INVALID_PROOF_HANDLE,
    INVALID_CLAIM_REQUEST,
}

static ERROR_LIST: [Error2; 19] = [
    Error2::SUCCESS,
    Error2::UNKNOWN_ERROR,
    Error2::CONNECTION_ERROR,
    Error2::INVALID_CONNECTION_HANDLE,
    Error2::INVALID_CONFIGURATION,
    Error2::NOT_READY,
    Error2::NO_ENDPOINT,
    Error2::INVALID_OPTION,
    Error2::INVALID_DID,
    Error2::INVALID_VERKEY,
    Error2::POST_MSG_FAILURE,
    Error2::INVALID_NONCE,
    Error2::INVALID_KEY_DELEGATE,
    Error2::INVALID_URL,
    Error2::NOT_BASE58,
    Error2::INVALID_ISSUER_CLAIM_HANDLE,
    Error2::INVALID_JSON,
    Error2::INVALID_PROOF_HANDLE,
    Error2::INVALID_CLAIM_REQUEST,
];

impl Error2 {
    pub fn message(&self) -> &'static str
    {
        match *self {
            Error2::SUCCESS => "Success",
            Error2::UNKNOWN_ERROR => "Unknown Error",
            Error2::CONNECTION_ERROR => "Error with Connection",
            Error2::INVALID_CONNECTION_HANDLE => "Invalid Connection Handle",
            Error2::INVALID_CONFIGURATION => "Invalid Configuration",
            Error2::NOT_READY => "Object not ready for specified action",
            Error2::NO_ENDPOINT => "No Endpoint set for Connection Object",
            Error2::INVALID_OPTION => "Invalid Option",
            Error2::INVALID_DID => "Invalid DID",
            Error2::INVALID_VERKEY => "Invalid VERKEY",
            Error2::POST_MSG_FAILURE => "Message failed in post",
            Error2::INVALID_NONCE => "Invalid NONCE",
            Error2::INVALID_KEY_DELEGATE => "Invalid DELEGATE",
            Error2::INVALID_URL => "Invalid URL",
            Error2::NOT_BASE58 => "Value needs to be base58",
            Error2::INVALID_ISSUER_CLAIM_HANDLE => "Invalid Claim Issuer Handle",
            Error2::INVALID_JSON => "Invalid JSON string",
            Error2::INVALID_PROOF_HANDLE => "Invalid Proof Handle",
            Error2::INVALID_CLAIM_REQUEST => "Invalid Claim Request",
        }
    }

    pub fn code_num(&self) -> u32
    {
        match *self {
            Error2::SUCCESS => 00,
            Error2::UNKNOWN_ERROR => 1001,
            Error2::CONNECTION_ERROR => 1002,
            Error2::INVALID_CONNECTION_HANDLE => 1003,
            Error2::INVALID_CONFIGURATION => 1004,
            Error2::NOT_READY => 1005,
            Error2::NO_ENDPOINT => 1006,
            Error2::INVALID_OPTION => 1007,
            Error2::INVALID_DID => 1008,
            Error2::INVALID_VERKEY => 1009,
            Error2::POST_MSG_FAILURE => 1010,
            Error2::INVALID_NONCE => 1011,
            Error2::INVALID_KEY_DELEGATE => 1012,
            Error2::INVALID_URL => 1013,
            Error2::NOT_BASE58 => 1014,
            Error2::INVALID_ISSUER_CLAIM_HANDLE => 1015,
            Error2::INVALID_JSON => 1016,
            Error2::INVALID_PROOF_HANDLE => 1017,
            Error2::INVALID_CLAIM_REQUEST => 1018,
        }
    }

    fn desc(&self) -> String{
        format!("{}: (Error Num:{})", self.message(),  self.code_num())
    }
}

impl fmt::Display for Error2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.desc())
    }
}

impl fmt::Debug for Error2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.desc())
    }
}

lazy_static! {

    static ref ERROR_MESSAGES: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        for error in ERROR_LIST.into_iter() {
            insert_message(&mut m, error);
        }
        m
    };
}


// Helper function for static defining of error messages. Does limited checking that it can.
fn insert_message(map: &mut HashMap<u32, &'static str>, error: &Error2) {
    if map.contains_key(&error.code_num()) {
        panic!("Error Code number was repeated which is not allowed! (likely a copy/paste error)")
    }
    map.insert(error.code_num(), error.message());

}


/// Finds a static string message for a unique Error code_num. This function allows for finding
/// this message without having the original Error struct.
///
/// Intended for use with wrappers that receive an error code without a message through a
/// c-callable interface.
pub fn error_message(code_num:u32) -> &'static str {
    match ERROR_MESSAGES.get(&code_num) {
        Some(msg) => msg,
        None => Error2::UNKNOWN_ERROR.message()
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_error(){
        let e = Error2::UNKNOWN_ERROR;
        assert_eq!(e.code_num(), 1001);
    }

    #[test]
    fn test_display_error(){
        let msg = format!("{}",Error2::UNKNOWN_ERROR);
        assert_eq!(msg, "Unknown Error: (Error Num:1001)")
    }

    #[test]
    fn test_error_message(){
        let msg = error_message(1);
        assert_eq!(msg, "Unknown Error");

        let msg = error_message(1002);
        assert_eq!(msg, "Error with Connection");
    }

    #[test]
    fn test_unknown_error(){
        assert_eq!(error_message(Error2::UNKNOWN_ERROR.code_num()), Error2::UNKNOWN_ERROR.message());
    }

    #[test]
    fn test_connection_error(){
        assert_eq!(error_message(Error2::CONNECTION_ERROR.code_num()), Error2::CONNECTION_ERROR.message());
    }

    #[test]
    fn test_success_error(){
        assert_eq!(error_message(Error2::SUCCESS.code_num()), Error2::SUCCESS.message());
    }

    #[test]
    fn test_no_endpoint_error(){
        assert_eq!(error_message(Error2::NO_ENDPOINT.code_num()), Error2::NO_ENDPOINT.message());
    }

    #[test]
    fn test_invalid_option_error(){
        assert_eq!(error_message(Error2::INVALID_OPTION.code_num()), Error2::INVALID_OPTION.message());
    }
}
