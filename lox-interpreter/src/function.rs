use crate::{errors::ResultExec, value::Value, Interpreter};

#[derive(Clone)]
pub enum Function {
    Native {
        arity: usize,
        body: Box<fn(&Vec<Value>) -> Value>,
    }
}

impl Function {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: &Vec<Value>) -> ResultExec<Value> {
        match self {
            Function::Native { body, .. } => Ok(body(arguments)),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Function::Native { arity, .. } => *arity,
        }
    }
}