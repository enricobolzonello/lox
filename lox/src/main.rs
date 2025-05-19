use error::{report, Result};
use lox_interpreter::Interpreter;
use std::{
    fs,
    io::{self, BufRead, Write},
    process,
};

use log::debug;

mod error;

use lox_syntax::{parse_program, Lexer, TreePrinter};

fn run_file(path: String) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let mut interpreter = Interpreter::new();
    run(&content, &mut interpreter);

    if error::had_error() {
        process::exit(65);
    }

    Ok(())
}

fn run_prompt() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut handle = stdin.lock();

    let mut interpreter = Interpreter::new();

    loop {
        print!("> ");
        stdout.flush()?;

        let mut line = String::new();
        let bytes_read = handle.read_line(&mut line)?;

        if bytes_read == 0 {
            // EOF (Ctrl+D or similar)
            break;
        }

        run(line.trim(), &mut interpreter);
        error::reset();
    }

    Ok(())
}

fn run(code: &str, interpreter: &mut Interpreter) {
    debug!("Running: \n{}\n", code);

    let mut scanner = Lexer::new(code);
    let tokens = match scanner.scan_tokens() {
        Ok(tok) => tok,
        Err(e) => {
            report(Box::new(e));
            Vec::new()
        }
    };

    let statements = match parse_program(&tokens) {
        Ok(stmts) => Some(stmts),
        Err(e) => {
            report(Box::new(e));
            None
        }
    };

    if let Some(statements) = statements {
        let mut printer = TreePrinter::new();
        printer.print_program(&statements);

        interpreter.interpret(&statements).unwrap();
    }
}

fn main() -> Result<()> {
    env_logger::init();

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
