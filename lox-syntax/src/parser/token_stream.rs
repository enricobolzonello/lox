use crate::tokenizer::{Token, TokenType};

pub(crate) struct TokenStream<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn peek_token(&self) -> &'a Token {
        match self.tokens.get(self.current) {
            Some(t) => t,
            None => &Token { token_type: TokenType::EOF, literal: None, line: 0 },
        }
    }

    pub fn peek(&self) -> TokenType {
        self.peek_token().into()
    }

    pub fn check(&self, token_type: TokenType) -> bool {
        self.peek() == token_type
    }

    pub fn advance(&mut self) -> &'a Token {
        let token = self.tokens.get(self.current);
        match token {
            Some(t) => {
                self.current += 1;
                t
            },
            None => {
                &Token{
                    token_type: TokenType::EOF,
                    literal: None,
                    line: 0,
                }
            },
        }
    }

    pub fn previous(&self) -> &'a Token {
        match self.tokens.get(self.current-1) {
            Some(t) => t,
            None => &Token { token_type: TokenType::INVALID, literal: None, line: 0 },
        }
    }

    pub fn match_tokens(&mut self, token_types: &'a [TokenType]) -> bool {
        for token_type in token_types {
            if self.check(*token_type) {
                let _ = self.advance();
                return true;
            }
        }

        false
    }
}