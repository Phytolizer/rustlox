use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::object::Obj;

#[derive(Clone)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(Box<Obj>),
}

impl Value {
    pub fn is_bool(&self) -> bool {
        if let Self::Bool(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_nil(&self) -> bool {
        if let Self::Nil = self {
            true
        } else {
            false
        }
    }

    pub fn is_number(&self) -> bool {
        if let Self::Number(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_obj(&self) -> bool {
        if let Self::Obj(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_bool(&self) -> bool {
        if let Self::Bool(b) = self {
            *b
        } else {
            panic!("not a bool");
        }
    }

    pub fn as_number(&self) -> f64 {
        if let Self::Number(n) = self {
            *n
        } else {
            panic!("not a number");
        }
    }

    pub fn is_falsey(&self) -> bool {
        self.is_nil() || (self.is_bool() && !self.as_bool())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{}", b),
            Self::Nil => write!(f, "nil"),
            Self::Number(n) => write!(f, "{}", n),
            Self::Obj(o) => write!(f, "{}", o),
        }
    }
}

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        Self::Number(-self.as_number())
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        Value::Number(self.as_number() + rhs.as_number())
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        Value::Number(self.as_number() - rhs.as_number())
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        Value::Number(self.as_number() * rhs.as_number())
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        Value::Number(self.as_number() / rhs.as_number())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Bool(b) => other.is_bool() && *b == other.as_bool(),
            Self::Nil => other.is_nil(),
            Self::Number(n) => other.is_number() && *n == other.as_number(),
            Self::Obj(o) => {
                let Obj::String(a) = o as &Obj;
                if let Value::Obj(other) = other {
                    let Obj::String(b) = other as &Obj;
                    a == b
                } else {
                    false
                }
            }
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_number().partial_cmp(&other.as_number())
    }
}
