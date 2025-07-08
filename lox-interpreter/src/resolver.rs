use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

use lox_syntax::{Expr, ExprVisitor, Node, Stmt, StmtVisitor, Token};

use crate::{
    errors::{ControlFlow, Error, ResultExec},
    Interpreter,
};

#[derive(Clone, PartialEq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Clone, Copy, PartialEq)]
enum ClassType {
    None,
    Class
}

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    scopes: Vec<HashMap<String, (bool, bool)>>, // (is_defined, is_used)
    current_function: FunctionType,
    current_class: ClassType,
}

impl ExprVisitor<ResultExec<()>> for Resolver {
    fn visit_expr(&mut self, expr: &Expr) -> ResultExec<()> {
        match expr {
            Expr::Variable { name } => self.visit_var_expr(name),
            Expr::Assign { name, value } => self.visit_assign_expr(name, value),
            Expr::Binary { left, right, .. } => self.visit_binary_expr(left, right),
            Expr::Call {
                callee, arguments, ..
            } => self.visit_call_expr(callee, arguments),
            Expr::Grouping { expression } => self.visit_grouping_expr(expression),
            Expr::Literal { .. } => Ok(()),
            Expr::Logical { left, right, .. } => {
                self.resolve(&Node::Expr(left.clone()))?;
                self.resolve(&Node::Expr(right.clone()))?;
                Ok(())
            }
            Expr::Unary { right, .. } => {
                self.resolve(&Node::Expr(right.clone()))?;
                Ok(())
            }
            Expr::Get { object, .. } => {
                self.resolve(&Node::Expr(object.clone()))?;
                Ok(())
            }
            Expr::Set { object, value, .. } => {
                self.resolve(&Node::Expr(value.clone()))?;
                self.resolve(&Node::Expr(object.clone()))?;
                Ok(())
            }
            Expr::Super { keyword, .. } => {
                println!("here");
                self.resolve_local(keyword);
                Ok(())
            }
            Expr::This { keyword } => {
                if self.current_class == ClassType::None {
                    return Err(Error::invalid_context("Can't use 'this' outside of a class.", Some(keyword.clone())));
                }
                println!("here this");
                self.resolve_local(keyword);
                Ok(())
            }
            _ => Err(Error::unexpected_expr("unknown expression type", None)),
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
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::Print { expression } => self.visit_print_stmt(expression),
            Stmt::Return { value, .. } => self.visit_return_stmt(value),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
            Stmt::Class { name, methods, superclass} => self.visit_class_stmt(name, methods, superclass),
            _ => Err(Error::unexpected_stmt("unknown statement type", None)),
        }
    }
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve_stmts(&mut self, statements: &[Stmt]) -> Result<(), Error> {
        for stmt in statements {
            if let Err(ControlFlow::Error(e)) = self.resolve(&Node::Stmt(Box::new(stmt.clone()))) {
                return Err(e);
            }
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
        self.end_scope()?;
        Ok(())
    }

    fn visit_class_stmt(&mut self, c_name: &Token, methods: &Vec<Box<Stmt>>, superclass: &Option<Box<Expr>>) -> ResultExec<()> {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;

        self.declare(c_name);
        self.define(c_name);

        if let Some(superclass) = superclass {
            match superclass.deref() {
                Expr::Variable { name } => {
                    if name == c_name {
                        return Err(Error::unexpected_expr(
                            "A class can't inherit from itself.", 
                            Some(name.clone())
                        ));
                    }
                },
                _ => {
                    return Err(Error::unexpected_expr(
                        "Superclass declaration should be a variable", 
                        None
                    ));
                },
            }

            self.resolve(&Node::Expr(superclass.clone()))?;

            self.begin_scope();
            self.scopes
                .last_mut()
                .expect("Scope must exist")
                .insert("super".to_string(), (true, false));
        }

        self.begin_scope();
        self.scopes
            .last_mut()
            .expect("Scope must exist")
            .insert("this".to_string(), (true, false));

        for method in methods {
            if let Stmt::Function { params, body, name } = method.as_ref() {
                let declaration = if name.to_string() == "init" {
                    FunctionType::Initializer
                } else {
                    FunctionType::Method
                };
                self.resolve_function(params, body, declaration)?;
            } else {
                return Err(Error::unexpected_stmt(
                    "Should be a function", 
                    None
                ));
            }
        }

        self.end_scope()?;
        if superclass.is_some() {
            self.end_scope()?;
        }

        self.current_class = enclosing_class;

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

    fn visit_function_stmt(
        &mut self,
        name: &Token,
        parameters: &[Token],
        body: &[Stmt],
    ) -> ResultExec<()> {
        self.declare(name);
        self.define(name);

        self.resolve_function(parameters, body, FunctionType::Function)?;
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) -> ResultExec<()> {
        self.resolve(&Node::Expr(Box::new(expr.clone())))?;
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> ResultExec<()> {
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
        match self.current_function {
            FunctionType::None => {
                return Err(Error::unexpected_stmt(
                    "return statement outside of function",
                    None,
                ))
            }
            _ => {}
        }

        if let Some(value) = value {
            if self.current_function == FunctionType::Initializer {
                return Err(Error::unexpected_stmt(
                    "Can't return a value from an initializer.",
                    None,
                ));
            }
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
            && self
                .scopes
                .last()
                .unwrap()
                .get(&name.to_string())
                .map(|(defined, _)| !*defined)
                .unwrap_or(false)
        {
            return Err(Error::invalid_context(
                "Can't read local variable in its own initializer",
                Some(name.clone()),
            ));
        }

        // mark as used
        for scope in self.scopes.iter_mut().rev() {
            if let Some((_, used)) = scope.get_mut(&name.to_string()) {
                *used = true;
                break;
            }
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

    fn end_scope(&mut self) -> ResultExec<()> {
        if let Some(scope) = self.scopes.pop() {
            for (name, (defined, used)) in scope {
                if defined && !used  && name != "this" && name != "super" {
                    return Err(Error::unused_variable(name, None));
                }
            }
        }

        Ok(())
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.to_string(), (false, false));
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.last_mut().unwrap();
        if let Some((_, used)) = scope.get_mut(&name.to_string()) {
            *used = false;
        }
        scope.insert(name.to_string(), (true, false));
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
            println!("here 2");
            self.interpreter.borrow_mut().resolve(name, distance);
        }
    }

    fn resolve_function(
        &mut self,
        parameters: &[Token],
        body: &[Stmt],
        function_type: FunctionType,
    ) -> ResultExec<()> {
        let enclosing_function = self.current_function.clone();
        self.current_function = function_type;
        self.begin_scope();
        for param in parameters {
            self.declare(param);
            self.define(param);
        }

        for stmt in body {
            self.resolve(&Node::Stmt(Box::new(stmt.clone())))?;
        }
        self.end_scope()?;
        self.current_function = enclosing_function;
        Ok(())
    }
}
