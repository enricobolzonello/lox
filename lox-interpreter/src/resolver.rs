use std::{cell::RefCell, collections::HashMap, rc::Rc};

use lox_syntax::{Expr, ExprVisitor, Node, Stmt, StmtVisitor, Token};

use crate::{
    errors::{ControlFlow, Error, ResultExec}, Interpreter
};

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    scopes: Vec<HashMap<String, bool>>,
}

impl ExprVisitor<ResultExec<()>> for Resolver {
    fn visit_expr(&mut self, expr: &Expr) -> ResultExec<()> {
        match expr {
            Expr::Variable { name } => self.visit_var_expr(name),
            Expr::Assign { name, value } => self.visit_assign_expr(name, value),
            Expr::Binary { left, right, .. } => self.visit_binary_expr(left, right),
            Expr::Call { callee, arguments , ..} => self.visit_call_expr(callee, arguments),
            Expr::Grouping { expression } => self.visit_grouping_expr(expression),
            Expr::Literal { .. } => Ok(()),
            Expr::Logical { left, right, .. } => {
                self.resolve(&Node::Expr(left.clone()))?;
                self.resolve(&Node::Expr(right.clone()))?;
                Ok(())
            },
            Expr::Unary { right, .. } => {
                self.resolve(&Node::Expr(right.clone()))?;
                Ok(())
            }
            _ => panic!("temp"),
        }
    }
}

impl StmtVisitor<ResultExec<()>> for Resolver {
    fn visit_stmt(&mut self, stmt: &Stmt) -> ResultExec<()> {
        match stmt {
            Stmt::Block { statements } => self.visit_block_stmt(statements),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
            Stmt::Function { name, params, body } => self.visit_function_stmt(name, params, body),
            Stmt::Expression { expression } => self.visit_expression_stmt(expression),
            Stmt::If { condition, then_branch, else_branch } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::Print { expression } => self.visit_print_stmt(expression),
            Stmt::Return { value, .. } => self.visit_return_stmt(value),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
            _ => panic!("temporary"),
        }
    }
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
        }
    }

    pub fn resolve_stmts(&mut self, statements: &[Stmt]) -> ResultExec<()> {
        for stmt in statements {
            self.resolve(&Node::Stmt(Box::new(stmt.clone())))?;
        }
        Ok(())
    }

    fn resolve(&mut self, node: &Node) -> ResultExec<()> {
        match node {
            Node::Expr(expr) => expr.accept(self),
            Node::Stmt(stmt) => stmt.accept(self),
        }
    }

    fn visit_block_stmt(&mut self, statements: &[Stmt]) -> ResultExec<()> {
        self.begin_scope();
        for stmt in statements {
            self.resolve(&Node::Stmt(Box::new(stmt.clone())))?;
        }
        self.end_scope();
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> ResultExec<()> {
        self.declare(name);
        if let Some(init) = initializer {
            self.resolve(&Node::Expr(Box::new(init.clone())))?;
        }
        self.define(name);
        Ok(())
    }

    fn visit_function_stmt(&mut self, name: &Token, parameters: &[Token], body: &[Stmt]) -> ResultExec<()> {
        self.declare(name);
        self.define(name);

        self.resolve_function(parameters, body);
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) -> ResultExec<()> {
        self.resolve(&Node::Expr(Box::new(expr.clone())))?;
        Ok(())
    }

    fn visit_if_stmt(&mut self, condition: &Expr, then_branch: &Box<Stmt>, else_branch: &Option<Box<Stmt>>) -> ResultExec<()> {
        self.resolve(&Node::Expr(Box::new(condition.clone())))?;
        self.resolve(&Node::Stmt(then_branch.clone()))?;
        if let Some(else_branch) = else_branch {
            self.resolve(&Node::Stmt(else_branch.clone()))?;
        }
        Ok(())
    }
 
    fn visit_print_stmt(&mut self, expr: &Expr) -> ResultExec<()> {
        self.resolve(&Node::Expr(Box::new(expr.clone())))?;
        Ok(())
    }

    fn visit_return_stmt(&mut self, value: &Option<Expr>) -> ResultExec<()> {
        if let Some(value) = value {
            self.resolve(&Node::Expr(Box::new(value.clone())))?;
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Box<Stmt>) -> ResultExec<()> {
        self.resolve(&Node::Expr(Box::new(condition.clone())))?;
        self.resolve(&Node::Stmt(body.clone()))?;
        Ok(())
    }

    fn visit_var_expr(&mut self, name: &Token) -> ResultExec<()> {
        if !self.scopes.is_empty()
            && *self.scopes.last().unwrap().get(&name.to_string()).unwrap_or(&false) == false
        {
            return Err(ControlFlow::Error(Error::runtime_error(
                name.clone(),
                "Can't read local variable in its own initializer.",
            )));
        }

        self.resolve_local(name);

        Ok(())
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Box<Expr>) -> ResultExec<()> {
        self.resolve(&Node::Expr(value.clone()))?;
        self.resolve_local(name);
        Ok(())
    }

    fn visit_binary_expr(&mut self, left: &Box<Expr>, right: &Box<Expr>) -> ResultExec<()> {
        self.resolve(&Node::Expr(left.clone()))?;
        self.resolve(&Node::Expr(right.clone()))?;
        Ok(())
    }

    fn visit_call_expr(&mut self, callee: &Box<Expr>, arguments: &[Expr]) -> ResultExec<()> {
        self.resolve(&Node::Expr(callee.clone()))?;
        for argument in arguments {
            self.resolve(&Node::Expr(Box::new(argument.clone())))?;
        }
        Ok(())
    }

    fn visit_grouping_expr(&mut self, expression: &Box<Expr>) -> ResultExec<()> {
        self.resolve(&Node::Expr(expression.clone()))?;
        Ok(())
    }

    // HELPERS

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.to_string(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.to_string(), true);
    }

    fn resolve_local(&mut self, name: &Token) {
        if let Some(distance) = self
            .scopes
            .iter()
            .rev()
            .enumerate()
            .find(|(_, scope)| scope.contains_key(&name.to_string()))
            .map(|(i, _)| i)
        {
            let depth = distance;
            let scope_depth = self.scopes.len() - 1 - depth;
            self.interpreter.borrow_mut().resolve(name, scope_depth);
        }
    }

    fn resolve_function(&mut self, parameters: &[Token], body: &[Stmt]) {
        self.begin_scope();
        for param in parameters {
            self.declare(param);
            self.define(param);
        }

        for stmt in body {
            self.resolve(&Node::Stmt(Box::new(stmt.clone())));
        }
        self.end_scope();
    }
}
