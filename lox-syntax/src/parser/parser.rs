use crate::{
    errors::{Error, Result},
    tokenizer::{Literal, Token, TokenType},
    Expr,
};

use super::{ast::Stmt, token_stream::TokenStream};

pub struct Parser<'a> {
    stream: TokenStream<'a>,
    errors: Vec<Error>,
    in_loop: bool,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            stream: TokenStream::new(tokens),
            errors: Vec::new(),
            in_loop: false,
        }
    }

    fn error(&mut self, token: &Token, message: impl std::fmt::Display) -> Error {
        let err = Error::parse_error(token.clone(), message);
        self.errors.push(err.clone());
        err
    }

    fn consume(
        &mut self,
        token_type: TokenType,
        message: impl std::fmt::Display,
    ) -> Option<&'a Token> {
        let token = match self.stream.check(token_type) {
            true => Some(self.stream.advance()),
            false => {
                self.error(self.stream.peek_token(), message);
                None
            }
        };

        token
    }

    // Synchronize the panic point
    fn synchronize(&mut self) {
        self.stream.advance();

        while self.stream.peek() != TokenType::EOF {
            // after a semicolon, we are done with the statement
            if self.stream.previous().token_type == TokenType::SEMICOLON {
                return;
            }

            // discard tokens until we have found a statement boundary
            match self.stream.advance().token_type {
                TokenType::CLASS
                | TokenType::FUN
                | TokenType::VAR
                | TokenType::FOR
                | TokenType::IF
                | TokenType::WHILE
                | TokenType::PRINT
                | TokenType::RETURN => return,
                _ => {}
            }
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.stream.is_eof() {
            match self.declaration() {
                Some(stmt) => statements.push(stmt),
                None => {
                    if self.has_errors() {
                        return Err(self.errors.remove(0));
                    }
                }
            }
        }

        Ok(statements)
    }

    // ----- Expression parsing methods -----

    fn expression(&mut self) -> Expr {
        self.comma()
    }

    fn comma(&mut self) -> Expr {
        let mut expr = self.assignment();

        while self.stream.match_tokens(&[TokenType::COMMA]) {
            let right = self.assignment();
            expr = Expr::Comma {
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        expr
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.or();

        if self.stream.match_tokens(&[TokenType::EQUAL]) {
            let equals = self.stream.previous();
            let value = self.assignment();

            match expr {
                Expr::Variable { name } => {
                    return Expr::Assign {
                        name,
                        value: Box::new(value),
                    }
                }
                _ => {
                    self.error(equals, "Invalid assignment target.");
                }
            }
        }

        expr
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();

        while self.stream.match_tokens(&[TokenType::OR]) {
            let operator = self.stream.previous();
            let right = self.and();
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: operator.clone(),
                right: Box::new(right),
            };
        }

        expr
    }

    fn and(&mut self) -> Expr {
        let mut expr = self.equality();

        while self.stream.match_tokens(&[TokenType::AND]) {
            let operator = self.stream.previous();
            let right = self.equality();
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: operator.clone(),
                right: Box::new(right),
            };
        }

        expr
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self
            .stream
            .match_tokens(&[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL])
        {
            let operator = self.stream.previous();
            let right = self.comparison();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator.clone(),
                right: Box::new(right),
            };
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        if self.stream.match_tokens(&[
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            let operator = self.stream.previous();
            self.error(
                &operator,
                format!("Missing left‐hand operand before '{}'", operator.token_type),
            );
            let _ = self.factor();
            return Expr::Literal {
                value: Literal::Null,
            };
        }

        let mut expr = self.term();

        while self.stream.match_tokens(&[
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            let operator = self.stream.previous();
            let right = self.term();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator.clone(),
                right: Box::new(right),
            };
        }

        expr
    }

    fn term(&mut self) -> Expr {
        if self
            .stream
            .match_tokens(&[TokenType::PLUS, TokenType::MINUS])
        {
            // We saw `+` or `-` at the start of term() → report
            let operator = self.stream.previous();
            self.error(
                &operator,
                format!("Missing left‐hand operand before '{}'", operator.token_type),
            );
            let _ = self.factor();
            return Expr::Literal {
                value: Literal::Null,
            };
        }

        let mut expr = self.factor();

        while self
            .stream
            .match_tokens(&[TokenType::MINUS, TokenType::PLUS])
        {
            let operator = self.stream.previous();
            let right = self.factor();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator.clone(),
                right: Box::new(right),
            };
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        if self
            .stream
            .match_tokens(&[TokenType::SLASH, TokenType::STAR])
        {
            let operator = self.stream.previous();
            self.error(
                &operator,
                format!("Missing left‐hand operand before '{}'", operator.token_type),
            );
            let _ = self.factor();
            return Expr::Literal {
                value: Literal::Null,
            };
        }

        let mut expr = self.unary();

        while self
            .stream
            .match_tokens(&[TokenType::SLASH, TokenType::STAR])
        {
            let operator = self.stream.previous();
            let right = self.unary();
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator.clone(),
                right: Box::new(right),
            };
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self
            .stream
            .match_tokens(&[TokenType::BANG, TokenType::MINUS])
        {
            let operator = self.stream.previous();
            let right = self.unary();
            return Expr::Unary {
                operator: operator.clone(),
                right: Box::new(right),
            };
        }

        self.call()
    }

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();

        loop {
            if self.stream.match_tokens(&[TokenType::LEFT_PAREN]) {
                expr = self.finish_call(expr)
            } else {
                break;
            }
        }

        expr
    }

    fn finish_call(&mut self, callee: Expr) -> Expr {
        let mut arguments = Vec::new();
        if !self.stream.check(TokenType::RIGHT_PAREN) {
            loop {
                if arguments.len() >= 255 {
                    self.error(
                        self.stream.peek_token(),
                        "Can't have more than 255 arguments.",
                    );
                }
                arguments.push(self.assignment());
                if !self.stream.match_tokens(&[TokenType::COMMA]) {
                    break;
                }
            }
        }

        let mut paren = &Token {
            token_type: TokenType::INVALID,
            literal: None,
            line: 0,
        };

        if let Some(token) = self.consume(TokenType::RIGHT_PAREN, "Expect ')' after arguments.") {
            paren = token;
        }

        let temp = Expr::Call {
            callee: Box::new(callee),
            paren: paren.clone(),
            arguments,
        };
        temp
    }

    fn primary(&mut self) -> Expr {
        if self.stream.match_tokens(&[TokenType::FUN]) {
            return self.lambda();
        }

        if self.stream.match_tokens(&[TokenType::FALSE]) {
            return Expr::Literal {
                value: Literal::Bool(false),
            };
        }
        if self.stream.match_tokens(&[TokenType::TRUE]) {
            return Expr::Literal {
                value: Literal::Bool(true),
            };
        }

        if self
            .stream
            .match_tokens(&[TokenType::NUMBER, TokenType::STRING])
        {
            let prev = &self.stream.previous().literal;
            return Expr::Literal {
                value: prev.clone().unwrap(),
            };
        }

        if self.stream.match_tokens(&[TokenType::IDENTIFIER]) {
            let prev = self.stream.previous();
            return Expr::Variable { name: prev.clone() };
        }

        if self.stream.match_tokens(&[TokenType::LEFT_PAREN]) {
            let expr = self.expression();
            if self.stream.match_tokens(&[TokenType::RIGHT_PAREN]) {
                return Expr::Grouping {
                    expression: Box::new(expr),
                };
            } else {
                // Error handling for missing closing parenthesis
                let token = self.stream.peek_token();
                self.error(token, "Expected ')' after expression.");

                // Try to recover by continuing with what we have
                return Expr::Grouping {
                    expression: Box::new(expr),
                };
            }
        }

        // Error handling for unexpected tokens
        let token = self.stream.peek_token();
        self.error(token, "Expected expression.");

        Expr::Literal {
            value: Literal::Null,
        }
    }

    fn lambda(&mut self) -> Expr {
        // parameters parsing
        self.consume(TokenType::LEFT_PAREN, "Expected '(' after function name");
        let mut params = Vec::new();
        if !self.stream.check(TokenType::RIGHT_PAREN) {
            loop {
                if params.len() >= 255 {
                    self.error(
                        self.stream.peek_token(),
                        "Can't have more than 255 parameters.",
                    );
                }

                let temp = self
                    .consume(TokenType::IDENTIFIER, "Expect parameter name.")
                    .unwrap();
                params.push(temp.clone());

                if !self.stream.match_tokens(&[TokenType::COMMA]) {
                    break;
                }
            }
        }

        self.consume(
            TokenType::RIGHT_PAREN,
            "Expected ')' after function parameters",
        );

        // body
        self.consume(TokenType::LEFT_BRACE, "Expected '{' before function body");
        let body = self.block();

        Expr::Lambda { params, body }
    }

    // ----- Statement parsing methods -----

    fn declaration(&mut self) -> Option<Stmt> {
        let result = if self.stream.match_tokens(&[TokenType::VAR]) {
            self.var_declaration()
        } else if self.stream.match_tokens(&[TokenType::FUN]) {
            self.fun_declaration("function")
        } else if self.stream.match_tokens(&[TokenType::CLASS]) {
            self.class_declaration()
        } else {
            self.statement()
        };

        // If we got an error (None), then synchronize
        if result.is_none() && self.has_errors() {
            self.synchronize();
        }

        result
    }

    fn statement(&mut self) -> Option<Stmt> {
        if self.stream.match_tokens(&[TokenType::FOR]) {
            return self.for_stmt();
        }

        if self.stream.match_tokens(&[TokenType::PRINT]) {
            return self.print_stmt();
        }

        if self.stream.match_tokens(&[TokenType::RETURN]) {
            return self.return_stmt();
        }

        if self.stream.match_tokens(&[TokenType::WHILE]) {
            return self.while_stmt();
        }

        if self.stream.match_tokens(&[TokenType::LEFT_BRACE]) {
            return Some(Stmt::Block {
                statements: self.block(),
            });
        }

        if self.stream.match_tokens(&[TokenType::IF]) {
            return self.if_stmt();
        }

        if self.stream.match_tokens(&[TokenType::BREAK]) {
            return self.break_stmt();
        }

        self.expr_stmt()
    }

    fn block(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.stream.is_eof() && !self.stream.check(TokenType::RIGHT_BRACE) {
            let dec = self.declaration();
            if let Some(d) = dec {
                statements.push(d);
            }
        }

        if self.stream.check(TokenType::RIGHT_BRACE) {
            self.stream.advance();
        } else {
            self.error(
                self.stream.peek_token(),
                "Expect '}' after block declaration.",
            );
        }

        return statements;
    }

    fn var_declaration(&mut self) -> Option<Stmt> {
        let name = match self.stream.check(TokenType::IDENTIFIER) {
            true => self.stream.advance(),
            false => {
                self.error(self.stream.peek_token(), "Expect variable name.");
                return None;
            }
        };

        let initializer = match self.stream.match_tokens(&[TokenType::EQUAL]) {
            true => Some(self.expression()),
            false => None,
        };

        if self.stream.check(TokenType::SEMICOLON) {
            self.stream.advance();
        } else {
            self.error(
                self.stream.peek_token(),
                "Expect ';' after variable declaration.",
            );
            return None;
        }

        Some(Stmt::Var {
            name: name.clone(),
            initializer,
        })
    }

    fn fun_declaration(&mut self, kind: &str) -> Option<Stmt> {
        // function name
        let name = self.consume(TokenType::IDENTIFIER, format!("Expect {} name.", kind))?;

        // parameters parsing
        self.consume(
            TokenType::LEFT_PAREN,
            format!("Expected '(' after {} name", kind),
        );
        let mut params = Vec::new();
        if !self.stream.check(TokenType::RIGHT_PAREN) {
            loop {
                if params.len() >= 255 {
                    self.error(
                        self.stream.peek_token(),
                        "Can't have more than 255 parameters.",
                    );
                }

                let temp = self.consume(TokenType::IDENTIFIER, "Expect parameter name.")?;
                params.push(temp.clone());

                if !self.stream.match_tokens(&[TokenType::COMMA]) {
                    break;
                }
            }
        }

        self.consume(
            TokenType::RIGHT_PAREN,
            "Expected ')' after function parameters",
        );

        // body
        self.consume(
            TokenType::LEFT_BRACE,
            format!("Expected '{{' before {} body", kind),
        );
        let body = self.block();

        Some(Stmt::Function {
            name: name.clone(),
            params,
            body,
        })
    }

    fn class_declaration(&mut self) -> Option<Stmt> {
        let name = self.consume(TokenType::IDENTIFIER, "Expect class name.")?;
        self.consume(TokenType::LEFT_BRACE, "Expected '{' after class name");

        let mut methods = Vec::new();
        while !self.stream.check(TokenType::RIGHT_BRACE) && !self.stream.is_eof() {
            methods.push(Box::new(self.fun_declaration("method")?));
        }

        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after class body.");
        Some(Stmt::Class {
            name: name.clone(),
            superclass: None,
            methods,
        })
    }

    fn print_stmt(&mut self) -> Option<Stmt> {
        let expression = self.expression();

        self.consume(TokenType::SEMICOLON, "Expect ';' after expression.");
        Some(Stmt::Print { expression })
    }

    fn return_stmt(&mut self) -> Option<Stmt> {
        let keyword = self.stream.previous();

        let mut value = None;
        if !self.stream.check(TokenType::SEMICOLON) {
            value = Some(self.expression());
        }

        self.consume(TokenType::SEMICOLON, "Expect ';' after return value.");
        Some(Stmt::Return {
            keyword: keyword.clone(),
            value,
        })
    }

    fn while_stmt(&mut self) -> Option<Stmt> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'while'.");
        let condition = self.expression();

        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after condition.");
        let enclosing_loop = self.in_loop;
        self.in_loop = true;
        let body = self.statement();
        self.in_loop = enclosing_loop;

        Some(Stmt::While {
            condition,
            body: Box::new(body?),
        })
    }

    fn for_stmt(&mut self) -> Option<Stmt> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'for'.");
        let mut initializer = None;
        if self.stream.match_tokens(&[TokenType::SEMICOLON]) {
            initializer = None;
        } else if self.stream.match_tokens(&[TokenType::VAR]) {
            initializer = self.var_declaration();
        } else {
            initializer = self.expr_stmt();
        }

        let mut condition = None;
        if !self.stream.match_tokens(&[TokenType::SEMICOLON]) {
            condition = Some(self.expression());
        }

        self.consume(TokenType::SEMICOLON, "Except ';' after loop condition.");
        let mut increment = None;
        if !self.stream.match_tokens(&[TokenType::RIGHT_PAREN]) {
            increment = Some(self.expression());
        }

        self.consume(TokenType::RIGHT_PAREN, "Except ')' after for clauses.");
        let mut body = self.statement();

        if let Some(increment) = increment {
            body = Some(Stmt::Block {
                statements: vec![
                    body?,
                    Stmt::Expression {
                        expression: increment,
                    },
                ],
            });
        }

        let c = condition.unwrap_or(Expr::Literal {
            value: Literal::Bool(true),
        });

        body = Some(Stmt::While {
            condition: c,
            body: Box::new(body?),
        });

        if let Some(initializer) = initializer {
            body = Some(Stmt::Block {
                statements: vec![initializer, body?],
            });
        }

        body
    }

    fn if_stmt(&mut self) -> Option<Stmt> {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'if'.");
        let condition = self.expression();

        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after 'if'.");
        let then_branch = match self.statement() {
            Some(v) => Box::new(v),
            None => {
                self.error(self.stream.peek_token(), "Expect then block after 'if'.");
                return None;
            }
        };
        let mut else_branch = None;
        if self.stream.match_tokens(&[TokenType::ELSE]) {
            else_branch = match self.statement() {
                Some(v) => Some(Box::new(v)),
                None => {
                    self.error(self.stream.peek_token(), "Expect then block after 'else'.");
                    return None;
                }
            };
        }

        Some(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn expr_stmt(&mut self) -> Option<Stmt> {
        let expression = self.expression();

        self.consume(TokenType::SEMICOLON, "Expect ';' after expression.");
        Some(Stmt::Expression { expression })
    }

    fn break_stmt(&mut self) -> Option<Stmt> {
        self.consume(TokenType::SEMICOLON, "Expect ';' after break.");
        if !self.in_loop {
            self.error(
                self.stream.peek_token(),
                "Cannot use 'break' outside of a loop.",
            );
            return None;
        }

        Some(Stmt::Break)
    }
}
