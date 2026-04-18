#![allow(dead_code, unused_variables, unused_imports, non_camel_case_types)]
//#![allow(warnings)]
mod lexer;
mod token;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::process;

// Assuming you have these modules defined in your Rust project
use crate::lexer::Lexer;
// use crate::parser::Parser;
// use crate::interpreter::Interpreter;

fn main() {
    // Collect command line arguments into a Vector
    let args: Vec<String> = env::args().collect();

    // In Rust, args[0] is the path to the executable itself.
    // So if the user passed a file, it will be at args[1].
    if args.len() > 1 {
        // Run the file, and if it fails, print the error and exit
        if let Err(e) = run_file(&args[1]) {
            eprintln!("Failed to execute file: {}", e);
            process::exit(1);
        }
    } else {
        // Run the REPL prompt
        if let Err(e) = run_prompt() {
            eprintln!("Failed to run prompt: {}", e);
            process::exit(1);
        }
    }
}

fn run_file(path: &str) -> io::Result<()> {
    // 1 & 2. Read all bytes and convert to a UTF-8 String in ONE step!
    let source = fs::read_to_string(path)?;

    // 3. Execute the source
    if let Err(e) = run(&source) {
        // In your Java code, you exited with 65 (Compile) or 70 (Runtime).
        // You can match on your custom error types here later to exit with specific codes.
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

    // Read lines in an infinite loop until EOF (Ctrl+D / Ctrl+Z)
    while reader.read_line(&mut line)? > 0 {
        if line.trim().eq_ignore_ascii_case("run") {
            // Execute the accumulated string
            if let Err(e) = run(&multi_line_source) {
                eprintln!("Error: {}", e);
            }

            println!("\n--- Script Executed. Enter new script or 'RUN' ---");
            multi_line_source.clear(); // Empty the buffer for the next script
        } else {
            // Rust's `read_line` keeps the newline character (`\n`) automatically!
            // So we just push the exact line into our source buffer.
            multi_line_source.push_str(&line);
        }

        // Clear the line buffer so the next read_line doesn't append to the old input
        line.clear();
    }

    Ok(())
}

/// The Rust equivalent of your `run` method.
/// Notice it returns a `Result<(), String>` instead of checking global flags.
fn run(source: &str) -> Result<(), String> {
    /* TODO: Uncomment and adapt this once your modules are written



    // 2. Syntax Analysis
    let mut parser = Parser::new(tokens);
    // If parse() fails, the `?` will immediately return the error
    let statements = parser.parse().map_err(|_| "Syntax Error occurred")?;

    // 3. Execution
    let mut interpreter = Interpreter::new();
    interpreter.interpret(&statements).map_err(|_| "Runtime Error occurred")?;

    */

    // 1. Lexical Analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens();

    println!("--- Scanned Tokens ---");
    for token in tokens {
        println!("{:#?}", token);
    }
    // For now, just print to verify the REPL works:
    //println!("Executing:\n{}", source);

    Ok(())
}
