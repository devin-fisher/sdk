extern crate uuid;
extern crate serde_json;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate chrono;

use self::rusoto_core::Region;
use self::rusoto_s3::{S3Client, PutObjectRequest, GetObjectRequest};
use self::rusoto_core::default_tls_client;
use self::rusoto_core::ProvideAwsCredentials;
use self::rusoto_core::AwsCredentials;
use self::rusoto_core::CredentialsError;
use self::rusoto_s3::S3;

use self::chrono::{DateTime};
use std::str::FromStr;

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;


use settings;

use utils::error;

use request_share;

use serde_json::Value;
use utils::libindy;

const S3_BUCKET:&'static str = "dkms-dhs-demo";

#[derive(Serialize, Deserialize, Debug)]
struct ManifestEntry {
    file_path: String,
    s3_key: String,
    s3_bucket: String
}

#[derive(Serialize, Deserialize, Debug)]
struct BackupManifest {
    entries: Vec<ManifestEntry>
}

impl BackupManifest {
    fn add_entry(&mut self, entry: ManifestEntry) {
        self.entries.push(entry)
    }
}

impl Default for BackupManifest {
    fn default() -> BackupManifest
    {
        BackupManifest {
            entries: Vec::new(),
        }
    }
}

struct CustomeCredentialsProvider {

}

impl CustomeCredentialsProvider {
    fn new() -> CustomeCredentialsProvider {
        CustomeCredentialsProvider {}
    }

}

impl ProvideAwsCredentials for CustomeCredentialsProvider {
    fn credentials(&self) -> Result<AwsCredentials, CredentialsError> {
        let rtn = AwsCredentials::new("AKIAJPTTLG4U2OOUZQAA",
                                      "j5grd02baASEkD/+xy5CbJHKzgsAGNyn/GTSPVV7",
                                      None,
                                      DateTime::from_str("2018-03-07T16:45:01.478720166Z").unwrap());
        Ok(rtn)
    }
}

fn _prep_file(file_path: &str, verkey: &str) -> Result<Vec<u8>, u32> {
    let file_path = Path::new(file_path);

    let mut f = File::open(file_path).map_err(|e| {
        error!("Unable to open file '{:?}' -- {:?}", file_path, e);
        e
    }).or(Err(101 as u32))?;

    let mut rtn: Vec<u8> = Default::default();

    f.read_to_end(&mut rtn).map_err(|e|{
        error!("Unable to read file '{:?}' -- {:?}", file_path, e);
        e
    }).or(Err(102 as u32))?;

    let rtn = libindy::crypto::prep_anonymous_msg(verkey, &rtn[..])?; //Encrypt !!
    Ok(rtn)
}

fn _send_to_s3(data: Vec<u8>, entry: &ManifestEntry) -> Result<(), u32> {
    let client = S3Client::new(default_tls_client().or(Err(103 as u32))?,
                                   CustomeCredentialsProvider::new(),
                                   Region::UsWest2);

    let mut put_request = PutObjectRequest::default();
    put_request.key = entry.s3_key.to_owned();
    put_request.bucket = entry.s3_bucket.to_owned();
    put_request.body = Some(data);
    let result = client.put_object(&put_request).map_err(|e| {
        error!("Unable to send data to s3 '{:?}' -- {:?}", entry, e);
        e
    }).or(Err(106 as u32))?;

    Ok(())
}

fn _backup_indtity_file(file_path: &str, verkey: &str) -> Result<ManifestEntry, u32> {
    let s3_key = uuid::Uuid::new_v4();
    let s3_key = s3_key.to_string();
    let rtn = ManifestEntry{
        file_path: file_path.to_owned(),
        s3_key,
        s3_bucket: String::from(S3_BUCKET),
    };

    _send_to_s3(_prep_file(file_path, verkey)?, &rtn)?;

    Ok(rtn)
}

fn _backup_manifest(manifest: BackupManifest, verkey: &str) -> Result<(), u32> {
    let entry = ManifestEntry{
        file_path: String::new(),
        s3_key: verkey.to_owned(),
        s3_bucket: String::from(S3_BUCKET),
    };
    let data = serde_json::to_vec(&manifest).or(Err(104 as u32))?;
    let data = libindy::crypto::prep_anonymous_msg(verkey, &data[..])?; //ENCRYPT

    _send_to_s3(data, &entry)
}


fn backup_identity_files(file_list: Vec<String>, verkey: &str) -> Result<(), u32> {

    let mut manifest = BackupManifest::default();
    for file in file_list {
        let entry = _backup_indtity_file(&file, verkey)?;
        manifest.add_entry(entry);
        _backup_indtity_file(&file, verkey)?;
    }

    _backup_manifest(manifest, verkey)?;
    Ok(())
}

pub fn do_backup(file_list_json: &str) -> Result<u32, u32> {
    let backup_verkey = settings::get_config_value(settings::CONFIG_RECOVERY_VERKEY)?;

    let file_list: Vec<String> = serde_json::from_str(file_list_json)
        .map_err(|e| {
            error!("JSON with file list is not a list of Strings or is not valid JSON -- {}", file_list_json);
            e
        }).or(Err(error::INVALID_JSON.code_num))?;

    if file_list.len() == 0 {
        warn!("Files list to backup is empty");
        return Ok(error::SUCCESS.code_num);
    }

    match settings::test_indy_mode_enabled() {
        false => {
            backup_identity_files(file_list, &backup_verkey)?;
        },
        true => {
            info!("Backup not sent to S3 in test mode");
            info!("files to backup are: {}", file_list_json);
        }
    };

    Ok(error::SUCCESS.code_num)
}

pub fn do_restore(request_shares_handles: &str) -> Result<u32, u32> {
    let handles: Vec<u32> = serde_json::from_str(request_shares_handles).or(Err(error::INVALID_JSON.code_num))?;

    info!("Recovery on these share handles: {:?}", handles);

    let mut shares: Vec<Value> = Vec::default();
    for handle in handles {
        let share = request_share::get_share_val(handle)?;


        let share_json = json!({"value": share});
        shares.push(share_json);
    }

    let shares =  serde_json::to_string_pretty(&shares).unwrap();

    let w_h = libindy::wallet::get_wallet_handle();
    let verkey = _recover_key(&shares, w_h)?;

    _restore_files(&verkey, w_h)?;
    Ok(error::SUCCESS.code_num)
}

fn _decrypt_bytes(verkey: &str, wallet_handle: i32, data: &Vec<u8>) -> Result<Vec<u8>, u32> {
    libindy::crypto::parse_msg(wallet_handle, verkey, &data[..])
}

fn _retrieve_file(entry: &ManifestEntry) -> Result<Vec<u8>, u32> {
    let client = S3Client::new(default_tls_client().or(Err(103 as u32))?,
                               CustomeCredentialsProvider::new(),
                               Region::UsWest2);

    let mut put_request = GetObjectRequest::default();

    put_request.key = entry.s3_key.to_owned();
    put_request.bucket = entry.s3_bucket.to_owned();

    let result = client.get_object(&put_request).unwrap();
    let mut rtn = Vec::default();
    let mut out = result.body.unwrap();
    out.read_to_end(&mut rtn).unwrap();
    Ok(rtn)
}

fn _retrieve_manifest(verkey: &str, wallet_handle: i32) -> Result<BackupManifest, u32> {
    let manifest_entry = ManifestEntry{
        file_path: String::new(),
        s3_key: String::from(verkey),
        s3_bucket: String::from(S3_BUCKET),
    };

    let data = _retrieve_file(&manifest_entry)?;
    let data = _decrypt_bytes(verkey, wallet_handle,&data)?;

    let manifest: BackupManifest = serde_json::from_slice(&data[..])
        .or(Err(10000 as u32))?;
    Ok(manifest)
}

fn _restore_file(verkey: &str, wallet_handle: i32, entry: &ManifestEntry) -> Result<(), u32> {
    let data = _retrieve_file(entry)?;
    let data = _decrypt_bytes(verkey, wallet_handle, &data)?;

    if entry.file_path.contains("DKMS_for_Alice") {
        match entry.file_path.contains("sqlite.db") {
            true => println!("{} \n{}", entry.file_path, "WALLET DATA"),
            false => println!("{} \n{}", entry.file_path, String::from_utf8_lossy(&data[..]))
        };
    }
    else {
        let p = Path::new(&entry.file_path);
        let mut f = File::create(&p).or(Err(10004 as u32))?;

        f.write_all(&data[..]).or(Err(10005 as u32))?;
    }

    Ok(())
}

fn _restore_files(verkey: &str, wallet_handle: i32) -> Result<(), u32> {
    let manifest = _retrieve_manifest(verkey, wallet_handle)?;

    println!("Recovery Entries");
    println!("{}", serde_json::to_string_pretty(&manifest)
        .unwrap());

    for entry in manifest.entries {
        _restore_file(verkey, wallet_handle, &entry)?;
    }

    Ok(())
}

fn _recover_key(shares_json: &str, wallet_handle: i32) -> Result<String, u32> {
    let secret_json: Value = _sss_recovery(shares_json)?;

    let seed = secret_json["seed"].as_str().ok_or(error::INVALID_JSON.code_num)?;
    let verkey = secret_json["verkey"].as_str().ok_or(error::INVALID_JSON.code_num)?;

    let key_json = json!({"seed": seed});
    let key_json = serde_json::to_string(&key_json).or(Err(error::INVALID_JSON.code_num))?;
    let new_verkey = libindy::crypto::libindy_create_key(wallet_handle, &key_json)?;

    if new_verkey.eq(verkey) {
        Ok(new_verkey)
    } else {
        Err(10000)
    }
}

fn _sss_recovery(shares_json: &str) -> Result<Value, u32> {
    let secret = libindy::sss::libindy_recover_secret_from_shards(&shares_json)?;

    let out:Value = serde_json::from_str(&secret).unwrap();
    let out = serde_json::to_string_pretty(&out).unwrap();
    println!("secret: {}", out);

    serde_json::from_str(&secret).or(Err(error::INVALID_JSON.code_num))
}


#[cfg(test)]
mod tests {
//    use super::*;

//    #[test]
//    fn test_backup_files() {
//
//
//        let files = vec![String::from("/tmp/file1")];
//
//        backup_identity_files(files, "VERKEY").unwrap();
//    }

//    #[test]
//    fn do_restore_test() {
//        do_restore(r#"[234,4234,54352234,532534]"#).unwrap();
//    }



}
