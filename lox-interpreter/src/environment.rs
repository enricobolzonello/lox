use crate::errors::{Error, Result};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use lox_syntax::Literal;

pub struct Environment {
    values: HashMap<String, Literal>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_rec(other: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(other),
        }
    }

    pub fn define(&mut self, name: &str, value: Literal) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Result<Literal> {
        match self.values.get(name) {
            Some(v) => return Ok(v.clone()),
            None => {
                if let Some(env) = &self.enclosing {
                    return env.borrow().get(name);
                } else {
                    Err(Error::interpret_error(format!(
                        "Undefined variable '{}'.",
                        name
                    )))
                }
            }
        }
    }

    pub fn assign(&mut self, name: &str, value: Literal) -> Result<()> {
    if self.values.contains_key(name) {
        self.values.insert(name.to_string(), value);
        return Ok(());
    } else {
        if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(Error::interpret_error(format!(
                "Undefined variable '{}'.",
                name
            )))
        }
    }
}
}
