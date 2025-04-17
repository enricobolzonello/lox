use crate::tokenizer::token::{Literal, Token};

// Expression enum with all expression types as variants
#[derive(Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    Super {
        keyword: Token,
        method: Token,
    },
    This {
        keyword: Token,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}

// Statement enum with all statement types as variants
pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Class {
        name: Token,
        superclass: Option<Box<Expr>>, 
        methods: Vec<Box<Stmt>>,     
    },
    Expression {
        expression: Expr,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print {
        expression: Expr,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Var {
        name: Token,
        initializer: Option<Expr>, 
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

// Expression Visitor trait
pub trait ExprVisitor<R> {
    fn visit_expr(&mut self, expr: &Expr) -> R;
}

// Statement Visitor trait
pub trait StmtVisitor<R> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> R;
}

// Accept methods for expressions and statements
impl Expr {
    pub fn accept<R>(&self, visitor: &mut dyn ExprVisitor<R>) -> R {
        visitor.visit_expr(self)
    }
}

impl Stmt {
    pub fn accept<R>(&self, visitor: &mut dyn StmtVisitor<R>) -> R {
        visitor.visit_stmt(self)
    }
}