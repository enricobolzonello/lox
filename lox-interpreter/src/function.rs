use std::{cell::RefCell, fmt::Debug, rc::Rc};

use lox_syntax::{Stmt, Token};

use crate::{class::Instance, environment::Environment, errors::{ControlFlow, ResultExec, RuntimeControl}, interpreter::LoxCallable, value::Value, Interpreter};

#[derive(Clone)]
pub enum Function {
    Native {
        arity: usize,
        body: Box<fn(&Vec<Value>) -> Value>,
    },
    Custom {
        params: Rc<Vec<Token>>,
        body: Rc<Vec<Stmt>>,
        closure: Rc<RefCell<Environment>>,
    },
}

impl Function {
    pub fn bind(&self, instance: Rc<RefCell<Instance>>) -> Option<Function>{
        if let Self::Custom { params, body, closure } = self {
            let mut environment = Environment::from(closure);
            environment.define("this", Value::Instance(instance));
            return Some(Function::Custom { 
                params: Rc::clone(params), 
                body: Rc::clone(body), 
                closure: Rc::new(RefCell::new(environment)) 
            });
        }

        None
    }
}

impl LoxCallable for Function {
     fn call(&self, interpreter: &mut Interpreter, arguments: &Vec<Value>) -> ResultExec<Value> {
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

     fn arity(&self) -> usize {
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