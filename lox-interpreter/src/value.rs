use std::fmt::Display;

use lox_syntax::Literal;

use crate::function::Function;

#[derive(Clone, Debug)]
pub enum Value {
    Number(f32),
    String(String),
    Bool(bool),
    Null,
    Callable(Function),
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
