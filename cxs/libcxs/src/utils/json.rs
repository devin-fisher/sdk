extern crate serde;
extern crate serde_json;

use self::serde::{Serialize, Deserialize};
use self::serde_json::Error;
use serde_json::Value;
use serde_json::Map;
use std::string::String;
use utils::error;
use std::collections::HashMap;


pub trait JsonEncodable: Serialize + Sized {
    fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(self)
    }
}

pub trait JsonDecodable<'a>: Deserialize<'a> {
    fn from_json(to_stringd: &'a str) -> Result<Self, Error> {
        serde_json::from_str(to_stringd)
    }
}

/*
Rewrites keys in a serde value structor to new mapped values. Returns the remapped value. Leaves
unmapped keys as they are.
*/
pub fn mapped_key_rewrite(val: Value, remap: &HashMap<&str, &str>) -> Result<Value, u32> {

    if let Value::Object(mut map) = val {
        let mut keys:Vec<String> = collect_keys(&map);

        while let Some(k) = keys.pop() {
            let mut value = map.remove(&k).ok_or_else(||{
                warn!("Unexpected key value mutation");
                error::INVALID_INVITE_DETAILS.code_num
            })?;

            value = mapped_key_rewrite(value, remap)?;
            let new_k = match remap.get(k.as_str()) {
                Some(s) => s.to_string(),
                None => k
            };

            map.insert(new_k, value);
        }
        Ok(Value::Object(map))
    }
    else {
        Ok(val)
    }
}

fn collect_keys(map:&Map<String, Value>) -> Vec<String>{
    let mut rtn:Vec<String> = Default::default();
    for key in map.keys() {
        rtn.push(key.clone());
    }
    rtn
}