extern crate libc;

pub mod cxs;
pub mod connection;
pub mod issuer_claim;

use std::fmt;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum Errorcode
{
    Success = 0,
    Failure = 1,
    Waiting = 2,
}
/// This macro allows the CxsStateType to be
/// serialized within serde as an integer (represented as
/// a string, because its still JSON).
macro_rules! enum_number {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: ::serde::Serializer
            {
                // Serialize the enum as a u64.
                serializer.serialize_u64(*self as u64)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: ::serde::Deserializer<'de>
            {
                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("positive integer")
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<$name, E>
                        where E: ::serde::de::Error
                    {
                        // Rust does not come with a simple way of converting a
                        // number to an enum, so use a big `match`.
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }
                }

                // Deserialize the enum from a u64.
                deserializer.deserialize_u64(Visitor)
            }
        }
    }
}

enum_number!(CxsStateType
{
    CxsStateNone = 0,
    CxsStateInitialized = 1,
    CxsStateOfferSent = 2,
    CxsStateRequestReceived = 3,
    CxsStateAccepted = 4,
    CxsStateUnfulfilled = 5,
    CxsStateExpired = 6,
    CxsStateRevoked = 7,
});



#[repr(C)]
pub struct CxsStatus {
    pub handle: ::std::os::raw::c_int,
    pub status: ::std::os::raw::c_int,
    pub msg: *mut ::std::os::raw::c_char,
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use self::CxsStateType::*;

    #[test]
    fn test_serialize_cxs_state_type(){
        let z = CxsStateNone;
        let y = serde_json::to_string(&z).unwrap();
        assert_eq!(y,"0");
    }
}
