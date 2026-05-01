#![allow(dead_code, unused_variables, unused_imports, non_camel_case_types)]
//#![allow(warnings)]
mod environment;
mod error;
mod expr;
mod interpreter;
mod lexer;
mod parser;
mod stmt;
mod token;
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        if let Err(e) = run_file(&args[1]) {
            eprintln!("Failed to execute file: {}", e);
            process::exit(1);
        }
    } else {
        if let Err(e) = run_prompt() {
            eprintln!("Failed to run prompt: {}", e);
            process::exit(1);
        }
    }
}

fn run_file(path: &str) -> io::Result<()> {
    let source = fs::read_to_string(path)?;
    if let Err(e) = run(&source) {
        eprintln!("Error: {}", e);
        process::exit(65);
    }

    Ok(())
}

fn run_prompt() -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut multi_line_source = String::new();

    println!("--- Enter your LEXOR script (type 'RUN' on a new line to execute)");

    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        if line.trim().eq_ignore_ascii_case("run") {
            if let Err(e) = run(&multi_line_source) {
                eprintln!("Error: {}", e);
            }

            println!("\n--- Script Executed. Enter new script or 'RUN' ---");
            multi_line_source.clear();
        } else {
            multi_line_source.push_str(&line);
        }

        line.clear();
    }

    Ok(())
}

fn run(source: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens();

    let mut parser = Parser::new(tokens);

    // uncomment to see tokens and AST
    // println!("--- Scanned Tokens ---");
    // for token in &tokens { println!("{:?}", token); }
    // println!("\n--- Abstract Syntax Tree (AST) ---");
    // println!("{:#?}", statements);

    //
    let statements = match parser.parse() {
        Some(stmts) => stmts,
        None => return Err("Syntax Errors detected. Execution aborted.".to_string()),
    };

    println!("\n--- Program Output ---");
    let mut interpreter = Interpreter::new();

    interpreter
        .interpret(&statements)
        .map_err(|e| e.to_string())?;

    Ok(())
}
