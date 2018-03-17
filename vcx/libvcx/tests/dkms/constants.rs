#![allow(dead_code)]
use super::actor::Actor;



pub const ACCOUNT_CERT_SCHEMA_SEQ_NUM: u32 = 48;
pub const ACCOUNT_CERT_DID: &'static str = "Pd4fnFtRBcMKRVC2go5w3j";

pub const B_CARD_SCHEMA_SEQ_NUM: u32 = 52;
pub const B_CARD_DID: &'static str = "4fUDR9R7fjwELRvH9JT6HH";

pub const V_TITLE_SCHEMA_SEQ_NUM: u32 = 60;
pub const V_TITLE_DID: &'static str = "Niaxv2v4mPr1HdTeJkQxuU";

pub const DAKOTAS_VIN: &'static str = "1G1ZC5E06CF170071";

pub const DEV_GENESIS_NODE_TXNS: &[&'static str; 4] = &[
    r#"{"data":{"alias":"Node1","blskey":"4N8aUNHSgjQVgkpm8nhNEfDf6txHznoYREg9kirmJrkivgL4oSEimFF6nsQ6M41QvhM2Z33nves5vfSn9n1UwNFJBYtWVnHYMATn76vLuL3zU88KyeAYcHfsih3He6UHcXDxcaecHVz6jhCYz1P2UZn2bDVruL5wXpehgBfBaLKm3Ba","client_ip":"34.210.112.124","client_port":9702,"node_ip":"34.210.112.124","node_port":9701,"services":["VALIDATOR"]},"dest":"Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv","identifier":"Th7MpTaRZVRYnPiabds81Y","txnId":"fea82e10e894419fe2bea7d96296a6d46f50f93f9eeda954ec461b2ed2950b62","type":"0"}"#,
    r#"{"data":{"alias":"Node2","blskey":"37rAPpXVoxzKhz7d9gkUe52XuXryuLXoM6P6LbWDB7LSbG62Lsb33sfG7zqS8TK1MXwuCHj1FKNzVpsnafmqLG1vXN88rt38mNFs9TENzm4QHdBzsvCuoBnPH7rpYYDo9DZNJePaDvRvqJKByCabubJz3XXKbEeshzpz4Ma5QYpJqjk","client_ip":"34.216.146.243","client_port":9704,"node_ip":"34.216.146.243","node_port":9703,"services":["VALIDATOR"]},"dest":"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb","identifier":"EbP4aYNeTHL6q385GuVpRV","txnId":"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc","type":"0"}"#,
    r#"{"data":{"alias":"Node3","blskey":"3WFpdbg7C5cnLYZwFZevJqhubkFALBfCBBok15GdrKMUhUjGsk3jV6QKj6MZgEubF7oqCafxNdkm7eswgA4sdKTRc82tLGzZBd6vNqU8dupzup6uYUf32KTHTPQbuUM8Yk4QFXjEf2Usu2TJcNkdgpyeUSX42u5LqdDDpNSWUK5deC5","client_ip":"52.25.215.119","client_port":9706,"node_ip":"52.25.215.119","node_port":9705,"services":["VALIDATOR"]},"dest":"DKVxG2fXXTU8yT5N7hGEbXB3dfdAnYv1JczDUHpmDxya","identifier":"4cU41vWW82ArfxJxHkzXPG","txnId":"7e9f355dffa78ed24668f0e0e369fd8c224076571c51e2ea8be5f26479edebe4","type":"0"}"#,
    r#"{"data":{"alias":"Node4","blskey":"2zN3bHM1m4rLz54MJHYSwvqzPchYp8jkHswveCLAEJVcX6Mm1wHQD1SkPYMzUDTZvWvhuE6VNAkK3KxVeEmsanSmvjVkReDeBEMxeDaayjcZjFGPydyey1qxBHmTvAnBKoPydvuTAqx5f7YNNRAdeLmUi99gERUU7TD8KfAa6MpQ9bw","client_ip":"52.11.136.146","client_port":9708,"node_ip":"52.11.136.146","node_port":9707,"services":["VALIDATOR"]},"dest":"4PS3EDQ3dW1tci1Bp6543CfuuebjFrg36kLAUcskGfaA","identifier":"TWwCRQRZ2ZHMJFn9TzLp7W","txnId":"aa5e817d7cc626170eca175822029339a444eb0ee8f0bd20d3b0b76e566fb008","type":"0"}"#
];

#[allow(dead_code)]
pub struct ConfigInfo {
    pub agent_endpoint: String,
    pub agency_pairwise_did: String,
    pub agency_pairwise_verkey: String,
    pub enterprise_did_agent: String,
    pub enterprise_did: String,
    pub enterprise_verkey: String,
    pub agent_pairwise_did: String,
    pub agent_pairwise_verkey: String,
    pub agent_enterprise_verkey: String,
    pub identity_policy_address: String,
//    pub agent_policy_verkey: String,
    pub recovery_verkey: String,
}


pub fn config_info(actor: &Actor) -> ConfigInfo {
    match actor {
        &Actor::Alice => {
            ConfigInfo {
                agent_endpoint: String::from("https://dkmscas.pdev.evernym.com"),
                agency_pairwise_did: String::from("YTpNMUPavSFvTXYJHSZgsh"),
                agency_pairwise_verkey: String::from("J9b45WK4tW56kSV2SC1UN5DKkZdMHAejv8eVoNuaXMkX"),
                enterprise_did_agent: String::from("4A2GxCWKfxqUFEC8ozixJq"),
                agent_enterprise_verkey: String::from("2ikgmgutu3DLHfoofp6ibLfe9fZqjMvDo36hnoWXT5oZ"),
                enterprise_did: String::from("RiPVER16apmvuRmUkoQphz"),
                enterprise_verkey: String::from("EUFDEw2ZoJnJh3Ni7nW7de2F9cGH8NAU9ixkAcj4Xdn7"),
                agent_pairwise_did: String::from("D21h4rojfACty4EUH4FCQg"),
                agent_pairwise_verkey: String::from("7YtvPnhSYfF5a6DQb9AsGKEs1eYK6uXx7nUkSAPUZdX8"),
                identity_policy_address: String::from("175"),
//                agent_policy_verkey: String::from("F5xT9jRfsbm3GQLcLB1SCP539debmD9mN95VoYjYF4HX"),
                recovery_verkey: String::from("CXEbaQLPZoiLr17tKd6KkHcGjhzk8dUr4JEx82fdvN8Z"),
            }
        },
        &Actor::Bob => {
            ConfigInfo {
                agent_endpoint: String::from("https://dkmseas.pdev.evernym.com"),
                agency_pairwise_did: String::from("E52bWCiD5A4zuWScnHmjbA"),
                agency_pairwise_verkey: String::from("889k8QBcHXq3ND5LXYJDcQgUNudk2cpJrBqdsR9geKak"),
                enterprise_did_agent: String::from("WnmajvwGQpk7gVNmy4ng7U"),
                agent_enterprise_verkey: String::from("HEhB3SvhrXGoGSLe2Uc1FJy11PgPvmRSugESPXdwzLHk"),
                enterprise_did: String::from("VrMYPRfK6AsnuHpPBMvWFs"),
                enterprise_verkey: String::from("Gj32NuiYuaf2jyxY71f9x9siYYLggWJz8VH8aRbNg5XH"),
                agent_pairwise_did: String::from("2zitW92wBcvPt4TCiU2UvB"),
                agent_pairwise_verkey: String::from("265D1cR241omRUCnHiBFs1JZXvCMaLUJDqrvgSMUkXVH"),
                identity_policy_address: String::new(),
//                agent_policy_verkey: String::from("7AkE14sGDknmzxZC63BDSkm9HmCou1Nwx5aPgh2iEeFP"),
                recovery_verkey: String::from("At8x8WDMCrmJbxD1KWo7S51L87vFBehAfvnfMpfGcaQg"),
            }
        },
        &Actor::CUnion => {
            ConfigInfo {
                agent_endpoint: String::from("https://dkmseas.pdev.evernym.com"),
                agency_pairwise_did: String::from("E52bWCiD5A4zuWScnHmjbA"),
                agency_pairwise_verkey: String::from("889k8QBcHXq3ND5LXYJDcQgUNudk2cpJrBqdsR9geKak"),
                enterprise_did_agent: String::from("6Czcy9G2BfwyNAUuPJHTee"),
                agent_enterprise_verkey: String::from("3qbcHPgL7LmFgZf1UYyngHBujEFqN4NGedwVUPxyqcCk"),
                enterprise_did: String::from("Pd4fnFtRBcMKRVC2go5w3j"),
                enterprise_verkey: String::from("DL7uYjX8tAmGWX1uwKBuG8wWPswUx9TtcPtkMANu2nXz"),
                agent_pairwise_did: String::from("Qy7pkD729Dj5XSeZCNMyU5"),
                agent_pairwise_verkey: String::from("E4fMq1DV55k4FcPZCGQ68xLExF4pM4ALYtresjW8qZdx"),
                identity_policy_address: String::new(),
//                agent_policy_verkey: String::from("C5CZ2AQLdNQPFJAWLnGWJ85BWCVCGdiGp1GCrNtHStoX"),
                recovery_verkey: String::from("EEu1sdgWR55xsXbVoiJ8zi4v8LPbBLqpDSyAZUHUYQox"),
            }
        },
        &Actor::Dakota => {
            ConfigInfo {
                agent_endpoint: String::from("https://dkmseas.pdev.evernym.com"),
                agency_pairwise_did: String::from("E52bWCiD5A4zuWScnHmjbA"),
                agency_pairwise_verkey: String::from("889k8QBcHXq3ND5LXYJDcQgUNudk2cpJrBqdsR9geKak"),
                enterprise_did_agent: String::from("9FmJe8iMFyKTR5VCc3Bh8h"),
                agent_enterprise_verkey: String::from("5VwfqVFZ7Wa3nuHbAkrgwksWfaiQCEK4d4tS6ZWtxXuN"),
                enterprise_did: String::from("RztTtfb5PNY7bofjtGxLCb"),
                enterprise_verkey: String::from("EdEoAe6wgtgeeqtWksDVAWpstS5KcWqgzVGH1RkmVujN"),
                agent_pairwise_did: String::from("PWSoWzotn79Y2BSovgubZz"),
                agent_pairwise_verkey: String::from("DGWgmV8MgMA1ptd294thrns3mHBE9N9Nfy5okHLz8Sb9"),
                identity_policy_address: String::new(),
//                agent_policy_verkey: String::from("EgENwWrTW5zjSkSFWqg8orK3kzwiESLmQTDY6Dt9Tajp"),
                recovery_verkey: String::from("Ageo5PwVFdaQjSszNT9hsfPA4szCFq4sT378mfUnaNtL"),
            }
        },
        &Actor::AliceNew => {
            ConfigInfo {
                agent_endpoint: String::from("https://dkmscas.pdev.evernym.com"),
                agency_pairwise_did: String::from("YTpNMUPavSFvTXYJHSZgsh"),
                agency_pairwise_verkey: String::from("J9b45WK4tW56kSV2SC1UN5DKkZdMHAejv8eVoNuaXMkX"),
                enterprise_did_agent: String::from("JuJdkXUukd5wvGujXgCh2P"),
                agent_enterprise_verkey: String::from("AkuqZRrBTQLUVKkTKcGYtkFbypNDBWmLKEc1mrK3ZKSf"),
                enterprise_did: String::from("XZQU4ASGLWJpYowW44Lrvf"),
                enterprise_verkey: String::from("Hf2Cnwz9wD9S4Ma6pWQrDMprVcWipoG2XtZBbbVEMFf1"),
                agent_pairwise_did: String::from("9jUpkVPHNnjH2W8PKKs3dQ"),
                agent_pairwise_verkey: String::from("5m3pgiWkHFnTnTvNtVpQjfTQ6SpDrJVkaN8XUUjfVYVg"),
                identity_policy_address: String::new(),
//                agent_policy_verkey: String::from("EdVjJ7Ym9DU7mcgUQRoBEDZJHf1FPmmuw5MZZt2aAFEJ"),
                recovery_verkey: String::from("3kQC49LLPApgeSBRoHrX4JgW9rvDiihYAZsyHWRpzP4n"),
            }
        },
    }
}

pub fn wallet_entries(actor: &Actor) -> &[[&str;3]] {
    match actor {
        &Actor::Alice => {
            &[
                [
                    "my_did::4A2GxCWKfxqUFEC8ozixJq",
                    r#"{"did":"4A2GxCWKfxqUFEC8ozixJq","verkey":"2ikgmgutu3DLHfoofp6ibLfe9fZqjMvDo36hnoWXT5oZ"}"#,
                    "2018-02-22 16:00:33"
                ],
                [
                    "key::2ikgmgutu3DLHfoofp6ibLfe9fZqjMvDo36hnoWXT5oZ",
                    r#"{"verkey":"2ikgmgutu3DLHfoofp6ibLfe9fZqjMvDo36hnoWXT5oZ","signkey":"3HYnQ1zzQAWi7dVX3CFG1pQeajtw4agZ4iv47VsaMrb7KDo7cVFDa9aBuJubcRUXmjbCoVyJW6RgLFQoBrLQFBJj"}"#,
                    "2018-02-22 16:00:33"
                ],
                [
                    "my_did::RiPVER16apmvuRmUkoQphz",
                    r#"{"did":"RiPVER16apmvuRmUkoQphz","verkey":"EUFDEw2ZoJnJh3Ni7nW7de2F9cGH8NAU9ixkAcj4Xdn7"}"#,
                    "2018-02-22 16:00:33"
                ],
                [
                    "key::EUFDEw2ZoJnJh3Ni7nW7de2F9cGH8NAU9ixkAcj4Xdn7",
                    r#"{"verkey":"EUFDEw2ZoJnJh3Ni7nW7de2F9cGH8NAU9ixkAcj4Xdn7","signkey":"3uscKiWeq83kEwmXvupYehiceeEHjHWzMKb622vanPmpAVY38QBt1iew9efoxpNTw8gA94qgSkrQphE1ZQhBDLMu"}"#,
                    "2018-02-22 16:00:33"
                ],
                [
                    "master_secret::main",
                    "44292212780527715273569181416807314709025090365466852467172845866433579704768",
                    "2017-12-08 23:21:02"
                ],
                [
                    "claim::70e42df0-a93d-49aa-8117-425a314d7c7c",
                    r#"{"claim":{"email":["alice.smith@example.com","56901711577436054276367872728333077353183139130983372176207408151310182272649"],"title":["Manager","62928949658730028146506999708778947341182844594256749892644546069451170002964"],"name":["Alice Smith","62816810226936654797779705000772968058283780124309077049681734835796332704413"],"phone_number":["363-555-0111","26098064393724315718530507735731781130666607566527034735888166350027600989193"],"business":["LLC Group","103013494000866774643105352582681326915464325536727557995522450468582139772980"]},"schema_seq_no":52,"signature":{"primary_claim":{"m2":"86971216911095705449914254006386448836821958054758647721616018611944742808689","a":"91057648527793246284814398013684707078566216627913822462395888746863739248869821850487025480025864065049311972067911333629842540022380918130664993335643241013630839304078834078526533155172904674693162142710245849252166343262196523374225167642277535335741203436170359644966237135323052045126795056291695640192308478953723853376652249297829303392661581741878657701633932713873932798973993687101830449869217031513016133499583212591689787005406201118807843863123649468595165527750562757679521873390088104185753288312073441589459178137983826942809628955050499476719792145012591355972589625024519803894481912513086214150695","e":"259344723055062059907025491480697571938277889515152306249728583105665800713306759149981690559193987143012367913206299323899696942213235956742929985808042958112170556656264174353439","v":"8164552391691327677858122516325195406848532751622364797829104338522408932345209541670111844671533992990055566761735790284792559737539757988781984838653449208083154686821189846637446079358390103322563362734829251425676200475104456797545571427993329638086003999272432467289922043636781691447649986379433501352346823890429576722971907700752667307267286628825035360963995202851915107274314612130410058542240097932189186715596025072318691693094127960193351291180600166812240903536161921690400710817384492388722076805791082782711923532155594311256349850606430873931790812575268668989480578834914627995556853534602276518720165008303306742937799825316851277755012049323579947414390456492355664188654055208811950355433716935821052036662599575764159272259247350127213220446822076631025590394338611331901887734340934784022426445610"},"non_revocation_claim":null},"issuer_did":"4fUDR9R7fjwELRvH9JT6HH"}"#,
                    "2018-03-01 17:51:21"
                ],
                [
                    "claim::bae018de-76c7-496d-9302-c860a14554df",
                    r#"{"claim":{"owner_name":["Alice Smith","62816810226936654797779705000772968058283780124309077049681734835796332704413"],"year":["2018","9580575055681116735701888307693311696919443567981152550712229354087663885817"],"owner_address":["3569 Gore Street, Avon TN 44011","19384535770851212988443221030781497988507752636414290798260257632744461817193"],"make":["Toyota","77773225235315955558310796460870215037784467515149914108705163467072509007141"],"vin":["1G1ZC5E06CF170071","101673737806105001323898372608061717291339196162267977960883483742024720265360"],"model":["Avalon","24353634147174945122309475759963277096559197779347496000626523599657091347240"]},"schema_seq_no":60,"signature":{"primary_claim":{"m2":"45958957404117505648367358814000150357753869676087912822918363518712519643231","a":"18778739794006702946179521171277315373595687285107578835178426320831733838860837382510481567095007271116914913168435554640603937508873835773291013292535003488646081587872060434522343016333764874000801026788348127693390257989236153353495275191396021723498905721833309475527550796720960825303135801260174801987427983223003830205284702730412283344695735618367401776749003842874324848898399422431906110642776735650052850473182631277577091611061721412493578822861761630484468340039190882176751548981743800556125663282778366124218885267859860918775871393199181621319713464483999741525381882822650258079170743147804957390352","e":"259344723055062059907025491480697571938277889515152306249728583105665800713306759149981690559193987143012367913206299323899696942213235956742929747120560815032865983652957581182037","v":"9653060866891869998606827892063221125366091611714992704418680346308352505820771017169955013119434880693420450129906253269406809569116785606436681461529612198079410429168918909742578748814089725565106828430429497016980360915520951246902892747937823599437349637217124405442180588025353312928503074426584233963974257610284532200881568909419880047440883256242714897032852476489498923980751643176330526279707357525472495295696343986436692659097932516045781693622728076224475419111237638485497576658426230670862262846279526963798837127047026006689232829163592474928073519409333896089522764749924669615846536753393562852931437976471325496780224805451877058888245532538621712298103227783309261882908405615606694069028561459684709140707384555118553919551314866580272817899498080946456866161246435461182293325708058766123377591087"},"non_revocation_claim":null},"issuer_did":"Niaxv2v4mPr1HdTeJkQxuU"}"#,
                    "2018-03-01 18:02:11"
                ],
//                [
//                    "key::F5xT9jRfsbm3GQLcLB1SCP539debmD9mN95VoYjYF4HX",
//                    r#"{"verkey":"F5xT9jRfsbm3GQLcLB1SCP539debmD9mN95VoYjYF4HX","signkey":"xt19s1sp2UZCGhy9rNyb1FtxdKiDGZZPPWbEpqU41PZEyYbm9VofTjQuxHzrFQrFt3EDN9nUnpeorpNEAP9GRGm"}"#,
//                    "2018-03-08 23:37:05"
//                ],
                [
                    "key::CXEbaQLPZoiLr17tKd6KkHcGjhzk8dUr4JEx82fdvN8Z",
                    r#"{"verkey":"CXEbaQLPZoiLr17tKd6KkHcGjhzk8dUr4JEx82fdvN8Z","signkey":"xt19s1sp2UZCGhy9rNyb1FtxdKiDGZZPPWbEpqU41PZVe97gcA89iRV4drh4sg311Nz3w9FbRDpz4w9QZk1TgN5"}"#,
                    "2018-03-08 23:37:05"
                ],
                [
                    "authz_address::175",
                    r#"{"address":"175","agents":{}}"#,
                    "2018-03-08 23:37:05"
                ],
            ]
        },
        &Actor::Bob => {
            &[
                [
                    "my_did::WnmajvwGQpk7gVNmy4ng7U",
                    r#"{"did":"WnmajvwGQpk7gVNmy4ng7U","verkey":"HEhB3SvhrXGoGSLe2Uc1FJy11PgPvmRSugESPXdwzLHk"}"#,
                    "2018-02-21 18:31:17"
                ],
                [
                    "key::HEhB3SvhrXGoGSLe2Uc1FJy11PgPvmRSugESPXdwzLHk",
                    r#"{"verkey":"HEhB3SvhrXGoGSLe2Uc1FJy11PgPvmRSugESPXdwzLHk","signkey":"u2tjQjGh15M6qqUV3CFPW1VMBEZUq3JpXgsF3ob3wsRHZxQUWZrUjFRf15xUVvYK9oVfhwaLqZ736exc2mR9tCN"}"#,
                    "2018-02-21 18:31:17"
                ],
                [
                    "my_did::VrMYPRfK6AsnuHpPBMvWFs",
                    r#"{"did":"VrMYPRfK6AsnuHpPBMvWFs","verkey":"Gj32NuiYuaf2jyxY71f9x9siYYLggWJz8VH8aRbNg5XH"}"#,
                    "2018-02-21 18:31:17"
                ],
                [
                    "key::Gj32NuiYuaf2jyxY71f9x9siYYLggWJz8VH8aRbNg5XH",
                    r#"{"verkey":"Gj32NuiYuaf2jyxY71f9x9siYYLggWJz8VH8aRbNg5XH","signkey":"1GJn5t74Pq4FpxMwm7kjB4HsWSKMhbHtfUQwijLdFrjgT2sWb5yN9PjPUe9aKuky4GUNyZYgXvA8ZySfLEvugHh"}"#,
                    "2018-02-21 18:31:17"
                ],
                [
                    "key::7AkE14sGDknmzxZC63BDSkm9HmCou1Nwx5aPgh2iEeFP",
                    r#"{"verkey":"7AkE14sGDknmzxZC63BDSkm9HmCou1Nwx5aPgh2iEeFP","signkey":"xt19s1sp2UZCGhy9rNyb1FtxdKiDGZZPPWbEpqU41QpmztHi7NAuKdj6xiAwdc9UzegKyJ9NCZHLRsuZkafLEH3"}"#,
                    "2018-03-08 23:37:05"
                ],
                [
                    "key::At8x8WDMCrmJbxD1KWo7S51L87vFBehAfvnfMpfGcaQg",
                    r#"{"verkey":"At8x8WDMCrmJbxD1KWo7S51L87vFBehAfvnfMpfGcaQg","signkey":"xt19s1sp2UZCGhy9rNyb1FtxdKiDGZZPPWbEpqU41Qq8wbPLLTrgUM2H39QQHQLfXhPQpeuQ3wbYL3W6odzE7gc"}"#,
                    "2018-03-08 23:37:05"
                ],
            ]
        },
        &Actor::CUnion => {
            &[
                [
                    "my_did::6Czcy9G2BfwyNAUuPJHTee",
                    r#"{"did":"6Czcy9G2BfwyNAUuPJHTee","verkey":"3qbcHPgL7LmFgZf1UYyngHBujEFqN4NGedwVUPxyqcCk"}"#,
                    "2018-02-21 18:39:26"
                ],
                [
                    "key::3qbcHPgL7LmFgZf1UYyngHBujEFqN4NGedwVUPxyqcCk",
                    r#"{"verkey":"3qbcHPgL7LmFgZf1UYyngHBujEFqN4NGedwVUPxyqcCk","signkey":"3gv3krh5Z9oqC7y7kZRbc8xkhxgCUo3587vJYYx3fasXK2KHGAWXNMATfB8Qh6qan5PLVbD4qkMLCX1Gh6jXJfRe"}"#,
                    "2018-02-21 18:39:26"
                ],
                [
                    "my_did::WKbTGQHzYtGkiBTyzLPZnr",
                    r#"{"did":"WKbTGQHzYtGkiBTyzLPZnr","verkey":"Gyt8W5aAQZ6QF5Km4gwgfNyNenbLGMm5GbrYx5bXLt4t"}"#,
                    "2018-02-21 18:39:26"
                ],
                [
                    "key::Gyt8W5aAQZ6QF5Km4gwgfNyNenbLGMm5GbrYx5bXLt4t",
                    r#"{"verkey":"Gyt8W5aAQZ6QF5Km4gwgfNyNenbLGMm5GbrYx5bXLt4t","signkey":"26YK4UdBCHnFLnsC1vmNdrfRe42Ytf9t4ptmKr9M4UogWRBLsLnqFCp31TqakoCW69mCzBtnHcdUVpKWP1e2Vo3i"}"#,
                    "2018-02-21 18:39:26"
                ],
                [
                    "my_did::Pd4fnFtRBcMKRVC2go5w3j",
                    r#"{"did":"Pd4fnFtRBcMKRVC2go5w3j","verkey":"DL7uYjX8tAmGWX1uwKBuG8wWPswUx9TtcPtkMANu2nXz"}"#,
                    "2018-02-27 02:59:01"
                ],
                [
                    "key::DL7uYjX8tAmGWX1uwKBuG8wWPswUx9TtcPtkMANu2nXz",
                    r#"{"verkey":"DL7uYjX8tAmGWX1uwKBuG8wWPswUx9TtcPtkMANu2nXz","signkey":"xt19s1sp2UZCGhy9rNyb1FtxdKiDGZZPPWbEpqU41YiMk2jeZftCRBe1X6Ad91MEJ4EGd2XfqMpoBB3ioYqRFcU"}"#,
                    "2018-02-27 02:59:01"
                ],
                [
                    "claim_definition::Pd4fnFtRBcMKRVC2go5w3j:48",
                    r#"{"ref":48,"origin":"Pd4fnFtRBcMKRVC2go5w3j","signature_type":"CL","data":{"primary":{"n":"111022329548070187077857283706553082290467468182721551695170583260748493353229104766126301821267804336818781986543035653932381574039721510154121963547396250843829270989980812451178599498774427514329506503150217176351201656823080327832109899177020019143317035989459902155601667950349972102233015140970388321306142244673109008790451239141487202597317303462499297931118602503288115398931822690284781433980919529915179655154559830951707532454394778710178299813795035622771415843536009214707788376687283664749552907739803298913937710809619555843571765889475827580080668837889036513633043647655958933269959083490472470399293","s":"84814770308312248304673217372223858503142398044129347066619999530444327007051469130512114144864932279376179052559082044997626314515904294054830564074858131465201330314853232593035421558894333572500784170978117816394671623174861573261649923478455310806266667267254751143988401744995204923027403307671866440132211282863448870800817332697008783260752074711257334405677307657763247922033604177097730483501143503943097286822078433891994910350623474698778325949200574514592428835542289873454312322964233993828191356428838739554018191547266228730991831668571113297904197610035378256319328681644276362816851229773187506360873","rms":"22452064416211439656599755291209044499763435388151507979659169503310801093769881501007098778390635647213628094740972446186976033969304963994346283448575322318759612270034817763067664226955870519713158161727657871215786466321914955182261766445925668276041284543253659578412593746023118399700344807633000174791016205502494522004945025599200674203887262046735489169743859629360986021537478637782519671106009878229930395023546764766658670947956447998315796748643875653685421663037603895746829948561550308073596400852624925979736989707100827505058341412528752363662502356152654749364910692589309229595627659644749082167952","r":{"name_on_account":"90318670385912966505999349764729167986540898540572854197797204608095262513551602349246462737870827446427338460046780552366907858501835304427714386085736196260406982440243842822877839263185850601006674949446061563808282370540401201868362156610053537893608823473786475518262691959474193685952641206695782244668990429228644923637012431390596411260627520865538772520571311125183815000316742305061073375557139134986749121889533001848092623102737108608895688036433680446209707409373231344476442444900738690327497288829854567982529664472712528005642821391529200505500508146412997689296890589935959915485327368871402218030614","account_num":"1895824774342033347763444422620977109604913022302799385774412504083499397773748078556147825803950835524989340780966223171523981610924773767936354897747970346120106732850112861346755510121940420292069841259003070677710849793538474181426019883728951512137198718825759541668035453810234409282145627840200987423072316820216471709872857967188133088095355453260777757587027566462203295847467393002758397794170917578295676236552538312238823629636590856117790868581409989182812102414445029619265958105783489388509787647268098056584589242586487181717520627922346907203630135738602167937072597077626006987658795148114309327976"},"rctxt":"50925864510799682921261963381584241742528050674623333771118984723232890368523959163454173521408989735730015412476882017541681038222651340241191585090458790835967031994101782233830213669596403383547622143548598216523314896409607113251583098572574639256115826490617127821077614887294881393620124149154783113461075660530999637531710059024676111775473675467882348277375895315819277085344050583396865648200580848877583144575956080053227749285418799748114050685457649335162775546708492136700004252722628432364870821401622282213894050373098338079150024126714135490641809863431105418814407043929753945887516690373170037954174","z":"35570230535423432310010843214190387804653595094276860330589966282056181887798039053204037603959089795907754190429756874807140330084930201862418033046971873391861598287622211558484898763133596441546292146906847338595677860602441678651762177005280715438984427361275239124028251360171604427821296327470316885621642995817343674897157794356877957769267676425842467339927095707923962508110528300348831353818648088842042399720314826270646406171474182943521062281149440988320962847069066958656737033074314553462018024320884338556507162345420354453634298789353051085617357424332353757907055768870731657549557623835331807790222"},"revocation":null}}"#,
                    "2018-02-27 03:18:43"
                ],
                [
                    "claim_definition_private::Pd4fnFtRBcMKRVC2go5w3j:48",
                    r#"{"secret_key":{"p":"165620706875531324845648340547184433272638787597468540305938344400804913276783859956126475789452586694080216135500462698826265824228530929519756915310539765515589728658837570183276395012347296712436632524265844676253152418992104284809116307126060186637157327373538595212497515051212929698122399373985331268089","q":"167585218724351037334580037678595476872833839686732277800952761360571389584939644935118430551698244442175781736126154795956331706761166126460485859231573295191766946019622439689527658260849149838175196569482892033498652748433663215265076447840685219107992229105919000313564786075618543138061307425030353925383"},"secret_key_revocation":null}"#,
                    "2018-02-27 03:18:43"
                ],
                [
                    "key::C5CZ2AQLdNQPFJAWLnGWJ85BWCVCGdiGp1GCrNtHStoX",
                    r#"{"verkey":"C5CZ2AQLdNQPFJAWLnGWJ85BWCVCGdiGp1GCrNtHStoX","signkey":"xt19s1sp2UZCGhy9rNyb1FtxdKiDGZZPPWbEpqU41QqTMyMinQSu2isVTtKej42ioZm3PsBEF2oATUNTHHmaPKj"}"#,
                    "2018-03-08 23:37:05"
                ],
                [
                    "key::EEu1sdgWR55xsXbVoiJ8zi4v8LPbBLqpDSyAZUHUYQox",
                    r#"{"verkey":"EEu1sdgWR55xsXbVoiJ8zi4v8LPbBLqpDSyAZUHUYQox","signkey":"xt19s1sp2UZCGhy9rNyb1FtxdKiDGZZPPWbEpqU41QqnkzC53SDUn9Dgn6jLqFxccN1v2QttAES3x8f2J7jBraS"}"#,
                    "2018-03-08 23:37:05"
                ],

            ]
        },
        &Actor::Dakota => {
            &[
                [
                    "my_did::9FmJe8iMFyKTR5VCc3Bh8h",
                    r#"{"did":"9FmJe8iMFyKTR5VCc3Bh8h","verkey":"5VwfqVFZ7Wa3nuHbAkrgwksWfaiQCEK4d4tS6ZWtxXuN"}"#,
                    "2018-03-01 21:56:12"
                ],
                [
                    "key::5VwfqVFZ7Wa3nuHbAkrgwksWfaiQCEK4d4tS6ZWtxXuN",
                    r#"{"verkey":"5VwfqVFZ7Wa3nuHbAkrgwksWfaiQCEK4d4tS6ZWtxXuN","signkey":"2cTzZi7og5kt7TwNefZaQEhRAoGbirhSWDZx9zxLv1EGv1N8ptLu8sTsKb5p5wieyAqjcff6nS77VUQjLz2ieXoa"}"#,
                    "2018-03-01 21:56:12"
                ],
                [
                    "my_did::RztTtfb5PNY7bofjtGxLCb",
                    r#"{"did":"RztTtfb5PNY7bofjtGxLCb","verkey":"EdEoAe6wgtgeeqtWksDVAWpstS5KcWqgzVGH1RkmVujN"}"#,
                    "2018-03-01 21:56:12"
                ],
                [
                    "key::EdEoAe6wgtgeeqtWksDVAWpstS5KcWqgzVGH1RkmVujN",
                    r#"{"verkey":"EdEoAe6wgtgeeqtWksDVAWpstS5KcWqgzVGH1RkmVujN","signkey":"5NQSkc4hTKqNZSz69Uzb2cyNDBCEVcQV5ybkwg9zeoySaMpWxPSY6G7KhizvpGCS7nZ2KYf6uj9BNkkYRoMWhKrx"}"#,
                    "2018-03-01 21:56:12"
                ],
                [
                    "key::EgENwWrTW5zjSkSFWqg8orK3kzwiESLmQTDY6Dt9Tajp",
                    r#"{"verkey":"EgENwWrTW5zjSkSFWqg8orK3kzwiESLmQTDY6Dt9Tajp","signkey":"xt19s1sp2UZCGhy9rNyb1FtxdKiDGZZPPWbEpqU41Qr6RdvdiMmMqnkq5iuH8pFcyQfoBcLgToUj1LMQy9Acyka"}"#,
                    "2018-03-08 23:37:05"
                ],
                [
                    "key::Ageo5PwVFdaQjSszNT9hsfPA4szCFq4sT378mfUnaNtL",
                    r#"{"verkey":"Ageo5PwVFdaQjSszNT9hsfPA4szCFq4sT378mfUnaNtL","signkey":"xt19s1sp2UZCGhy9rNyb1FtxdKiDGZZPPWbEpqU41QrKfNiGPCPuS7Cgrx4MvU7t1HJMZJ9T4bgFeBGxLAaFj9N"}"#,
                    "2018-03-08 23:37:05"
                ],
            ]
        },
        &Actor::AliceNew => {
            &[
                [
                    "my_did::JuJdkXUukd5wvGujXgCh2P",
                    r#"{"did":"JuJdkXUukd5wvGujXgCh2P","verkey":"AkuqZRrBTQLUVKkTKcGYtkFbypNDBWmLKEc1mrK3ZKSf"}"#,
                    "2018-03-01 21:56:12"
                ],
                [
                    "key::AkuqZRrBTQLUVKkTKcGYtkFbypNDBWmLKEc1mrK3ZKSf",
                    r#"{"verkey":"AkuqZRrBTQLUVKkTKcGYtkFbypNDBWmLKEc1mrK3ZKSf","signkey":"5PhmtW6JkQnjVJFefQQHeGFAGD5KzR4etfksn2rwTGHzvgvy1zJPVAqXU3uMNnC1124Uftm6GLojWFifUUUXfhQs"}"#,
                    "2018-03-01 21:56:12"
                ],
                [
                    "my_did::XZQU4ASGLWJpYowW44Lrvf",
                    r#"{"did":"XZQU4ASGLWJpYowW44Lrvf","verkey":"Hf2Cnwz9wD9S4Ma6pWQrDMprVcWipoG2XtZBbbVEMFf1"}"#,
                    "2018-03-01 21:56:12"
                ],
                [
                    "key::Hf2Cnwz9wD9S4Ma6pWQrDMprVcWipoG2XtZBbbVEMFf1",
                    r#"{"verkey":"Hf2Cnwz9wD9S4Ma6pWQrDMprVcWipoG2XtZBbbVEMFf1","signkey":"4V9w1CRbdpW6Bktgud7eB6RCaRdid1sL7r4DKS4ET6XYaknkY2bBvnzgdEESMXHgHXYEZxEEBbKe6EACAdXPtm8f"}"#,
                    "2018-03-01 21:56:12"
                ],
                [
                    "key::EdVjJ7Ym9DU7mcgUQRoBEDZJHf1FPmmuw5MZZt2aAFEJ",
                    r#"{"verkey":"EdVjJ7Ym9DU7mcgUQRoBEDZJHf1FPmmuw5MZZt2aAFEJ","signkey":"xt19s1sp2UZCGhy9rNyb1FtxdKiDGZZPPWbEpqU41TUnPhn7mSR5EH8LLTpvvBfUnvzmgRzkmXDqomXJ6n6mEkr"}"#,
                    "2018-03-01 21:56:12"
                ],
                [
                    "key::3kQC49LLPApgeSBRoHrX4JgW9rvDiihYAZsyHWRpzP4n",
                    r#"{"verkey":"3kQC49LLPApgeSBRoHrX4JgW9rvDiihYAZsyHWRpzP4n","signkey":"xt19s1sp2UZCGhy9rNyb1FtxdKiDGZZPPWbEpqU41TUtjvcNayb7ENTnEkCYxRJW6ritNzHpvWwZMFFsec87Kqc"}"#,
                    "2018-03-01 21:56:12"
                ],
            ]
        },
    }
}

pub fn asset_name(actor: &Actor) -> String {
    format!("DKMS_for_{}", actor)
}