use error::{report, Result};
use lox_interpreter::{Interpreter, Resolver};
use lox_std::set_stdlib;
use std::{
    cell::RefCell,
    fs,
    io::{self, BufRead, Write},
    process,
    rc::Rc,
};

use log::debug;

mod error;

use lox_syntax::{parse_program, Lexer, TreePrinter};

fn run_file(path: String) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let interpreter = Rc::new(RefCell::new(Interpreter::new()));
    set_stdlib(interpreter.clone());
    let had_runtime_error = run(&content, interpreter);

    if error::had_error() {
        process::exit(65);
    }

    if had_runtime_error {
        process::exit(70);
    }

    Ok(())
}

fn run_prompt() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut handle = stdin.lock();

    let interpreter = Rc::new(RefCell::new(Interpreter::new()));

    loop {
        print!("> ");
        stdout.flush()?;

        let mut line = String::new();
        let bytes_read = handle.read_line(&mut line)?;

        if bytes_read == 0 {
            // EOF (Ctrl+D or similar)
            break;
        }

        run(line.trim(), interpreter.clone());
        error::reset();
    }

    Ok(())
}

fn run(code: &str, interpreter: Rc<RefCell<Interpreter>>) -> bool {
    debug!("Running: \n{}\n", code);

    let mut scanner = Lexer::new(code);
    let tokens = match scanner.scan_tokens() {
        Ok(tok) => tok,
        Err(e) => {
            report(Box::new(e));
            return false;
        }
    };

    let statements = match parse_program(&tokens) {
        Ok(stmts) => stmts,
        Err(e) => {
            report(Box::new(e));
            return false;
        }
    };

    let mut resolver = Resolver::new(interpreter.clone());
    if let Err(error) = resolver.resolve_stmts(&statements) {
        eprintln!("{}", error);
        return true;
    }

    match interpreter.borrow_mut().interpret(&statements) {
        Ok(()) => false,
        Err(error) => {
            eprintln!("{}", error);
            true
        }
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
