extern crate rand;
extern crate serde_json;
extern crate libc;
extern crate serde;
extern crate rmp_serde;

use object_cache::ObjectCache;
use utils::error;

use std::sync::Mutex;

use utils::libindy::wallet;
use utils::libindy::sss;

use settings;

use rand::Rng;

lazy_static! {
    static ref HANDLE_MAP: ObjectCache<RecoveryShares>  = Default::default();
}

const LINK_SECRET_ALIAS: &str = "main";

#[derive(Serialize, Deserialize, Debug)]
struct Counter {
    count: u8
}

impl Counter {
    fn new() -> Counter {
        Counter{
            count: 0,
        }
    }

    fn next(&mut self) -> u8 {
        self.count = self.count + 1;
        self.count
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecoveryShares {

    source_id: Option<String>,
    verkey: String,
    msg: String,
    share_count: u8,
    consumed: Mutex<Counter>,
    threshold: u8,
    shares: Mutex<Vec<String>>,
}

impl RecoveryShares {
    fn new(source_id: Option<String>, share_count: u8, threshold: u8) -> Result<RecoveryShares, u32> {
        let w_h = wallet::get_wallet_handle();

        let recov_verkey = settings::get_config_value(settings::CONFIG_RECOVERY_VERKEY)?;

        let share_json = sss::libindy_shard_msg_with_secret_and_store_shards(w_h,
                                                                             threshold as u8,
                                                                             share_count as u8,
                                                                             "{}",
                                                                             &recov_verkey
        )?;

        let mut shares = vec![];

        for i in 0..share_count{
            let rstr: String = rand::thread_rng()
                .gen_ascii_chars()
                .take(64)
                .collect();
            shares.push(rstr)
        }

        Ok(RecoveryShares{
            source_id,
            verkey: recov_verkey,
            msg: String::from("{}"),
            share_count,
            consumed: Mutex::new(Counter::new()),
            threshold,
            shares: Mutex::new(shares),
        })

    }

    fn consume_share(&mut self) -> Result<String, u32> {
        let share_num = self.consumed.lock().unwrap().next();

        let w_h = wallet::get_wallet_handle();

        let rtn = sss::libindy_get_shard_of_verkey(w_h, &self.verkey, share_num)?;
        Ok(rtn)
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
    let new: RecoveryShares = RecoveryShares::new(source_id, 3,2)?;


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
    use settings;
    use utils::libindy::crypto;

    #[test]
    fn noop(){
    }

    #[test]
    fn secret_gen_test() {
        const RECOVERY_TEST_W: &str = "RECOVERY_TEST_W";
        ::utils::logger::LoggerUtils::init();
        settings::set_defaults();
        wallet::init_wallet(RECOVERY_TEST_W).unwrap();

        let w_h = wallet::get_wallet_handle();

        let recovery_key = crypto::libindy_create_key(w_h,
                                                      r#"{"seed":"00000000000000000000000000000GEN" }"#
        ).unwrap();

        settings::set_config_value(settings::CONFIG_RECOVERY_VERKEY, &recovery_key);

        let mut recovery = RecoveryShares::new(Some(String::from("")),
                                               10,
                                               2).unwrap();

        let share1 = recovery.consume_share().unwrap();
        let share2 = recovery.consume_share().unwrap();
        let share3 = recovery.consume_share().unwrap();

        let recover_json = json!([{"value":share1},{"value": share2}, {"value": share3}]);
        let recover_json = serde_json::to_string_pretty(&recover_json).unwrap();

        println!("{}", recover_json);

        let secret = sss::libindy_recover_secret_from_shards(&recover_json).unwrap();

        println!("verkey:{}", recovery_key);
        println!("SECRET:{}", secret);

        wallet::delete_wallet(RECOVERY_TEST_W).unwrap();
    }
}
