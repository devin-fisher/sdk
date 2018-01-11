extern crate libc;

use self::libc::c_char;
use std::ffi::CString;
use std::env;
use std::fs;
use std::io::Write;
use std::ptr::null;
use std::path::{Path, PathBuf};
use utils::error;
use utils::libindy::{indy_function_eval};
use utils::libindy::return_types::{Return_I32, Return_I32_I32};
use utils::json::JsonEncodable;
use utils::libindy::error_codes::{map_indy_error_code, map_string_error};
use std::sync::RwLock;

lazy_static! {
    static ref POOL_HANDLE: RwLock<Option<i32>> = RwLock::new(None);
}

fn change_pool_handle(handle: Option<i32>){
    let mut h = POOL_HANDLE.write().unwrap();
    *h = handle;
}


#[derive(Serialize, Deserialize)]
struct PoolConfig {
    pub genesis_txn: String
}
impl JsonEncodable for PoolConfig {}

extern {
    fn indy_create_pool_ledger_config(command_handle: i32,
                                      config_name: *const c_char,
                                      config: *const c_char,
                                      cb: Option<extern fn(xcommand_handle: i32, err: i32)>) -> i32;

    fn indy_delete_pool_ledger_config(command_handle: i32,
                                      config_name: *const c_char,
                                      cb: Option<extern fn(xcommand_handle: i32, err: i32)>) -> i32;

    fn indy_open_pool_ledger(command_handle: i32,
                             config_name: *const c_char,
                             config: *const c_char,
                             cb: Option<extern fn(xcommand_handle: i32, err: i32, pool_handle: i32)>) -> i32;

    fn indy_refresh_pool_ledger(command_handle: i32,
                                handle: i32,
                                cb: Option<extern fn(xcommand_handle: i32, err: i32)>) -> i32;

    fn indy_close_pool_ledger(command_handle: i32,
                              handle: i32,
                              cb: Option<extern fn(xcommand_handle: i32, err: i32)>) -> i32;
}

fn test_pool_ip() -> String { env::var("TEST_POOL_IP").unwrap_or("127.0.0.1".to_string()) }

fn tmp_path() -> PathBuf {
    let mut path = env::temp_dir();
    path.push("indy_client");
    path
}

fn tmp_file_path(file_name: &str) -> PathBuf {
    let mut path = tmp_path();
    path.push(file_name);
    path
}

pub fn create_genesis_txn_file(pool_name: &str,
                               txn_file_data: &str,
                               txn_file_path: Option<&Path>) -> PathBuf {
    let txn_file_path = txn_file_path.map_or(
        tmp_file_path(format!("/tmp/{}.txn", pool_name).as_str()),
        |path| path.to_path_buf());

    if !txn_file_path.parent().unwrap().exists() {
        fs::DirBuilder::new()
            .recursive(true)
            .create(txn_file_path.parent().unwrap()).unwrap();
    }

    println!("attempting to create file: {}", txn_file_path.to_string_lossy());
    let mut f = fs::File::create(txn_file_path.as_path()).unwrap();
    f.write_all(txn_file_data.as_bytes()).unwrap();
    f.flush().unwrap();
    f.sync_all().unwrap();

    txn_file_path
}


// Note that to be a valid config, it assumes that the genesis txt file already exists
pub fn pool_config_json(txn_file_path: &Path) -> String {
    PoolConfig {
        genesis_txn: txn_file_path.to_string_lossy().to_string()
    }
        .to_json()
        .unwrap()
}

pub fn create_pool_ledger_config(pool_name: &str, path: Option<&Path>) -> Result<u32, u32> {
    let pool_config = match path {
        Some(c) => pool_config_json(c),
        None => return Err(error::INVALID_GENESIS_TXN_PATH.code_num)
    };

    let pool_name = CString::new(pool_name).map_err(map_string_error)?;
    let pool_config = CString::new(pool_config).map_err(map_string_error)?;

    let rtn_obj = Return_I32::new()?;

    unsafe {
        indy_function_eval(
            indy_create_pool_ledger_config(rtn_obj.command_handle,
                                                 pool_name.as_ptr(),
                                                 pool_config.as_ptr(),
                                                 Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    match rtn_obj.receive() {
        Ok(()) => Ok(0),
        Err(e) => Err(error::CREATE_POOL_CONFIG.code_num)
    }
}

pub fn open_pool_ledger(pool_name: &str, config: Option<&str>) -> Result<u32, u32> {

    let pool_name = CString::new(pool_name).map_err(map_string_error)?;
    let pool_config = match config {
        Some(str) => Some(CString::new(str).map_err(map_string_error)?),
        None => None
    };
    let rtn_obj = Return_I32_I32::new()?;

    unsafe {
        indy_function_eval(indy_open_pool_ledger(rtn_obj.command_handle,
                                pool_name.as_ptr(),
                                match pool_config {
                                    Some(str) => str.as_ptr(),
                                    None => null()
                                },
                                Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    rtn_obj.receive().and_then(|handle|{
        change_pool_handle(Some(handle));
        Ok(handle as u32)
    })
}

pub fn call(pool_handle: i32, func: unsafe extern "C" fn(i32, i32, Option<extern "C" fn(i32, i32)>) -> i32) -> Result<(), u32> {
    let rtn_obj = Return_I32::new()?;
    unsafe {
        indy_function_eval(func(rtn_obj.command_handle,
                                pool_handle,
                                Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    rtn_obj.receive()
}

pub fn refresh(pool_handle: i32) -> Result<(), u32> {
    call(pool_handle, indy_refresh_pool_ledger)
}

pub fn close(pool_handle: i32) -> Result<(), u32> {
    call(pool_handle, indy_close_pool_ledger)
}

pub fn delete(pool_name: &str) -> Result<(), u32> {
    let pool_name = CString::new(pool_name).map_err(map_string_error)?;

    let rtn_obj = Return_I32::new()?;

    unsafe {
        indy_function_eval(
            indy_delete_pool_ledger_config(rtn_obj.command_handle,
                                           pool_name.as_ptr(),
                                           Some(rtn_obj.get_callback()))
        ).map_err(map_indy_error_code)?;
    }

    rtn_obj.receive()
}

pub fn get_pool_handle() -> Result<i32, u32> {
    let h = POOL_HANDLE.read().unwrap();
    if h.is_none() {
        Err(error::NO_POOL_OPEN.code_num)
    }
    else {
        Ok(h.unwrap())
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::{Path, PathBuf};
    use std::env::home_dir;
    use utils::libindy::pool::create_pool_ledger_config;
    use super::*;

    pub fn create_genesis_txn_file_for_test_pool(pool_name: &str,
                                                 nodes_count: Option<u8>,
                                                 txn_file_path: Option<&Path>) -> PathBuf {
        let nodes_count = nodes_count.unwrap_or(4);

//        let test_pool_ip = test_pool_ip();
        let test_pool_ip = "127.0.0.1".to_string();
//        let test_pool_ip = "10.0.0.2".to_string();

//        let node_txns = vec![
//            format!("{{\"data\":{{\"alias\":\"Node1\",\"blskey\":\"4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba\",\"client_ip\":\"{}\",\"client_port\":9702,\"node_ip\":\"{}\",\"node_port\":9701,\"services\":[\"VALIDATOR\"]}},\"dest\":\"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv\",\"identifier\":\"Th7MpTaRZVRYnPiabds81Y\",\"txnId\":\"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62\",\"type\":\"0\"}}", test_pool_ip, test_pool_ip),
//            format!("{{\"data\":{{\"alias\":\"Node2\",\"blskey\":\"37rAPpXVoxzKhz7d9gkUe52XuXryuLXoM6P6LbWDB7LSbG62Lsb33sfG7zqS8TK1MXwuCHj1FKNzVpsnafmqLG1vXN88rt38mNFs9TENzm4QHdBzsvCuoBnPH7rpYYDo9DZNJePaDvRvqJKByCabubJz3XXKbEeshzpz4Ma5QYpJqjk\",\"client_ip\":\"{}\",\"client_port\":9704,\"node_ip\":\"{}\",\"node_port\":9703,\"services\":[\"VALIDATOR\"]}},\"dest\":\"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb\",\"identifier\":\"EbP4aYNeTHL6q385GuVpRV\",\"txnId\":\"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc\",\"type\":\"0\"}}", test_pool_ip, test_pool_ip),
//            format!("{{\"data\":{{\"alias\":\"Node3\",\"blskey\":\"3WFpdbg7C5cnLYZwFZevJqhubkFALBfCBBok15GdrKMUhUjGsk3jV6QKj6MZgEubF7oqCafxNdkm7eswgA4sdKTRc82tLGzZBd6vNqU8dupzup6uYUf32KTHTPQbuUM8Yk4QFXjEf2Usu2TJcNkdgpyeUSX42u5LqdDDpNSWUK5deC5\",\"client_ip\":\"{}\",\"client_port\":9706,\"node_ip\":\"{}\",\"node_port\":9705,\"services\":[\"VALIDATOR\"]}},\"dest\":\"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya\",\"identifier\":\"4cU41vWW82ArfxJxHkzXPG\",\"txnId\":\"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4\",\"type\":\"0\"}}", test_pool_ip, test_pool_ip),
//            format!("{{\"data\":{{\"alias\":\"Node4\",\"blskey\":\"2zN3bHM1m4rLz54MJHYSwvqzPchYp8jkHswveCLAEJVcX6Mm1wHQD1SkPYMzUDTZvWvhuE6VNAkK3KxVeEmsanSmvjVkReDeBEMxeDaayjcZjFGPydyey1qxBHmTvAnBKoPydvuTAqx5f7YNNRAdeLmUi99gERUU7TD8KfAa6MpQ9bw\",\"client_ip\":\"{}\",\"client_port\":9708,\"node_ip\":\"{}\",\"node_port\":9707,\"services\":[\"VALIDATOR\"]}},\"dest\":\"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA\",\"identifier\":\"TWwCRQRZ2ZHMJFn9TzLp7W\",\"txnId\":\"aa5e817d7cc626170eca175822029339a444eb0ee8f0bd20d3b0b76e566fb008\",\"type\":\"0\"}}", test_pool_ip, test_pool_ip)];
        let node_txns = vec![
            format!("{{\"data\":{{\"alias\":\"Node1\",\"client_ip\":\"{}\",\"client_port\":9702,\"node_ip\":\"{}\",\"node_port\":9701,\"services\":[\"VALIDATOR\"]}},\"dest\":\"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv\",\"identifier\":\"Th7MpTaRZVRYnPiabds81Y\",\"txnId\":\"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62\",\"type\":\"0\"}}", test_pool_ip, test_pool_ip),
            format!("{{\"data\":{{\"alias\":\"Node2\",\"client_ip\":\"{}\",\"client_port\":9704,\"node_ip\":\"{}\",\"node_port\":9703,\"services\":[\"VALIDATOR\"]}},\"dest\":\"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb\",\"identifier\":\"EbP4aYNeTHL6q385GuVpRV\",\"txnId\":\"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc\",\"type\":\"0\"}}", test_pool_ip, test_pool_ip),
            format!("{{\"data\":{{\"alias\":\"Node3\",\"client_ip\":\"{}\",\"client_port\":9706,\"node_ip\":\"{}\",\"node_port\":9705,\"services\":[\"VALIDATOR\"]}},\"dest\":\"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya\",\"identifier\":\"4cU41vWW82ArfxJxHkzXPG\",\"txnId\":\"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4\",\"type\":\"0\"}}", test_pool_ip, test_pool_ip),
            format!("{{\"data\":{{\"alias\":\"Node4\",\"client_ip\":\"{}\",\"client_port\":9708,\"node_ip\":\"{}\",\"node_port\":9707,\"services\":[\"VALIDATOR\"]}},\"dest\":\"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA\",\"identifier\":\"TWwCRQRZ2ZHMJFn9TzLp7W\",\"txnId\":\"aa5e817d7cc626170eca175822029339a444eb0ee8f0bd20d3b0b76e566fb008\",\"type\":\"0\"}}", test_pool_ip, test_pool_ip)];


        let txn_file_data = node_txns[0..(nodes_count as usize)].join("\n");

        create_genesis_txn_file(pool_name, txn_file_data.as_str(), txn_file_path)
    }

    fn clean_pools(slice: &[String]) {
        ::utils::logger::LoggerUtils::init();
        let home = home_dir().unwrap().as_path().to_str().unwrap().to_owned();
        let temp_path = format!("{}/{}",home, ".indy_client/pool/");
        assert!(temp_path.len() > "./indy_client/pool/".to_string().len());
        use std::fs;
        use std::path::Path;
        for p in slice{
            let pp = format!("{}{}",temp_path, p);
            let path = Path::new(&pp);
            if path.exists(){
                match fs::remove_dir_all(path){
                    Ok(_) => info!("Removed {:?}", path),
                    Err(_)=> info!("Failed to remove {:?}", path),
                };
            }
        }

    }
    #[test]
    fn test_create_pool_ledger_config() {
        let pool1 = "Pool1".to_string();
        let pool2 = "Pool2".to_string();
        let pool3 = "Pool3".to_string();
        let pools = [pool1, pool2, pool3];
        clean_pools(&pools);
        let path = create_genesis_txn_file_for_test_pool(&pools[0], None, None);
        let config_string = format!("{{\"genesis_txn\":\"/tmp/{}.txn\"}}", &pools[0]);
        let incorrect_path = Path::new(r#"{"genesis_txn":this is missing quotes}"#);
        assert_eq!(pool_config_json(&path),config_string);
        assert_eq!(create_pool_ledger_config(&pools[0], Some(&path)),Ok(error::SUCCESS.code_num));
        assert_eq!(create_pool_ledger_config(&pools[1], Some(&incorrect_path)),Err(error::CREATE_POOL_CONFIG.code_num));
        assert_eq!(create_pool_ledger_config(&pools[2], None), Err(error::INVALID_GENESIS_TXN_PATH.code_num));
    }

//    #[test]
//    fn test_open_pool() {
//        let pool1 = "Pool1".to_string();
//        let pool2 = "Pool2".to_string();
//        let pool3 = "Pool3".to_string();
//        let pools = [pool1, pool2, pool3];
//        clean_pools(&pools);
//        let config = r#"{"refresh_on_open": true}"#;
//        let path = create_genesis_txn_file_for_test_pool(&pools[0], None, None);
//        let config_string = format!("{{\"genesis_txn\":\"/tmp/{}.txn\"}}", &pools[0]);
//        assert_eq!(pool_config_json(&path),config_string);
//        assert_eq!(create_pool_ledger_config(&pools[0], Some(&path)),Ok(error::SUCCESS.code_num));
//        assert_eq!(create_pool_ledger_config(&pools[1], Some(&path)),Ok(error::SUCCESS.code_num));
//        assert_eq!(create_pool_ledger_config(&pools[2], Some(&path)), Ok(error::SUCCESS.code_num));
//
//        let pool_handle = open_pool_ledger(&pools[0], Some(config)).unwrap();
//        assert_ne!(pool_handle, 0);
//    }
}
