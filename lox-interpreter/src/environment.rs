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
        dbg!(name);
        self.values.insert(name.to_string(), value);
        dbg!(&self.values);
    } 

    pub fn get(&self, name: &str) -> Result<Literal> {
        dbg!(name);
        if self.values.contains_key(name) {
            let v = self.values.get(name).unwrap().clone();
            Ok(v)
        }else{
            Err(Error::interpret_error(format!("Undefined variable '{}'.", name)))
        }
    }
}