use lox_syntax::{Expr, ExprVisitor, Literal, Token, TokenType};
use crate::errors::{Error, Result};

struct Interpreter {}

impl ExprVisitor<Result<Literal>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<Literal> {
        match expr {
            Expr::Literal { value } => {
                self.visit_literal_expr(value)
            }
            Expr::Grouping { expression } => {
                self.visit_grouping_expr(&expression)
            }
            Expr::Binary { left, operator, right } => {
                self.visit_binary_expr(left, operator, right)
            }
            Expr::Unary { operator, right } => {
                self.visit_unary_expr(operator, right)
            }
            _ => Err(Error::interpret_error("Unrecognized expression."))
        }
    }
}

impl Interpreter{
    pub fn new() -> Self {
        Self { }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Literal> {
        expr.accept(self)
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
            TokenType::BANG => {
                Ok(Literal::Bool(!self.is_truthy(&right)))
            }
            _ => {
                Err(Error::runtime_error(operator.clone(),"Unknown unary operator."))
            }
        }
    }

    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Literal> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::MINUS => {
                let (l, r) = self.check_number_operands(&left,&right)?;
                Ok(Literal::Number(l-r))
            }
            TokenType::SLASH => {
                let (l, r) = self.check_number_operands(&left,&right)?;
                Ok(Literal::Number(l/r))
            }
            TokenType::STAR => {
                let (l, r) = self.check_number_operands(&left,&right)?;
                Ok(Literal::Number(l*r))
            }
            TokenType::PLUS => {
                match (left, right) {
                    (Literal::Number(l), Literal::Number(r)) => Ok(Literal::Number(l+r)),
                    (Literal::String(l), Literal::String(r)) => Ok(Literal::String(format!("{}{}", l,r))),
                    _ => Err(Error::runtime_error(operator.clone(),"Operands must be two numbers or two strings.")),
                }
            }
            TokenType::GREATER => {
                let (l, r) = self.check_number_operands(&left,&right)?;
                Ok(Literal::Bool(l>r))
            },
            TokenType::GREATER_EQUAL => {
                let (l, r) = self.check_number_operands(&left,&right)?;
                Ok(Literal::Bool(l>=r))
            },
            TokenType::LESS => {
                let (l, r) = self.check_number_operands(&left,&right)?;
                Ok(Literal::Bool(l<r))
            },
            TokenType::LESS_EQUAL => {
                let (l, r) = self.check_number_operands(&left,&right)?;
                Ok(Literal::Bool(l<=r))
            },
            TokenType::EQUAL_EQUAL => Ok(Literal::Bool(self.is_equal(&left, &right))),
            TokenType::BANG_EQUAL => Ok(Literal::Bool(!self.is_equal(&left, &right))),
            _ => {
                Err(Error::runtime_error(operator.clone(),"Unknown binary operator."))
            }
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

    fn check_number_operands(&self, left: &Literal, right: &Literal) -> Result<(f32,f32)> {
        match (left, right) {
            (Literal::Number(l), Literal::Number(r)) => Ok((*l,*r)),
            _ => Err(Error::interpret_error("Both operands must be a number."))
        }
    }

    fn check_number_operand(&self, operand: &Literal) -> Result<f32> {
        match operand {
            Literal::Number(n) => Ok(*n),
            _ => Err(Error::interpret_error("Operand must be a number."))
        }
    }
}

pub fn interpret(expr: &Expr) -> Result<Literal> {
    let mut interpreter = Interpreter::new();
    interpreter.evaluate(expr)
}