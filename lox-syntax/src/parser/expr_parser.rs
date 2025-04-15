use crate::tokenizer::{Literal, Token, TokenType};

use super::{ast::Expr, token_stream::TokenStream};

pub struct ExprParser<'a> {
    stream: TokenStream<'a>,
}

impl<'a> ExprParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            stream: TokenStream::new(tokens),
        }
    }

    fn expression(&mut self) -> Expr {
        self.equality()
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
            if self.stream.check(TokenType::RIGHT_PAREN) {
                let _ = self.stream.advance();
            } else {
                // error
                todo!()
            }

            return Expr::Grouping {
                expression: Box::new(expr),
            };
        }

        Expr::Literal {
            value: Literal::Null,
        }
    }
}

fn parse_expr(tokens: &[Token]) -> Expr {
    let mut parser = ExprParser::new(tokens);
    parser.expression()
}

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
            Expr::Binary {
                left,
                operator,
                right,
            } => {
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
            Expr::Unary { operator, right } => {
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
            Expr::Grouping { expression } => match *expression {
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
            Expr::Binary { operator, .. } => {
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
            Expr::Binary { operator, .. } => assert_eq!(operator.token_type, TokenType::LESS),
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
            Expr::Literal { value } => assert_eq!(value, Literal::Bool(true)),
            _ => panic!("Expected literal true"),
        }
    }
}
