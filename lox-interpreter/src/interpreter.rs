use crate::{
    environment::Environment,
    errors::{Error, Result},
};
use lox_syntax::{Expr, ExprVisitor, Literal, Stmt, StmtVisitor, Token, TokenType};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl ExprVisitor<Result<Literal>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<Literal> {
        match expr {
            Expr::Literal { value } => self.visit_literal_expr(value),
            Expr::Grouping { expression } => self.visit_grouping_expr(&expression),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.visit_binary_expr(left, operator, right),
            Expr::Unary { operator, right } => self.visit_unary_expr(operator, right),
            Expr::Variable { name } => self.visit_var_expr(name),
            Expr::Assign { name, value } => self.visit_assign_expr(name, value),
            _ => Err(Error::interpret_error("Unrecognized expression.")),
        }
    }
}

impl StmtVisitor<Result<()>> for Interpreter {
    fn visit_stmt(&mut self, stmt: &lox_syntax::Stmt) -> Result<()> {
        match stmt {
            Stmt::Print { expression } => self.visit_print_stmt(expression),
            Stmt::Expression { expression } => self.visit_expr_stmt(expression),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
            Stmt::Block { statements } => self.visit_block_stmt(statements),
            _ => Err(Error::interpret_error("Unrecognized statement.")),
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<()> {
        for stmt in statements {
            self.execute(stmt)?;
        }

        Ok(())
    }

    // ----- Expression interpreting methods ----

    fn evaluate(&mut self, expr: &Expr) -> Result<Literal> {
        expr.accept(self)
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<Literal> {
        let value = self.evaluate(value)?;
        self.environment
            .borrow_mut()
            .assign(&name.literal.as_ref().unwrap().to_string(), value.clone())?;
        return Ok(value);
    }

    fn visit_var_expr(&self, name: &Token) -> Result<Literal> {
        self.environment
            .borrow()
            .get(&name.literal.as_ref().unwrap().to_string())
    }

    fn visit_literal_expr(&self, value: &Literal) -> Result<Literal> {
        Ok(value.clone())
    }

    fn visit_grouping_expr(&mut self, value: &Expr) -> Result<Literal> {
        self.evaluate(value)
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<Literal> {
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::MINUS => {
                let value = self.check_number_operand(&right)?;
                Ok(Literal::Number(-value))
            }
            TokenType::BANG => Ok(Literal::Bool(!self.is_truthy(&right))),
            _ => Err(Error::runtime_error(
                operator.clone(),
                "Unknown unary operator.",
            )),
        }
    }

    fn visit_binary_expr(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Literal> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::MINUS => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Literal::Number(l - r))
            }
            TokenType::SLASH => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Literal::Number(l / r))
            }
            TokenType::STAR => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Literal::Number(l * r))
            }
            TokenType::PLUS => match (left, right) {
                (Literal::Number(l), Literal::Number(r)) => Ok(Literal::Number(l + r)),
                (Literal::String(l), Literal::String(r)) => {
                    Ok(Literal::String(format!("{}{}", l, r)))
                }
                _ => Err(Error::runtime_error(
                    operator.clone(),
                    "Operands must be two numbers or two strings.",
                )),
            },
            TokenType::GREATER => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Literal::Bool(l > r))
            }
            TokenType::GREATER_EQUAL => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Literal::Bool(l >= r))
            }
            TokenType::LESS => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Literal::Bool(l < r))
            }
            TokenType::LESS_EQUAL => {
                let (l, r) = self.check_number_operands(&left, &right)?;
                Ok(Literal::Bool(l <= r))
            }
            TokenType::EQUAL_EQUAL => Ok(Literal::Bool(self.is_equal(&left, &right))),
            TokenType::BANG_EQUAL => Ok(Literal::Bool(!self.is_equal(&left, &right))),
            _ => Err(Error::runtime_error(
                operator.clone(),
                "Unknown binary operator.",
            )),
        }
    }

    fn is_truthy(&self, value: &Literal) -> bool {
        match value {
            Literal::Null => false,
            Literal::Bool(b) => *b,
            _ => true,
        }
    }

    fn is_equal(&self, left: &Literal, right: &Literal) -> bool {
        match (left, right) {
            (Literal::Null, Literal::Null) => true,
            (Literal::Null, _) | (_, Literal::Null) => false,
            (Literal::Bool(a), Literal::Bool(b)) => a == b,
            (Literal::Number(a), Literal::Number(b)) => a == b,
            (Literal::String(a), Literal::String(b)) => a == b,
            _ => false,
        }
    }

    fn check_number_operands(&self, left: &Literal, right: &Literal) -> Result<(f32, f32)> {
        match (left, right) {
            (Literal::Number(l), Literal::Number(r)) => Ok((*l, *r)),
            _ => Err(Error::interpret_error("Both operands must be a number.")),
        }
    }

    fn check_number_operand(&self, operand: &Literal) -> Result<f32> {
        match operand {
            Literal::Number(n) => Ok(*n),
            _ => Err(Error::interpret_error("Operand must be a number.")),
        }
    }

    // ----- Statement interpreting methods ----

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<()> {
        let value = self.evaluate(expr)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<()> {
        let value = match initializer {
            Some(expr) => self.evaluate(expr)?,
            None => Literal::Null,
        };

        self.environment
            .borrow_mut()
            .define(&name.literal.as_ref().unwrap().to_string(), value);

        Ok(())
    }

    fn visit_block_stmt(&mut self, stmts: &[Stmt]) -> Result<()> {
        self.execute_block(
            stmts,
            Rc::new(RefCell::new(Environment::new_rec(self.environment.clone()))),
        )
    }

    fn execute_block(&mut self, stmts: &[Stmt], env: Rc<RefCell<Environment>>) -> Result<()> {
        let previous = self.environment.clone();

        self.environment = env;
        for stmt in stmts {
            self.execute(stmt)?;
        }

        self.environment = previous;

        Ok(())
    }
}
