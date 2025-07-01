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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Instance {
    klass: Rc<Class>,
    fields: Rc<RefCell<HashMap<String, Value>>>,
}

impl Instance {
    pub fn new(klass: Rc<Class>) -> Self {
        Self {
            klass,
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&self, name: &Token) -> ResultExec<Value> {
        let key = name.to_string();
        self.fields.borrow().get(&key).cloned().or_else(|| {
            let method = self.klass.find_method(&key)?;
            let instance_ref = Rc::new(RefCell::new(self.clone()));
            method.bind(instance_ref).map(Value::Callable)
        }).ok_or_else(|| {
            Error::undefined_var(format!("Undefined property '{}'.", key), Some(name.clone()))
        })
    }

    pub fn set(&mut self, name: &Token, value: &Value) {
        self.fields.borrow_mut().insert(name.to_string(), value.clone());
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.klass)
    }
}
