use std::fmt;

impl fmt::Display for Actor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Actor {
    Alice,
    Bob,
    CUnion,
    Dakota,
    Alice_New
}