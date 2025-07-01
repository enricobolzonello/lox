use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    rc::Rc,
};

use lox_syntax::Token;

use crate::{
    errors::{Error, ResultExec}, function::Function, interpreter::LoxCallable, Value
};

#[derive(Clone)]
pub struct Class {
    name: String,
    methods: HashMap<String, Function>
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Function>) -> Self {
        Self { name, methods}
    }

    pub fn find_method(&self, name: &str) -> Option<Function> {
        self.methods.get(name).cloned()
    }
}

impl LoxCallable for Class {
    fn call(
        &self,
        interpreter: &mut crate::Interpreter,
        arguments: &Vec<crate::Value>,
    ) -> crate::errors::ResultExec<crate::Value> {
        let instance = Instance::new(Rc::new(self.clone()));

        return Ok(Value::Instance(Rc::new(RefCell::new(instance))));
    }

    fn arity(&self) -> usize {
        0
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

pub struct Instance {
    klass: Rc<Class>,
    fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(klass: Rc<Class>) -> Self {
        Self {
            klass,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> ResultExec<Value> {
        let key = name.to_string();
        self.fields.get(&key).cloned().or_else(|| {
            self.klass.find_method(&key).map(Value::Callable)
        }).ok_or_else(|| {
            Error::undefined_var(format!("Undefined property '{}'.", key), Some(name.clone()))
        })
    }

    pub fn set(&mut self, name: &Token, value: &Value) {
        self.fields.insert(name.to_string(), value.clone());
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.klass)
    }
}
