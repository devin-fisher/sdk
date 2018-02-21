use super::actor::Actor;

pub const DEV_GENESIS_NODE_TXNS: &[&'static str; 4] = &[
    r#"{"data":{"alias":"Node1","blskey":"4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba","client_ip":"34.210.112.124","client_port":9702,"node_ip":"34.210.112.124","node_port":9701,"services":["VALIDATOR"]},"dest":"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv","identifier":"Th7MpTaRZVRYnPiabds81Y","txnId":"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62","type":"0"}"#,
    r#"{"data":{"alias":"Node2","blskey":"37rAPpXVoxzKhz7d9gkUe52XuXryuLXoM6P6LbWDB7LSbG62Lsb33sfG7zqS8TK1MXwuCHj1FKNzVpsnafmqLG1vXN88rt38mNFs9TENzm4QHdBzsvCuoBnPH7rpYYDo9DZNJePaDvRvqJKByCabubJz3XXKbEeshzpz4Ma5QYpJqjk","client_ip":"34.216.146.243","client_port":9704,"node_ip":"34.216.146.243","node_port":9703,"services":["VALIDATOR"]},"dest":"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb","identifier":"EbP4aYNeTHL6q385GuVpRV","txnId":"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc","type":"0"}"#,
    r#"{"data":{"alias":"Node3","blskey":"3WFpdbg7C5cnLYZwFZevJqhubkFALBfCBBok15GdrKMUhUjGsk3jV6QKj6MZgEubF7oqCafxNdkm7eswgA4sdKTRc82tLGzZBd6vNqU8dupzup6uYUf32KTHTPQbuUM8Yk4QFXjEf2Usu2TJcNkdgpyeUSX42u5LqdDDpNSWUK5deC5","client_ip":"52.25.215.119","client_port":9706,"node_ip":"52.25.215.119","node_port":9705,"services":["VALIDATOR"]},"dest":"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya","identifier":"4cU41vWW82ArfxJxHkzXPG","txnId":"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4","type":"0"}"#,
    r#"{"data":{"alias":"Node4","blskey":"2zN3bHM1m4rLz54MJHYSwvqzPchYp8jkHswveCLAEJVcX6Mm1wHQD1SkPYMzUDTZvWvhuE6VNAkK3KxVeEmsanSmvjVkReDeBEMxeDaayjcZjFGPydyey1qxBHmTvAnBKoPydvuTAqx5f7YNNRAdeLmUi99gERUU7TD8KfAa6MpQ9bw","client_ip":"52.11.136.146","client_port":9708,"node_ip":"52.11.136.146","node_port":9707,"services":["VALIDATOR"]},"dest":"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA","identifier":"TWwCRQRZ2ZHMJFn9TzLp7W","txnId":"aa5e817d7cc626170eca175822029339a444eb0ee8f0bd20d3b0b76e566fb008","type":"0"}"#
];

#[allow(dead_code)]
pub struct PairwiseInfo {
    agent_my_did: String,
    agent_my_vk: String,
    agent_their_did: String,
    agent_their_vk: String,

    agency_my_did: String,
    agency_my_vk: String,
    agency_their_did: String,
    agency_their_vk: String,
}


pub fn pairwise_info(actor: &Actor) -> PairwiseInfo {
    match actor {
        &Actor::Alice => {
            PairwiseInfo {
                agent_my_did: String::from(""),
                agent_my_vk: String::from(""),
                agent_their_did: String::from(""),
                agent_their_vk: String::from(""),
                agency_my_did: String::from(""),
                agency_my_vk: String::from(""),
                agency_their_did: String::from(""),
                agency_their_vk: String::from(""),
            }
        },
        &Actor::Bob => unimplemented!(),
        &Actor::CUnion => unimplemented!(),
        &Actor::Dakota => unimplemented!(),
    }
}

pub fn wallet_entries(actor: &Actor) -> &[[&str;3]] {
    match actor {
        &Actor::Alice => {
            &[
                [
                    "my_did::38tBpmsH99LqfXrG7wuYNx",
                    r#"{"did":"38tBpmsH99LqfXrG7wuYNx","verkey":"2AXAuzeMXCh23wpPZzGq3B182jU29ccXdRgfAQGhN9wX"}"#,
                    "2018-01-25 22:17:30"
                ],
                [
                    "key::2AXAuzeMXCh23wpPZzGq3B182jU29ccXdRgfAQGhN9wX",
                    r#"{"verkey":"2AXAuzeMXCh23wpPZzGq3B182jU29ccXdRgfAQGhN9wX","signkey":"ZaRtXpSNAPrBjD1jywTQDqswuevXFvjuYNiGqsH4RuoXzjofrrxGyx6SwXsMN51qRYveFRkyh5dDPLQJjdtPQss"}"#,
                    "2018-01-25 22:17:30"
                ],
                [
                    "my_did::SdpzhfX4eKBUMZ6fnvec46",
                    r#"{"did":"SdpzhfX4eKBUMZ6fnvec46","verkey":"EyNYoVXzJCFxs4Q7v8McUqbkFLkKxaKPYRivZ1XQgNL4"}"#,
                    "2018-01-25 22:17:58"
                ],
                [
                    "key::EyNYoVXzJCFxs4Q7v8McUqbkFLkKxaKPYRivZ1XQgNL4",
                    r#"{"verkey":"EyNYoVXzJCFxs4Q7v8McUqbkFLkKxaKPYRivZ1XQgNL4","signkey":"2DA2PHPPjv1rmrnNBv1CRRr7J8DP9jhyuTDbiWDvWzKc7hC1AFfpqjrUBaccEdRf9LiHmtFPKAbHcxPnef317Qr8"}"#,
                    "2018-01-25 22:17:58"
                ],
            ]
        },
        &Actor::Bob => unimplemented!(),
        &Actor::CUnion => unimplemented!(),
        &Actor::Dakota => unimplemented!(),
    }
}

pub fn asset_name(actor: &Actor) -> String {
    format!("DKMS_for_{}", actor)
}