use crate::{
    errors::{Error, Result},
    tokenizer::{Literal, Token, TokenType}, Expr,
};

use super::{ast::Stmt, token_stream::TokenStream};

pub struct Parser<'a> {
    stream: TokenStream<'a>,
    errors: Vec<Error>
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            stream: TokenStream::new(tokens),
            errors: Vec::new(),
        }
    }

    fn error(&mut self, token: &Token, message: impl std::fmt::Display) -> Error {
        let err = Error::parse_error(token.clone(), message);
        self.errors.push(err.clone());
        err
    }

    // Synchronize the panic point
    fn _synchronize(&mut self) {
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
            statements.push(self.statement().unwrap()); // TODO: handle unwrap better
        }

        Ok(statements)
    }


    // ----- Expression parsing methods -----

    fn expression(&mut self) -> Expr {
        self.comma()
    }

    fn comma(&mut self) -> Expr {
        let mut expr = self.equality();

        while self.stream.match_tokens(&[TokenType::COMMA]) {
            let right = self.equality();
            expr = Expr::Comma {
                left: Box::new(expr),
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
        if self
            .stream
            .match_tokens(&[
                TokenType::GREATER,
                TokenType::GREATER_EQUAL,
                TokenType::LESS,
                TokenType::LESS_EQUAL,
            ])
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

        self.primary()
    }

    fn primary(&mut self) -> Expr {
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


    // ----- Statement parsing methods -----

    fn statement(&mut self) -> Option<Stmt> {
        if self.stream.match_tokens(&[TokenType::PRINT]) {
            return self.print_stmt();
        }

        self.expr_stmt()
    }

    fn print_stmt(&mut self) -> Option<Stmt> {
        let expression = self.expression();

        if self.stream.check(TokenType::SEMICOLON) {
            self.stream.advance();
        }else{
            self.error(self.stream.peek_token(), "Expect ';' after expression.");
            return None;
        }

        Some(Stmt::Print { expression })
    }

    fn expr_stmt(&mut self) -> Option<Stmt> {
        let expression = self.expression();

        if self.stream.check(TokenType::SEMICOLON) {
            self.stream.advance();
        }else{
            self.error(self.stream.peek_token(), "Expect ';' after expression.");
            return None;
        }

        Some(Stmt::Expression { expression })
    }
}