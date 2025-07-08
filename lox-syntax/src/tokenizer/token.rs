use std::fmt::Display;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,

    // Keywords.
    AND,
    BREAK,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
    INVALID,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f32),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: Option<Literal>,
    pub line: usize,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TokenType::LEFT_PAREN => write!(f, "("),
            TokenType::RIGHT_PAREN => write!(f, ")"),
            TokenType::LEFT_BRACE => write!(f, "{{"),
            TokenType::RIGHT_BRACE => write!(f, "}}"),
            TokenType::COMMA => write!(f, ","),
            TokenType::DOT => write!(f, "."),
            TokenType::MINUS => write!(f, "-"),
            TokenType::PLUS => write!(f, "+"),
            TokenType::SEMICOLON => write!(f, ";"),
            TokenType::SLASH => write!(f, "/"),
            TokenType::STAR => write!(f, "*"),
            TokenType::BANG => write!(f, "!"),
            TokenType::BANG_EQUAL => write!(f, "!="),
            TokenType::EQUAL => write!(f, "="),
            TokenType::EQUAL_EQUAL => write!(f, "=="),
            TokenType::GREATER => write!(f, ">"),
            TokenType::GREATER_EQUAL => write!(f, ">="),
            TokenType::LESS => write!(f, "<"),
            TokenType::LESS_EQUAL => write!(f, "<="),
            TokenType::IDENTIFIER => write!(f, "Identifier"),
            TokenType::STRING => write!(f, "String"),
            TokenType::NUMBER => write!(f, "Number"),
            TokenType::AND => write!(f, "&&"),
            TokenType::BREAK => write!(f, "Break"),
            TokenType::CLASS => write!(f, "Class"),
            TokenType::ELSE => write!(f, "Else"),
            TokenType::FALSE => write!(f, "False"),
            TokenType::FUN => write!(f, "Function"),
            TokenType::FOR => write!(f, "For"),
            TokenType::IF => write!(f, "If"),
            TokenType::NIL => write!(f, "Nil"),
            TokenType::OR => write!(f, "Or"),
            TokenType::PRINT => write!(f, "Print"),
            TokenType::RETURN => write!(f, "Return"),
            TokenType::SUPER => write!(f, "Super"),
            TokenType::THIS => write!(f, "This"),
            TokenType::TRUE => write!(f, "True"),
            TokenType::VAR => write!(f, "Var"),
            TokenType::WHILE => write!(f, "While"),
            TokenType::EOF => write!(f, "Eof"),
            TokenType::INVALID => write!(f, "Invalid"),
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(s) => write!(f, "{}", s),
            Literal::Number(x) => write!(f, "{}", x),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Null => write!(f, "null"),
        }
    }
}

impl Into<TokenType> for &Token {
    fn into(self) -> TokenType {
        self.token_type
    }
}

impl ToString for Token {
    fn to_string(&self) -> String {
        if self.token_type == TokenType::THIS {
            return "this".to_string();
        }

        if self.token_type == TokenType::SUPER {
            return "super".to_string();
        }

        match &self.literal {
            Some(Literal::String(s)) => s.clone(),
            Some(Literal::Number(n)) => n.to_string(),
            Some(Literal::Bool(b)) => b.to_string(),
            Some(Literal::Null) => "null".to_string(),
            None => "".to_string(),
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.token_type == other.token_type && self.literal == other.literal
    }
}

