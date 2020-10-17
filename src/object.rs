use lazy_static::lazy_static;

use std::{borrow::Cow, fmt::Debug, fmt::Display, sync::Arc, sync::RwLock};

use crate::interpreter::Interpreter;

pub type LoxObject = Arc<RwLock<Object>>;

lazy_static! {
    static ref NIL: LoxObject = Arc::new(RwLock::new(Object::Nil));
    static ref TRUE: LoxObject = Arc::new(RwLock::new(Object::Bool(true)));
    static ref FALSE: LoxObject = Arc::new(RwLock::new(Object::Bool(false)));
}

#[derive(Debug)]
pub enum Object {
    Nil,
    String(String),
    Number(f64),
    Bool(bool),
    BuiltinFunction(usize, fn(Vec<LoxObject>) -> LoxObject),
}

impl Object {
    pub fn new_number(value: f64) -> LoxObject {
        Arc::new(RwLock::new(Object::Number(value)))
    }

    pub fn new_bool(value: bool) -> LoxObject {
        match value {
            true => TRUE.clone(),
            false => FALSE.clone(),
        }
    }

    pub fn nil() -> LoxObject {
        NIL.clone()
    }

    pub fn new_string(value: String) -> LoxObject {
        Arc::new(RwLock::new(Object::String(value)))
    }

    pub fn new_builtin_function(arity: usize, func: fn(Vec<LoxObject>) -> LoxObject) -> LoxObject {
        Arc::new(RwLock::new(Object::BuiltinFunction(arity, func)))
    }

    pub fn is_nil(&self) -> bool {
        match self {
            Object::Nil => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Object::String(_) => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Object::Number(_) => true,
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Object::Bool(_) => true,
            _ => false,
        }
    }

    /// Why?
    pub fn as_nil(&self) {}

    pub fn as_string(&self) -> Cow<str> {
        match self {
            Object::String(s) => Cow::Borrowed(s),
            _ => Cow::Owned(self.to_string()),
        }
    }

    pub fn as_number(&self) -> f64 {
        match self {
            Object::Nil => 0.0,
            Object::String(s) => s.len() as f64,
            Object::Number(n) => *n,
            Object::Bool(b) => *b as i32 as f64,
            Object::BuiltinFunction(..) => 0.0,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Object::Bool(b) => *b,
            Object::Nil => false,
            _ => true,
        }
    }

    pub fn is_callable(&self) -> bool {
        match self {
            Object::Nil => false,
            Object::String(_) => false,
            Object::Number(_) => false,
            Object::Bool(_) => false,
            Object::BuiltinFunction(..) => true,
        }
    }

    pub fn call(&mut self, interpreter: &mut Interpreter, arguments: Vec<LoxObject>) -> LoxObject {
        match self {
            Object::BuiltinFunction(_, func) => func(arguments),
            _ => unreachable!(),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Object::BuiltinFunction(arity, ..) => *arity,
            _ => std::usize::MAX,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Nil => write!(f, "nil"),
            Object::String(s) => write!(f, "{}", s),
            Object::Number(n) => write!(f, "{}", n),
            Object::Bool(b) => write!(f, "{}", b),
            Object::BuiltinFunction(..) => write!(f, "<native fn>"),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        if self.is_nil() && other.is_nil() {
            true
        } else if self.is_nil() || other.is_nil() {
            false
        } else if self.is_bool() && other.is_bool() {
            self.as_bool() == other.as_bool()
        } else if self.is_number() && other.is_number() {
            self.as_number() == other.as_number()
        } else if self.is_string() && other.is_string() {
            self.as_string().as_ref() == other.as_string().as_ref()
        } else {
            false
        }
    }
}
