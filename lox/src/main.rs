use std::{fs, io::{self, BufRead, Write}, process};
use error::Result;

mod error;

use lox_syntax::{parse_expr, Lexer, TreePrinter};

fn run_file(path: String) -> Result<()>{
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
    let tokens = scanner.scan_tokens();
    println!("{:?}", tokens);

    println!("\n\n\n\n");

    let expr = parse_expr(&tokens);
    let mut tree_printer = TreePrinter::new();
    println!("{:?}", expr);
}

fn main() -> Result<()> {
    let mut arguments = std::env::args();
    if arguments.len() > 2 {
        println!("Usage: jlox [script]");
    }else if arguments.len() == 2 {
        run_file(arguments.nth(1).expect("no script argument"))?
    }else{
        run_prompt()?
    }

    Ok(())
}
