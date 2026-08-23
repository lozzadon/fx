mod ast;
mod evaluator;
mod lexer;
mod object;
mod parser;
mod token;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::rc::Rc;
use std::cell::RefCell;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::object::Environment;
use crate::evaluator::eval_program;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Run a file
        let filename = &args[1];
        run_file(filename);
    } else {
        // Start REPL
        start_repl();
    }
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

fn start_repl() {
    println!("Welcome to the f(x) programming language!");
    println!("We are now running live! Try storing a variable and then typing its name.");
    println!("Example: \n  >> let my_score = 100 \n  >> my_score");
    println!("(Type 'exit' to quit)");

    let stdin = io::stdin();
    let env = Rc::new(RefCell::new(Environment::new()));
    
    loop {
        print!(">> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                if input.trim() == "exit" {
                    break;
                }
                
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

                let result = eval_program(program, Rc::clone(&env));
                
                match result {
                    crate::object::Object::Null => {},
                    _ => println!("{}", result),
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}
