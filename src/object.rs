use std::fmt::Display;

use crate::value::Value;

impl Value {
    pub fn is_string(&self) -> bool {
        if let Value::Obj(obj) = self {
            return obj.is_string();
        }
        false
    }
}

#[derive(Clone)]
pub enum Obj {
    String(Vec<u8>),
}

impl Obj {
    pub fn is_string(&self) -> bool {
        if let Obj::String(_) = self {
            true
        } else {
            false
        }
    }
}

impl Display for Obj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", String::from_utf8_lossy(&s)),
        }
    }
}
