use ast::Stmt;
use parser::Parser;

use crate::{
    errors::Result, Token,
};

pub(crate) mod ast;
pub(crate) mod ast_printer;
mod token_stream;
pub(crate) mod parser;

pub fn parse_program(tokens: &[Token]) -> Result<Vec<Stmt>> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}