extern crate serde_json;

pub mod offer;
pub mod request;
pub mod data;

#[derive(Serialize, Deserialize, Debug)]
pub enum MsgVersion {
    #[serde(rename = "0.1")]
    v0_1
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrusteeMsgType {
    TrusteeOffer,
    TrusteeRequest,
    TrusteeData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrusteeCapability {
    RecoveryShare,
    RevokeAuthz,
    ProvisionAuthz,
    AdminAuthZ,
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use super::offer::*;
    use super::data::*;

    #[test]
    fn test_offer_ser(){
        let offer = TrusteeOffer{
            version: MsgVersion::v0_1,
            msg_type: TrusteeMsgType::TrusteeOffer,
            capabilities: vec![TrusteeCapability::RecoveryShare, TrusteeCapability::RevokeAuthz, TrusteeCapability::ProvisionAuthz],
            expires: None,
            msg_uid: None,
        };

        let test_offer = serde_json::to_value(&offer).unwrap();

        let expect_offer = json!({
          "version": "0.1",
          "msg_type": "TRUSTEE_OFFER",
          "capabilities": ["RECOVERY_SHARE", "REVOKE_AUTHZ", "PROVISION_AUTHZ"],
          "expires": null
        });
        println!("{}", serde_json::to_string_pretty(&test_offer).unwrap());
        println!("{}", serde_json::to_string_pretty(&expect_offer).unwrap());
        assert_eq!(test_offer, expect_offer);
    }


    #[test]
    fn test_trust_ser(){
        let data = TrusteeData{
            version: MsgVersion::v0_1,
            msg_type: TrusteeMsgType::TrusteeData,
            address: String::from("b3AFkei98bf3R2s"),
            share: RecoveryShare{
                version: MsgVersion::v0_1,
                source_did: String::from("did:sov:UVPW2vb5BX6fzTRb5TFHEw"),
                tag: String::from("ze4152Bsxo90"),
                value: String::from("abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ123456789"),
                hint: Some(RecoveryShareHint{
                    theshold: Some(3),
                    trustees: Some(vec![String::from("Mike L"),
                                        String::from("Lovesh"),
                                        String::from("Corin"),
                                        String::from("Devin"),
                                        String::from("Drummond")]),
                }),
            },
        };

        let test_data = serde_json::to_value(&data).unwrap();

        let expect_data = json!({
          "version":"0.1",
          "msg_type":"TRUSTEE_DATA",
          "address":"b3AFkei98bf3R2s",
          "share":{
            "version":"0.1",
            "source_did":"did:sov:UVPW2vb5BX6fzTRb5TFHEw",
            "tag":"ze4152Bsxo90",
            "value":"abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ123456789",
            "hint":{
              "theshold":3,
              "trustees":[
                "Mike L",
                "Lovesh",
                "Corin",
                "Devin",
                "Drummond"
              ]
            }
          }
        });
        println!("{}", serde_json::to_string_pretty(&test_data).unwrap());
        println!("{}", serde_json::to_string_pretty(&expect_data).unwrap());
        assert_eq!(test_data, expect_data);
    }

}