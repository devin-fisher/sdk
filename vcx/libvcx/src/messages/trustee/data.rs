extern crate serde_json;

use super::MsgVersion;
use super::TrusteeMsgType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecoveryShareHint {
    pub theshold: Option<u32>,
    pub trustees: Option<Vec<String>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecoveryShare {
    pub version: MsgVersion,
    pub source_did: String,
    pub tag: String,
    pub value: String,
    pub hint: Option<RecoveryShareHint>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrusteeData {
    pub version: MsgVersion,
    pub msg_type: TrusteeMsgType,
    pub address: String,
    pub share: RecoveryShare,
}