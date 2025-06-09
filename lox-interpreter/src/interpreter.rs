use crate::{
    environment::Environment,
    errors::{ControlFlow, Error, ResultExec, RuntimeControl},
    function::Function,
    value::Value,
};
use lox_syntax::{Expr, ExprVisitor, Stmt, StmtVisitor, Token, TokenType};
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    pub globals: Rc<RefCell<Environment>>,
    locals: HashMap<String, usize>,
}

impl ExprVisitor<ResultExec<Value>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> ResultExec<Value> {
        match expr {
            // TODO: handle comma operator
            Expr::Literal { value } => Ok(Value::from(value.clone())),
            Expr::Grouping { expression } => self.visit_grouping_expr(&expression),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary_expr(left, operator, right),
            Expr::Unary { operator, right } => self.visit_unary_expr(operator, right),
            Expr::Variable { name } => self.visit_var_expr(name),
            Expr::Assign { name, value } => self.visit_assign_expr(name, value),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.visit_logical_expr(left, operator, right),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => self.visit_call_expr(callee, paren, arguments),
            Expr::Lambda { params, body } => self.visit_lambda_expr(params, body),
            Expr::Comma { left, right } => self.visit_comma_expr(left, right),
            _ => Err(Error::unrecognized_expr(
                format!("Unrecognized expression: {:?}.", expr),
                None,
            )),
        }
    }
}

impl StmtVisitor<ResultExec<()>> for Interpreter {
    fn visit_stmt(&mut self, stmt: &lox_syntax::Stmt) -> ResultExec<()> {
        match stmt {
            Stmt::Print { expression } => self.visit_print_stmt(expression),
            Stmt::Expression { expression } => self.visit_expr_stmt(expression),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
            Stmt::Function { name, params, body } => self.visit_function_stmt(name, params, body),
            Stmt::Return { keyword, value } => self.visit_return_stmt(keyword, value),
            Stmt::Block { statements } => self.visit_block_stmt(statements),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
            Stmt::Break => self.visit_break_stmt(),
            _ => Err(Error::unrecognized_stmt(
                format!("Unrecognized stmt: {:?}.", stmt),
                None,
            )),
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));

        Self {
            environment: Rc::clone(&globals),
            globals: Rc::clone(&globals),
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), Error> {
        for stmt in statements {
            if let Err(ControlFlow::Error(e)) = self.execute(stmt) {
                return Err(e);
            }
        }

        Ok(())
    }

    pub fn set_global_fn(&mut self, name: &str, arity: usize, func: fn(&Vec<Value>) -> Value) {
        let callable = Value::Callable(Function::Native {
            arity,
            body: Box::new(func),
        });
        self.globals.borrow_mut().define(name, callable);
    }

    pub fn resolve(&mut self, name: &Token, depth: usize) {
        self.locals.insert(name.to_string(), depth);
    }

    // ----- Expression interpreting methods ----

    fn evaluate(&mut self, expr: &Expr) -> ResultExec<Value> {
        expr.accept(self)
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> ResultExec<Value> {
        let value = self.evaluate(value)?;

        let distance = self.locals.get(&name.to_string());
        if let Some(distance) = distance {
            self.environment
                .borrow_mut()
                .assign_at(*distance, &name.to_string(), value.clone())
        } else {
            self.globals
                .borrow_mut()
                .assign(&name.to_string(), value.clone())?;
        }

        return Ok(value);
    }

    fn visit_var_expr(&self, name: &Token) -> ResultExec<Value> {
        self.look_up_var(name)
    }

    fn visit_grouping_expr(&mut self, value: &Expr) -> ResultExec<Value> {
        self.evaluate(value)
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> ResultExec<Value> {
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::MINUS => {
                let value = self.check_number_operand(&right)?;
                Ok(Value::Number(-value))
            }
            TokenType::BANG => Ok(Value::Bool(!self.is_truthy(&right))),
            _ => Err(Error::unrecognized_opt(
                format!("Unknown unary operator: {}", operator.to_string()),
                Some(operator.clone()),
            )),
        }
    }

    fn visit_binary_expr(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> ResultExec<Value> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::MINUS => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Value::Number(l - r))
            }
            TokenType::SLASH => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Value::Number(l / r))
            }
            TokenType::STAR => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Value::Number(l * r))
            }
            TokenType::PLUS => match (left, right) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
                _ => Err(Error::wrong_value_type(
                    "Operands must be two numbers or two strings.",
                    Some(operator.clone()),
                )),
            },
            TokenType::GREATER => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Value::Bool(l > r))
            }
            TokenType::GREATER_EQUAL => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Value::Bool(l >= r))
            }
            TokenType::LESS => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Value::Bool(l < r))
            }
            TokenType::LESS_EQUAL => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Value::Bool(l <= r))
            }
            TokenType::EQUAL_EQUAL => Ok(Value::Bool(self.is_equal(&left, &right))),
            TokenType::BANG_EQUAL => Ok(Value::Bool(!self.is_equal(&left, &right))),
            _ => Err(Error::unrecognized_opt(
                format!("Unknown binary operator operator: {}", operator.to_string()),
                Some(operator.clone()),
            )),
        }
    }

    fn visit_logical_expr(
        &mut self,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> ResultExec<Value> {
        let left = self.evaluate(&left)?;

        match operator.token_type {
            TokenType::OR => {
                if self.is_truthy(&left) {
                    return Ok(left);
                }
            }
            TokenType::AND => {
                if !self.is_truthy(&left) {
                    return Ok(left);
                }
            }
            _ => {
                return Err(Error::unexpected_opt(
                    format!("Expect logical operator, got {}.", operator.token_type),
                    Some(operator.clone()),
                ))
            }
        }

        self.evaluate(&right)
    }

    fn visit_call_expr(
        &mut self,
        callee_expr: &Expr,
        paren: &Token,
        arg_exprs: &Vec<Expr>,
    ) -> ResultExec<Value> {
        let callee = self.evaluate(callee_expr)?;
        let callable = match callee {
            Value::Callable(f) => f,
            _ => {
                return Err(
                    Error::not_callable(paren.to_string(), Some(paren.clone()))
                )
            }
        };

        let mut args = Vec::with_capacity(arg_exprs.len());
        for argument in arg_exprs {
            args.push(self.evaluate(argument)?);
        }

        callable.call(self, &args)
    }

    fn visit_comma_expr(&mut self, left: &Box<Expr>, right: &Box<Expr>) -> ResultExec<Value> {
        let _left = self.evaluate(&left)?;

        self.evaluate(&right)
    }

    fn visit_lambda_expr(&mut self, params: &Vec<Token>, body: &Vec<Stmt>) -> ResultExec<Value> {
        let function = Function::Custom {
            params: params.to_vec(),
            body: body.to_vec(),
            closure: self.environment.clone(),
        };
        Ok(Value::Callable(function))
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }

    fn is_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Null, Value::Null) => true,
            (Value::Null, _) | (_, Value::Null) => false,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }

    fn check_number_operands(&self, left: &Value, right: &Value) -> ResultExec<(f32, f32)> {
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok((*l, *r)),
            _ => Err(
                Error::wrong_value_type("Both operands must be a number.", None)
            ),
        }
    }

    fn check_number_operand(&self, operand: &Value) -> ResultExec<f32> {
        match operand {
            Value::Number(n) => Ok(*n),
            _ => Err(
                Error::wrong_value_type("Operand must be a number.", None)
            ),
        }
    }

    fn look_up_var(&self, name: &Token) -> ResultExec<Value> {
        let distance = self.locals.get(&name.to_string());
        if let Some(distance) = distance {
            self.environment
                .borrow()
                .get_at(*distance, &name.to_string())
        } else {
            self.globals.borrow().get(&name.to_string())
        }
    }

    // ----- Statement interpreting methods ----

    fn execute(&mut self, stmt: &Stmt) -> ResultExec<()> {
        stmt.accept(self)
    }

    fn visit_expr_stmt(&mut self, expr: &Expr) -> ResultExec<()> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn visit_function_stmt(
        &mut self,
        name: &Token,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
    ) -> ResultExec<()> {
        let function = Function::Custom {
            params: params.clone(),
            body: body.clone(),
            closure: Rc::clone(&self.environment),
        };
        self.environment
            .borrow_mut()
            .define(&name.to_string(), Value::Callable(function));
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> ResultExec<()> {
        let value = self.evaluate(expr)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_return_stmt(&mut self, _keyword: &Token, value: &Option<Expr>) -> ResultExec<()> {
        let value = match value {
            Some(expr) => self.evaluate(expr)?,
            None => Value::Null,
        };

        Err(ControlFlow::Runtime(RuntimeControl::Return(value)))
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> ResultExec<()> {
        let value = match initializer {
            Some(expr) => self.evaluate(expr)?,
            None => Value::Null,
        };

        self.environment
            .borrow_mut()
            .define(&name.literal.as_ref().unwrap().to_string(), value);

        Ok(())
    }

    fn visit_block_stmt(&mut self, stmts: &[Stmt]) -> ResultExec<()> {
        self.execute_block(
            stmts,
            Rc::new(RefCell::new(Environment::from(&self.environment.clone()))),
        )
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> ResultExec<()> {
        let cond = self.evaluate(condition)?;
        if self.is_truthy(&cond) {
            self.execute(&then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(&else_branch)?;
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Box<Stmt>) -> ResultExec<()> {
        while {
            let value = self.evaluate(condition)?;
            self.is_truthy(&value)
        } {
            self.execute(&body)?;
        }
        Ok(())
    }

    fn visit_break_stmt(&mut self) -> ResultExec<()> {
        Err(ControlFlow::Runtime(RuntimeControl::Break))
    }

    pub fn execute_block(
        &mut self,
        stmts: &[Stmt],
        env: Rc<RefCell<Environment>>,
    ) -> ResultExec<()> {
        let previous = self.environment.clone();
        self.environment = env;

        let result = stmts.iter().try_for_each(|stmt| self.execute(stmt));

        self.environment = previous;
        result
    }
}
