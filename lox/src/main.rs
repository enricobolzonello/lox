use error::{report, Result};
use lox_interpreter::interpret;
use std::{
    fs,
    io::{self, BufRead, Write},
    process,
};

mod error;

use lox_syntax::{parse_expr, Lexer, TreePrinter};

fn run_file(path: String) -> Result<()> {
    let content = fs::read_to_string(path)?;
    run(&content);

    if error::had_error() {
        process::exit(65);
    }

    Ok(())
}

fn run_prompt() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut handle = stdin.lock();

    loop {
        print!("> ");
        stdout.flush()?;

        let mut line = String::new();
        let bytes_read = handle.read_line(&mut line)?;

        if bytes_read == 0 {
            // EOF (Ctrl+D or similar)
            break;
        }

        run(line.trim());
        error::reset();
    }

    Ok(())
}

fn run(code: &str) {
    println!("Running: \n{}\n", code);

    let mut scanner = Lexer::new(code);
    let tokens = match scanner.scan_tokens(){
        Ok(tok) => tok,
        Err(e) => {
            report(Box::new(e));
            Vec::new()
        }
    };

    let expression = match parse_expr(&tokens) {
        Ok(exp) => Some(exp),
        Err(e) => {
            report(Box::new(e));
            None
        }
    };

    if let Some(expression) = expression {
        let mut printer = TreePrinter::new();
        println!("AST pretty print: \n {} \n", printer.print(&expression));

        match interpret(&expression) {
            Ok(val) => println!("Evaluated expression: \n{}", val),
            Err(e) => report(Box::new(e)),
        }
    }
}

fn main() -> Result<()> {
    let mut arguments = std::env::args();
    if arguments.len() > 2 {
        println!("Usage: jlox [script]");
    } else if arguments.len() == 2 {
        run_file(arguments.nth(1).expect("no script argument"))?
    } else {
        run_prompt()?
    }

    Ok(())
}
