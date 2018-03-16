extern crate uuid;
extern crate serde_json;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate chrono;

use self::rusoto_core::Region;
use self::rusoto_s3::{S3Client, PutObjectRequest};
use self::rusoto_core::default_tls_client;
use self::rusoto_core::ProvideAwsCredentials;
use self::rusoto_core::AwsCredentials;
use self::rusoto_core::CredentialsError;
use self::rusoto_s3::S3;

use self::chrono::{DateTime};
use std::str::FromStr;

use std::fs::File;
use std::io::Read;
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
    let mut f = File::open(file_path).or(Err(101 as u32))?;
    let mut rtn: Vec<u8> = Default::default();
    f.read_to_end(&mut rtn).or(Err(102 as u32))?;

    verkey.len();
    let rtn = rtn.clone(); //Encrypt !!
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
    let result = client.put_object(&put_request).unwrap();
    println!("{:?}", result);

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
    let data = data.clone(); //ENCRYPT

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

    let file_list: Vec<String> = serde_json::from_str(file_list_json).or(Err(error::INVALID_JSON.code_num))?;

    if file_list.len() == 0 {
        warn!("Files list to backup is empty");
        return Ok(error::SUCCESS.code_num);
    }

    backup_identity_files(file_list, &backup_verkey)?;
    Ok(error::SUCCESS.code_num)
}

pub fn do_restore(request_shares_handles: &str) -> Result<u32, u32> {
    let handles: Vec<u32> = serde_json::from_str(request_shares_handles).or(Err(error::INVALID_JSON.code_num))?;

    println!("{:?}", handles);

    let mut shares: Vec<Value> = Vec::default();
    for handle in handles {
        let share = request_share::get_share_val(handle)?;


        let share_json = json!({"value": share});
        shares.push(share_json);
    }

    let shares =  serde_json::to_string_pretty(&shares).unwrap();
    println!("shares:{}",shares);
    let secret = libindy::sss::libindy_recover_secret_from_shards(&shares)?;
    println!("secret:{}", secret);

//    unimplemented!()
    Ok(error::SUCCESS.code_num)
}

#[cfg(test)]
mod tests {
    use super::*;

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
