use crate::{
    callable::Value,
    environment::Environment,
    errors::{ControlFlow, Error, ResultExec, RuntimeControl},
};
use lox_syntax::{Expr, ExprVisitor, Stmt, StmtVisitor, Token, TokenType};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl ExprVisitor<ResultExec<Value>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> ResultExec<Value> {
        match expr {
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
            _ => Err(ControlFlow::Error(Error::interpret_error(
                "Unrecognized expression.",
            ))),
        }
    }
}

impl StmtVisitor<ResultExec<()>> for Interpreter {
    fn visit_stmt(&mut self, stmt: &lox_syntax::Stmt) -> ResultExec<()> {
        match stmt {
            Stmt::Print { expression } => self.visit_print_stmt(expression),
            Stmt::Expression { expression } => self.visit_expr_stmt(expression),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
            Stmt::Block { statements } => self.visit_block_stmt(statements),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
            Stmt::Break => self.visit_break_stmt(),
            _ => Err(ControlFlow::Error(Error::interpret_error(
                "Unrecognized statement.",
            ))),
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
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

    // ----- Expression interpreting methods ----

    fn evaluate(&mut self, expr: &Expr) -> ResultExec<Value> {
        expr.accept(self)
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> ResultExec<Value> {
        let value = self.evaluate(value)?;
        self.environment
            .borrow_mut()
            .assign(&name.literal.as_ref().unwrap().to_string(), value.clone())?;
        return Ok(value);
    }

    fn visit_var_expr(&self, name: &Token) -> ResultExec<Value> {
        self.environment
            .borrow()
            .get(&name.literal.as_ref().unwrap().to_string())
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
            _ => Err(ControlFlow::Error(Error::runtime_error(
                operator.clone(),
                "Unknown unary operator.",
            ))),
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
                _ => Err(ControlFlow::Error(Error::runtime_error(
                    operator.clone(),
                    "Operands must be two numbers or two strings.",
                ))),
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
            _ => Err(ControlFlow::Error(Error::runtime_error(
                operator.clone(),
                "Unknown binary operator.",
            ))),
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
                return Err(ControlFlow::Error(Error::interpret_error(format!(
                    "Expect logical operator, got {}.",
                    operator.token_type
                ))));
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
            Value::NativeFunction(f) => f,
            _ => {
                return Err(ControlFlow::Error(Error::runtime_error(
                    paren.clone(),
                    "Can only call functions and classes.",
                )));
            }
        };

        let mut args = Vec::with_capacity(arg_exprs.len());
        for argument in arg_exprs {
            args.push(self.evaluate(argument)?);
        }

        callable.call(self, args)
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
            _ => Err(ControlFlow::Error(Error::interpret_error(
                "Both operands must be a number.",
            ))),
        }
    }

    fn check_number_operand(&self, operand: &Value) -> ResultExec<f32> {
        match operand {
            Value::Number(n) => Ok(*n),
            _ => Err(ControlFlow::Error(Error::interpret_error(
                "Operand must be a number.",
            ))),
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

    fn visit_print_stmt(&mut self, expr: &Expr) -> ResultExec<()> {
        let value = self.evaluate(expr)?;
        println!("{}", value);
        Ok(())
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
            Rc::new(RefCell::new(Environment::new_rec(self.environment.clone()))),
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

    fn execute_block(&mut self, stmts: &[Stmt], env: Rc<RefCell<Environment>>) -> ResultExec<()> {
        let previous = self.environment.clone();

        self.environment = env;
        for stmt in stmts {
            self.execute(stmt)?;
        }

        self.environment = previous;

        Ok(())
    }
}
