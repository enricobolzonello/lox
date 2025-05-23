use crate::{errors::ResultExec, Interpreter};
use std::{fmt::Display, rc::Rc};

use lox_syntax::Literal;

pub trait LoxCallable {
    fn arity(&self) -> usize;

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> ResultExec<Value>;
}

#[derive(Clone)]
pub enum Value {
    Number(f32),
    String(String),
    Bool(bool),
    Null,
    NativeFunction(Rc<dyn LoxCallable>),
}

impl From<Literal> for Value {
    fn from(value: Literal) -> Self {
        match value {
            Literal::Number(n) => Value::Number(n),
            Literal::Bool(b) => Value::Bool(b),
            Literal::String(s) => Value::String(s),
            Literal::Null => Value::Null,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(x) => write!(f, "{}", x),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            _ => Err(std::fmt::Error),
        }
    }
}
