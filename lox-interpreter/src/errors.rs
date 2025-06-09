use core::fmt;
use lox_syntax::Token;
use crate::Value;

pub type ResultExec<T> = Result<T, ControlFlow>;

#[derive(Debug)]
pub enum ControlFlow {
    Error(Error),
    Runtime(RuntimeControl),
}

#[derive(Debug)]
pub enum RuntimeControl {
    Break,
    Return(Value),
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    location: Option<Token>,
}

#[derive(Debug)]
pub enum ErrorKind {
    UnrecognizedExpr(String),
    UnrecognizedStmt(String), 
    UnrecognizedOpt(String),
    UnexpectedExpr(String),
    UnexpectedStmt(String),
    UnexpectedOpt(String),
    WrongValueType(String),
    NotCallable(String),
    UnusedVariable(String),
    InvalidContext(String),
    UndefinedVar(String),
}

macro_rules! error_constructors {
    ($(($method:ident, $variant:ident, $param:ident)),+ $(,)?) => {
        impl Error {
            pub fn new(kind: ErrorKind, location: Option<Token>) -> Self {
                Self { kind, location }
            }
            
            $(
                pub fn $method($param: impl Into<String>, location: Option<Token>) -> ControlFlow {
                    ControlFlow::Error(Self::new(ErrorKind::$variant($param.into()), location))
                }
            )+
        }
    };
}

error_constructors! {
    (unrecognized_expr, UnrecognizedExpr, desc),
    (unrecognized_stmt, UnrecognizedStmt, desc),
    (unrecognized_opt, UnrecognizedOpt, desc),
    (unexpected_expr, UnexpectedExpr, desc),
    (unexpected_stmt, UnexpectedStmt, desc),
    (unexpected_opt, UnexpectedOpt, desc),
    (wrong_value_type, WrongValueType, msg),
    (not_callable, NotCallable, name),
    (unused_variable, UnusedVariable, name),
    (invalid_context, InvalidContext, msg),
    (undefined_var, UndefinedVar, desc),
}

impl ControlFlow {
    pub fn break_flow() -> Self {
        Self::Runtime(RuntimeControl::Break)
    }

    pub fn return_(value: Value) -> Self {
        Self::Runtime(RuntimeControl::Return(value))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.location {
            Some(token) => write!(f, "[line {}] Error at '{}': {}", token.line, token.to_string(), self.kind),
            None => write!(f, "Error: {}", self.kind),
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnrecognizedExpr(desc) => write!(f, "Unrecognized expression: {}", desc),
            Self::UnrecognizedStmt(desc) => write!(f, "Unrecognized statement: {}", desc),
            Self::UnrecognizedOpt(desc) => write!(f, "Unrecognized optional construct: {}", desc),
            Self::UnexpectedExpr(desc) => write!(f, "Unexpected expression: {}", desc),
            Self::UnexpectedStmt(desc) => write!(f, "Unexpected statement: {}", desc),
            Self::UnexpectedOpt(desc) => write!(f, "Unexpected optional construct: {}", desc),
            Self::WrongValueType(msg) => write!(f, "Wrong value type: {}", msg),
            Self::NotCallable(name) => write!(f, "'{}' is not callable", name),
            Self::UnusedVariable(name) => write!(f, "Variable '{}' is declared but never used", name),
            Self::InvalidContext(msg) => write!(f, "Invalid context: {}", msg),
            Self::UndefinedVar(desc) => write!(f, "Undefined variable: {}", desc),
        }
    }
}

impl std::error::Error for Error {}