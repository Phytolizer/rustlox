use std::{collections::HashMap, sync::Arc, sync::RwLock};

use crate::{object::LoxObject, runtime_error::RuntimeError, token::Token};

pub struct Environment {
    enclosing: Option<Arc<RwLock<Environment>>>,
    values: HashMap<String, LoxObject>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_enclosed(enclosing: Arc<RwLock<Environment>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: LoxObject) {
        self.values.insert(name.to_owned(), value);
    }

    fn try_get(&self, name: &Token) -> Option<LoxObject> {
        self.values.get(&name.lexeme).cloned().or_else(|| {
            self.enclosing
                .as_ref()
                .and_then(|e| e.write().unwrap().try_get(name))
        })
    }

    pub fn get(&self, name: &Token) -> Result<LoxObject, RuntimeError> {
        self.try_get(name).ok_or_else(|| {
            RuntimeError::new(
                name.clone(),
                format!("Undefined variable '{}'.", name.lexeme),
            )
        })
    }

    fn try_assign(&mut self, name: &Token, value: LoxObject) -> Option<()> {
        self.values
            .get_mut(&name.lexeme)
            .map(|v| *v = value.clone())
            .or_else(|| {
                self.enclosing
                    .as_ref()
                    .and_then(|e| e.write().unwrap().try_assign(name, value))
            })
    }

    pub fn assign(&mut self, name: &Token, value: LoxObject) -> Result<(), RuntimeError> {
        self.try_assign(name, value).ok_or_else(|| {
            RuntimeError::new(
                name.clone(),
                format!("Undefined variable '{}'.", name.lexeme),
            )
        })
    }
}
