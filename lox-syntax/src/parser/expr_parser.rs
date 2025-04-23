use crate::{
    errors::{Error, Result},
    tokenizer::{Literal, Token, TokenType},
};

use super::{ast::Expr, token_stream::TokenStream};

pub struct ExprParser<'a> {
    stream: TokenStream<'a>,
    errors: Vec<Error>,
}

impl<'a> ExprParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            stream: TokenStream::new(tokens),
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Expr> {
        let result = self.expression();

        if !self.errors.is_empty() {
            return Err(self.errors.remove(0));
        }

        Ok(result)
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
}

pub fn parse_expr(tokens: &[Token]) -> Result<Expr> {
    let mut parser = ExprParser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::token::{Literal, Token, TokenType};

    fn make_token(token_type: TokenType, literal: Option<Literal>) -> Token {
        Token {
            token_type,
            literal,
            line: 1,
        }
    }

    #[test]
    fn test_operator_precedence() {
        let tokens = vec![
            make_token(TokenType::NUMBER, Some(Literal::Number(1.0))),
            make_token(TokenType::PLUS, None),
            make_token(TokenType::NUMBER, Some(Literal::Number(2.0))),
            make_token(TokenType::STAR, None),
            make_token(TokenType::NUMBER, Some(Literal::Number(3.0))),
            make_token(TokenType::EOF, None),
        ];

        let expr = parse_expr(&tokens);

        match expr {
            Ok(Expr::Binary {
                left,
                operator,
                right,
            }) => {
                assert_eq!(operator.token_type, TokenType::PLUS);
                match *left {
                    Expr::Literal { value } => assert_eq!(value, Literal::Number(1.0)),
                    _ => panic!("Expected 1.0"),
                }
                match *right {
                    Expr::Binary {
                        left: l2,
                        operator: op2,
                        right: r2,
                    } => {
                        assert_eq!(op2.token_type, TokenType::STAR);
                        match *l2 {
                            Expr::Literal { value } => assert_eq!(value, Literal::Number(2.0)),
                            _ => panic!("Expected 2.0"),
                        }
                        match *r2 {
                            Expr::Literal { value } => assert_eq!(value, Literal::Number(3.0)),
                            _ => panic!("Expected 3.0"),
                        }
                    }
                    _ => panic!("Expected nested binary on the right"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_unary_expression() {
        let tokens = vec![
            make_token(TokenType::MINUS, None),
            make_token(TokenType::NUMBER, Some(Literal::Number(42.0))),
            make_token(TokenType::EOF, None),
        ];

        let expr = parse_expr(&tokens);

        match expr {
            Ok(Expr::Unary { operator, right }) => {
                assert_eq!(operator.token_type, TokenType::MINUS);
                match *right {
                    Expr::Literal { value } => assert_eq!(value, Literal::Number(42.0)),
                    _ => panic!("Expected 42.0"),
                }
            }
            _ => panic!("Expected unary expression"),
        }
    }

    #[test]
    fn test_grouping_expression() {
        let tokens = vec![
            make_token(TokenType::LEFT_PAREN, None),
            make_token(TokenType::NUMBER, Some(Literal::Number(3.0))),
            make_token(TokenType::RIGHT_PAREN, None),
            make_token(TokenType::EOF, None),
        ];

        let expr = parse_expr(&tokens);

        match expr {
            Ok(Expr::Grouping { expression }) => match *expression {
                Expr::Literal { value } => assert_eq!(value, Literal::Number(3.0)),
                _ => panic!("Expected literal inside grouping"),
            },
            _ => panic!("Expected grouping expression"),
        }
    }

    #[test]
    fn test_equality_expression() {
        let tokens = vec![
            make_token(TokenType::NUMBER, Some(Literal::Number(5.0))),
            make_token(TokenType::EQUAL_EQUAL, None),
            make_token(TokenType::NUMBER, Some(Literal::Number(5.0))),
            make_token(TokenType::EOF, None),
        ];

        let expr = parse_expr(&tokens);

        match expr {
            Ok(Expr::Binary { operator, .. }) => {
                assert_eq!(operator.token_type, TokenType::EQUAL_EQUAL)
            }
            _ => panic!("Expected equality binary expression"),
        }
    }

    #[test]
    fn test_comparison_expression() {
        let tokens = vec![
            make_token(TokenType::NUMBER, Some(Literal::Number(10.0))),
            make_token(TokenType::LESS, None),
            make_token(TokenType::NUMBER, Some(Literal::Number(20.0))),
            make_token(TokenType::EOF, None),
        ];

        let expr = parse_expr(&tokens);

        match expr {
            Ok(Expr::Binary { operator, .. }) => assert_eq!(operator.token_type, TokenType::LESS),
            _ => panic!("Expected comparison binary expression"),
        }
    }

    #[test]
    fn test_literal_expression_true() {
        let tokens = vec![
            make_token(TokenType::TRUE, None),
            make_token(TokenType::EOF, None),
        ];

        let expr = parse_expr(&tokens);

        match expr {
            Ok(Expr::Literal { value }) => assert_eq!(value, Literal::Bool(true)),
            _ => panic!("Expected literal true"),
        }
    }
}
