extern crate rand;
extern crate serde_json;
extern crate libc;
extern crate serde;
extern crate rmp_serde;

use object_cache::ObjectCache;
use utils::error;


//use utils::libindy::wallet;
//use utils::libindy::crypto;


use trust_pong::_value_from_json;

use utils::option_util::expect_ok_or;

use std::sync::Mutex;


use serde_json::Value;
use rand::Rng;

lazy_static! {
    static ref HANDLE_MAP: ObjectCache<RecoveryShares>  = Default::default();
}

const LINK_SECRET_ALIAS: &str = "main";

#[derive(Serialize, Deserialize, Debug)]
pub struct RecoveryShares {

    source_id: Option<String>,
    shares: Mutex<Vec<String>>,
}

impl RecoveryShares {
    fn new(source_id: Option<String>, share_count: u32, threshold: u32) -> RecoveryShares {
        let mut shares = vec![];

        for i in 0..share_count{
            let rstr: String = rand::thread_rng()
                .gen_ascii_chars()
                .take(64)
                .collect();
            shares.push(rstr)
        }

        RecoveryShares{
            source_id,
            shares: Mutex::new(shares),
        }

    }

    fn consume_share(&mut self) -> Result<String, u32> {
        let mut shares = self.shares.lock().unwrap();
        match shares.pop() {
            Some(s) => Ok(s),
            None => Err(10) // No shares left
        }
    }
}

//********************************************
//         HANDLE FUNCTIONS
//********************************************
fn handle_err(code_num: u32) -> u32 {
    if code_num == error::INVALID_OBJ_HANDLE.code_num {
        error::INVALID_OBJ_HANDLE.code_num // TODO make a error
    }
        else {
            code_num
        }
}

pub fn create(source_id: Option<String>, count: u32, threshold:u32) -> Result<u32, u32> {
    let new: RecoveryShares = RecoveryShares::new(source_id, 3,2);


    info!("inserting recovery_shares into handle map");
    Ok(HANDLE_MAP.add(new)?)
}

pub fn consume_share(handle: u32) -> Result<String, u32> {
    HANDLE_MAP.get_mut(handle, |obj|{
        obj.consume_share()
    })
}

pub fn release(handle: u32) -> Result<(), u32> {
    HANDLE_MAP.release(handle).map_err(handle_err)
}

pub fn is_valid_handle(handle: u32) -> bool {
    HANDLE_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> Result<String, u32> {
    HANDLE_MAP.get(handle, |obj|{
        serde_json::to_string(&obj).map_err(|e|{
            warn!("Unable to serialize: {:?}", e);
            error::SERIALIZATION_ERROR.code_num
        })
    })
}

pub fn from_string(data: &str) -> Result<u32, u32> {
    let new: RecoveryShares = match serde_json::from_str(data) {
        Ok(x) => x,
        Err(y) => return Err(error::INVALID_JSON.code_num),
    };

    let new_handle = HANDLE_MAP.add(new)?;

    info!("inserting handle {} into proof table", new_handle);

    Ok(new_handle)
}


#[cfg(test)]
mod tests {
    extern crate serde_json;

    use super::*;

    #[test]
    fn noop(){
    }

    #[test]
    fn consume_test(){
        let count = 10;
        let recovery = RecoveryShares::new(Some(String::from("")), 10, 3);
        println!("{}", serde_json::to_string_pretty(&recovery).unwrap());

        let shares = recovery.shares.lock().unwrap();
        assert_eq!(count, shares.len())
    }
}
