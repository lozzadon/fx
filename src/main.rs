mod ast;
mod evaluator;
mod lexer;
mod object;
mod parser;
mod token;
mod code;
mod compiler;
mod vm;
mod formatter;
#[cfg(test)]
mod tests;

use std::env;
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::object::Environment;
use crate::evaluator::eval_program;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        if args[1] == "fmt" {
            if args.len() < 3 {
                eprintln!("Usage: fx fmt <file.fx>");
                std::process::exit(1);
            }
            format_file(&args[2]);
        } else if args[1] == "--vm" {
            if args.len() < 3 {
                eprintln!("Usage: fx --vm <file.fx>");
                std::process::exit(1);
            }
            run_file_vm(&args[2]);
        } else {
            // Run a file
            let filename = &args[1];
            run_file(filename);
        }
    } else {
        // Start REPL
        start_repl();
    }
}

fn run_file_vm(filename: &str) {
    let contents = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading {}: {}", filename, err);
        std::process::exit(1);
    });

    let lexer = Lexer::new(&contents);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        eprintln!("Cannot compile {}; there are syntax errors:", filename);
        for msg in parser.errors {
            eprintln!("\t{}", msg);
        }
        std::process::exit(1);
    }

    let mut compiler = compiler::Compiler::new();
    if let Err(err) = compiler.compile(&program) {
        eprintln!("Compiler error: {}", err);
        std::process::exit(1);
    }

    let mut machine = vm::VM::new(compiler.bytecode());
    if let Err(err) = machine.run() {
        eprintln!("VM error: {}", err);
        std::process::exit(1);
    }

    if let Some(result) = machine.last_popped_elem() {
        if *result != crate::object::Object::Null {
            println!("{}", result);
        }
    }
}

fn format_file(filename: &str) {
    let contents = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading {}: {}", filename, err);
        std::process::exit(1);
    });

    let lexer = Lexer::new(&contents);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        eprintln!("Cannot format {}; there are syntax errors:", filename);
        for msg in parser.errors {
            eprintln!("\t{}", msg);
        }
        std::process::exit(1);
    }

    let formatted = formatter::format_program(&program);
    
    if let Err(err) = fs::write(filename, formatted) {
        eprintln!("Error writing formatted file {}: {}", filename, err);
        std::process::exit(1);
    }
    
    println!("Successfully formatted {}", filename);
}

fn run_file(filename: &str) {
    let contents = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading {}: {}", filename, err);
        std::process::exit(1);
    });

    let env = Rc::new(RefCell::new(Environment::new()));
    let lexer = Lexer::new(&contents);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        eprintln!("Woops! We ran into some parser errors in {}:", filename);
        for msg in parser.errors {
            eprintln!("\t{}", msg);
        }
        std::process::exit(1);
    }

    let result = eval_program(program, env);
    match result {
        crate::object::Object::Null => {},
        crate::object::Object::Error(msg) => {
            eprintln!("Runtime Error: {}", msg);
            std::process::exit(1);
        }
        _ => println!("{}", result),
    }
}

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn start_repl() {
    println!("Welcome to the f(x) programming language!");
    println!("We are now running live! Try storing a variable and then typing its name.");
    println!("Example: \n  >> let my_score = 100 \n  >> my_score");
    println!("(Type 'exit' to quit)");

    let mut rl = DefaultEditor::new().expect("Failed to initialize rustyline");
    let history_path = env::var("HOME").map(|h| format!("{}/.fx_history", h)).unwrap_or_else(|_| ".fx_history".to_string());
    
    if rl.load_history(&history_path).is_err() {
        println!("No previous history.");
    }

    let env_ctx = Rc::new(RefCell::new(Environment::new()));
    
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }
                
                if input == "exit" {
                    break;
                }
                
                rl.add_history_entry(input).unwrap();
                
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program();

                if !parser.errors.is_empty() {
                    println!("Woops! We ran into some parser errors:");
                    for msg in parser.errors {
                        println!("\t{}", msg);
                    }
                    continue;
                }

                let result = eval_program(program, Rc::clone(&env_ctx));
                
                match result {
                    crate::object::Object::Null => {},
                    _ => println!("{}", result),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    
    if let Err(err) = rl.save_history(&history_path) {
        println!("Failed to save history: {}", err);
    }
}
