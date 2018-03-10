extern crate serde_json;

use super::MsgVersion;
use super::TrusteeMsgType;
use super::TrusteeCapability;

#[derive(Serialize, Deserialize, Debug)]
pub struct TrusteeRequest {
    pub version: MsgVersion,
    pub msg_type: TrusteeMsgType,
    pub capabilities: Vec<TrusteeCapability>,
    pub authorization_keys: Vec<String>,
}
