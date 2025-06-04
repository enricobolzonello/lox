use std::{cell::RefCell, fmt::Debug, rc::Rc};

use lox_syntax::{Stmt, Token};

use crate::{environment::Environment, errors::{ControlFlow, ResultExec, RuntimeControl}, value::Value, Interpreter};

#[derive(Clone)]
pub enum Function {
    Native {
        arity: usize,
        body: Box<fn(&Vec<Value>) -> Value>,
    },
    Custom {
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    }
}

impl Function {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: &Vec<Value>) -> ResultExec<Value> {
        match self {
            Function::Native { body, .. } => Ok(body(arguments)),
            Function::Custom { params, body , closure} => {
                let environment = Rc::new(RefCell::new(Environment::from(closure)));
                for (param, argument) in params.iter().zip(arguments.iter()) {
                    environment.borrow_mut().define(&param.literal.as_ref().unwrap().to_string(), argument.clone());
                }

                match interpreter.execute_block(&body, environment) {
                    Ok(_) => Ok(Value::Null),
                    Err(ControlFlow::Runtime(RuntimeControl::Return(value))) => Ok(value),
                    Err(e) => Err(e),
                }
                
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

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Native { arity, body } => f.debug_struct("Native").field("arity", arity).field("body", body).finish(),
            Self::Custom { params, body, ..  } => f.debug_struct("Custom").field("params", params).field("body", body).finish(),
        }
    }
}