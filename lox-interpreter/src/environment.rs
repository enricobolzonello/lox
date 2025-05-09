use std::collections::HashMap;
use crate::errors::{Error, Result};

use lox_syntax::Literal;

pub struct Environment {
    values: HashMap<String, Literal>
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Literal) {
        self.values.insert(name.to_string(), value);
    } 

    pub fn get(&self, name: &str) -> Result<Literal> {
        if self.values.contains_key(name) {
            let v = self.values.get(name).unwrap().clone();
            Ok(v)
        }else{
            Err(Error::interpret_error(format!("Undefined variable '{}'.", name)))
        }
    }

    pub fn assign(&mut self, name: &str, value: Literal) -> Result<()>{
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return  Ok(());
        }

        Err(Error::interpret_error(format!("Undefined variable '{}'.", name)))
    }
}