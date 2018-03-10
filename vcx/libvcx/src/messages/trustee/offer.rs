extern crate serde_json;

use super::MsgVersion;
use super::TrusteeMsgType;
use super::TrusteeCapability;

#[derive(Serialize, Deserialize, Debug)]
pub struct TrusteeOffer {
    pub version: MsgVersion,
    pub msg_type: TrusteeMsgType,
    pub capabilities: Vec<TrusteeCapability>,
    pub expires: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_uid: Option<String>
}
