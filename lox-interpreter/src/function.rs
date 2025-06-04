use std::{cell::RefCell, rc::Rc};

use lox_syntax::{Stmt, Token};

use crate::{environment::{Environment}, errors::ResultExec, value::Value, Interpreter};

#[derive(Clone)]
pub enum Function {
    Native {
        arity: usize,
        body: Box<fn(&Vec<Value>) -> Value>,
    },
    Custom {
        params: Vec<Token>,
        body: Vec<Stmt>,
    }
}

impl Function {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: &Vec<Value>) -> ResultExec<Value> {
        match self {
            Function::Native { body, .. } => Ok(body(arguments)),
            Function::Custom { params, body } => {
                let environment = Rc::new(RefCell::new(Environment::from(&interpreter.globals)));
                for (param, argument) in params.iter().zip(arguments.iter()) {
                    environment.borrow_mut().define(&param.literal.as_ref().unwrap().to_string(), argument.clone());
                }

                interpreter.execute_block(&body, environment)?;
                Ok(Value::Null)
            },
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Function::Native { arity, .. } => *arity,
            Function::Custom { params, .. } => params.len(),
        }
    }
}