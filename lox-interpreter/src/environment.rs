use crate::{
    errors::{Error, ResultExec},
    value::Value,
};
use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

#[derive(Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn from(enclosing: &Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Rc::clone(enclosing)),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> ResultExec<Value> {
        match self.values.get(name) {
            Some(v) => return Ok(v.clone()),
            None => {
                if let Some(env) = &self.enclosing {
                    return env.borrow().get(name);
                } else {
                    Err(
                        Error::undefined_var(format!("Undefined variable '{}'.", name), None)
                    )
                }
            }
        }
    }

    pub fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let mut environment = Rc::new(RefCell::new((*self).clone()));
        for _ in 0..distance {
            let next = environment.borrow().enclosing.clone();
            if let Some(enc) = next {
                environment = enc;
            } else {
                panic!("No enclosing environment at distance {}", distance);    // TODO: che cazzo faccio?
            }
        }
        environment
    }

    pub fn get_at(&self, distance: usize, name: &str) -> ResultExec<Value> {
        let env = self.ancestor(distance);
        let binding = &env.borrow().values;
        binding
            .get(name)
            .cloned()
            .ok_or_else(|| Error::undefined_var(format!("Undefined variable '{}'.", name), None))
    }

    pub fn assign(&mut self, name: &str, value: Value) -> ResultExec<()> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return Ok(());
        } else {
            if let Some(enclosing) = &self.enclosing {
                enclosing.borrow_mut().assign(name, value)
            } else {
                Err(Error::undefined_var(
                    format!("Undefined variable '{}'.", name),
                    None,
                ))
            }
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: &str, value: Value) {
        self.ancestor(distance)
            .borrow_mut()
            .values
            .insert(name.to_string(), value);
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Environment {{")?;
        for (key, value) in &self.values {
            writeln!(f, "  {}: {}", key, value)?;
        }

        if let Some(enclosing) = &self.enclosing {
            writeln!(f, "  Enclosing ->\n{}", enclosing.borrow())?;
        }

        write!(f, "}}")
    }
}
