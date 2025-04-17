mod position;
pub(crate) mod token;

use std::{iter::Peekable, str::Chars};

use phf::phf_map;
use position::BytePos;

pub use crate::tokenizer::token::{Literal,Token,TokenType};

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map!(
    "and" => TokenType::AND,
    "class" => TokenType::CLASS,
    "else" => TokenType::ELSE,
    "false" => TokenType::FALSE,
    "for" => TokenType::FOR,
    "fun" => TokenType::FUN,
    "if" => TokenType::IF,
    "nil" => TokenType::NIL,
    "or" => TokenType::OR,
    "print" => TokenType::PRINT,
    "return" => TokenType::RETURN,
    "super" => TokenType::SUPER,
    "this" => TokenType::THIS,
    "true" => TokenType::TRUE,
    "var" => TokenType::VAR,
    "while" => TokenType::WHILE
);

// just iterator stuff, no token logic
struct Scanner<'a> {
    iter: Peekable<Chars<'a>>,
    current_position: BytePos,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            iter: source.chars().peekable(),
            current_position: BytePos::default(),
            line: 0,
        }
    }

    fn next(&mut self) -> Option<char> {
        let next = self.iter.next();
        if let Some(ch) = next {
            self.current_position = self.current_position.shift(ch);
        }
        next
    }

    fn peek(&mut self) -> Option<&char> {
        self.iter.peek()
    }

    fn consume_if<F>(&mut self, x: F) -> bool
    where
        F: Fn(char) -> bool,
    {
        if let Some(&next) = self.peek() {
            if x(next) {
                self.next().unwrap();
                return true;
            } else {
                return false;
            }
        }

        false
    }

    fn consume_while<F>(&mut self, x: F) -> Vec<char>
    where
        F: Fn(char) -> bool,
    {
        let mut values = Vec::new();
        while let Some(&next) = self.iter.peek() {
            if !x(next) {
                break;
            }
            let val = self.next().unwrap();
            values.push(val);
        }
        values
    }
}

// token logic goes here
pub struct Lexer<'a> {
    iter: Scanner<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            iter: Scanner::new(source),
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let _start = self.iter.current_position; // TODO: what to do with it?
            let ch = match self.iter.next() {
                Some(ch) => ch,
                None => break,
            };

            if let Some(token) = self.match_token(ch) {
                tokens.push(token);
            }
        }
        tokens
    }

    fn match_token(&mut self, ch: char) -> Option<Token> {
        match ch {
            '(' => self.create_token(TokenType::LEFT_PAREN, None),
            ')' => self.create_token(TokenType::RIGHT_PAREN, None),
            '{' => self.create_token(TokenType::LEFT_BRACE, None),
            '}' => self.create_token(TokenType::RIGHT_BRACE, None),
            ',' => self.create_token(TokenType::COMMA, None),
            '.' => self.create_token(TokenType::DOT, None),
            '-' => self.create_token(TokenType::MINUS, None),
            '+' => self.create_token(TokenType::PLUS, None),
            ';' => self.create_token(TokenType::SEMICOLON, None),
            '*' => self.create_token(TokenType::STAR, None),
            '!' => {
                let token_type = self.either('=', TokenType::BANG_EQUAL, TokenType::BANG);
                self.create_token(token_type, None)
            }
            '=' => {
                let token_type = self.either('=', TokenType::EQUAL_EQUAL, TokenType::EQUAL);
                self.create_token(token_type, None)
            }
            '<' => {
                let token_type = self.either('=', TokenType::LESS_EQUAL, TokenType::LESS);
                self.create_token(token_type, None)
            }
            '>' => {
                let token_type = self.either('=', TokenType::GREATER_EQUAL, TokenType::GREATER);
                self.create_token(token_type, None)
            }
            '/' => {
                if self.iter.consume_if(|ch| ch == '/') {
                    // single line comment
                    self.iter.consume_while(|ch| ch != '\n');
                    None
                } else if self.iter.consume_if(|ch| ch == '*') {
                    // multi line comment
                    self.multi_line_comment();
                    None
                } else {
                    // division
                    self.create_token(TokenType::SLASH, None)
                }
            }
            ' ' | '\r' | '\t' => None,
            '\n' => {
                self.iter.line += 1;
                None
            }
            '"' => {
                let chars = self.iter.consume_while(|ch| ch != '"');

                for c in &chars {
                    if *c == '\n' {
                        self.iter.line += 1;
                    }
                }

                if !self.iter.consume_if(|ch| ch == '"') {
                    // TODO: report error
                }

                let value = String::from_iter(chars);
                self.create_token(TokenType::STRING, Some(Literal::String(value)))
            }
            _ => {
                if ch.is_digit(10) {
                    // number literals
                    self.numbers(ch)
                } else if ch.is_alphabetic() {
                    // reserved words and identifiers
                    self.identifiers(ch)
                } else {
                    // TODO: report error
                    None
                }
            }
        }
    }

    fn create_token(&self, token_type: TokenType, literal: Option<Literal>) -> Option<Token> {
        Some(Token {
            token_type,
            literal,
            line: self.iter.line,
        })
    }

    fn either(&mut self, to_match: char, matched: TokenType, unmatched: TokenType) -> TokenType {
        if self.iter.consume_if(|ch| ch == to_match) {
            matched
        } else {
            unmatched
        }
    }

    fn multi_line_comment(&mut self) {
        let mut comment_count = 1;

        while comment_count > 0 {
            if self.iter.peek().is_none() {
                // todo: error
                break;
            }

            // nested comment start '/*'
            if let Some(&'/') = self.iter.peek() {
                self.iter.next(); // Consume '/'
                if self.iter.consume_if(|ch| ch == '*') {
                    comment_count += 1;
                    continue;
                }
            }
            // comment end '*/'
            else if let Some(&'*') = self.iter.peek() {
                self.iter.next(); // Consume '*'
                if self.iter.consume_if(|ch| ch == '/') {
                    comment_count -= 1;
                    continue;
                }
            } else if let Some(&'\n') = self.iter.peek() {
                self.iter.next(); // Consume '\n'
                self.iter.line += 1;
                continue;
            }

            self.iter.next();
        }
    }

    fn numbers(&mut self, first_ch: char) -> Option<Token> {
        let mut number = String::from(first_ch);
        number.push_str(
            &self
                .iter
                .consume_while(|ch| ch.is_digit(10))
                .into_iter()
                .collect::<String>(),
        );

        if self.iter.consume_if(|ch| ch == '.')
            && self.iter.peek().map_or(false, |&ch| ch.is_digit(10))
        {
            number.push('.');
            number.push_str(
                &self
                    .iter
                    .consume_while(|ch| ch.is_digit(10))
                    .into_iter()
                    .collect::<String>(),
            );
        }

        match number.parse::<f32>() {
            Ok(value) => self.create_token(TokenType::STRING, Some(Literal::Number(value))),
            Err(_) => {
                todo!(); // report error
                None
            }
        }
    }

    fn identifiers(&mut self, first_ch: char) -> Option<Token> {
        let mut identifier = String::from(first_ch);
        identifier.push_str(
            &self
                .iter
                .consume_while(|ch| ch.is_alphanumeric())
                .into_iter()
                .collect::<String>(),
        );

        let token_type = KEYWORDS
            .get(&identifier)
            .copied()
            .unwrap_or(TokenType::IDENTIFIER);

        let mut literal = None;
        if token_type == TokenType::IDENTIFIER {
            literal = Some(Literal::String(identifier));
        }

        self.create_token(token_type, literal)
    }
}
