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
pub mod stdlib;
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
        } else if args[1] == "update" {
            update_system();
        } else if args[1] == "install" {
            if args.len() < 3 {
                eprintln!("Usage: fx install <pkg_name> (e.g. fx-math)");
                std::process::exit(1);
            }
            install_package(&args[2]);
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

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use std::io::{self, Write};

fn run_with_spinner(msg: &str, mut cmd: std::process::Command) -> std::process::Output {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let msg_clone = msg.to_string();
    
    let handle = thread::spawn(move || {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut i = 0;
        while running_clone.load(Ordering::Relaxed) {
            print!("\r\x1b[36m{}\x1b[0m {}", frames[i % frames.len()], msg_clone);
            let _ = io::stdout().flush();
            i += 1;
            thread::sleep(Duration::from_millis(80));
        }
        // clear the line when done
        print!("\r\x1b[K"); 
        let _ = io::stdout().flush();
    });

    let output = cmd.output().expect("Failed to execute command");
    
    running.store(false, Ordering::Relaxed);
    handle.join().unwrap();
    
    output
}

fn install_package(pkg_name: &str) {
    println!("Installing package {} from fx-pkgs...", pkg_name);
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let repo_dir = format!("{}/.fx/fx-pkgs-repo", home);
    let pkgs_dir = format!("{}/.fx/pkgs", home);
    let target_dir = format!("{}/{}", pkgs_dir, pkg_name);

    if !std::path::Path::new(&format!("{}/.fx", home)).exists() {
        fs::create_dir_all(format!("{}/.fx", home)).unwrap();
    }
    if !std::path::Path::new(&pkgs_dir).exists() {
        fs::create_dir_all(&pkgs_dir).unwrap();
    }

    if !std::path::Path::new(&repo_dir).exists() {
        let mut cmd = std::process::Command::new("git");
        cmd.arg("clone").arg("https://github.com/lozzadon/fx-pkgs.git").arg(&repo_dir);
        let output = run_with_spinner("Initializing local package cache...", cmd);
        
        if !output.status.success() {
            eprintln!("Failed to clone fx-pkgs repository.");
            std::process::exit(1);
        }
    } else {
        let mut cmd = std::process::Command::new("git");
        cmd.current_dir(&repo_dir).arg("pull");
        let output = run_with_spinner("Updating local package cache...", cmd);
        
        if !output.status.success() {
            eprintln!("Failed to update fx-pkgs repository.");
            std::process::exit(1);
        }
    }

    let source_dir = format!("{}/{}", repo_dir, pkg_name);
    if !std::path::Path::new(&source_dir).exists() {
        eprintln!("Package '{}' not found in fx-pkgs repository.", pkg_name);
        std::process::exit(1);
    }

    println!("Copying {} to ~/.fx/pkgs/{}...", pkg_name, pkg_name);
    let _ = std::process::Command::new("cp")
        .arg("-r")
        .arg(&source_dir)
        .arg(&pkgs_dir)
        .status();

    println!("Successfully installed {}!", pkg_name);
}

fn update_system() {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    
    // Update topia
    let topia_dir = format!("{}/topia", home);
    if std::path::Path::new(&topia_dir).exists() {
        let mut cmd = std::process::Command::new("git");
        cmd.current_dir(&topia_dir).arg("pull");
        let output = run_with_spinner("Updating topia...", cmd);
        
        if !output.status.success() {
            eprintln!("Failed to pull topia updates.");
        } else {
            println!("topia updated successfully.");
        }
    }

    // Update fx
    let fx_dir = format!("{}/fx", home);
    if std::path::Path::new(&fx_dir).exists() {
        let mut cmd = std::process::Command::new("git");
        cmd.current_dir(&fx_dir).arg("pull");
        let output = run_with_spinner("Updating fx...", cmd);
        
        if !output.status.success() {
            eprintln!("Failed to pull fx updates.");
        } else {
            println!("fx pulled successfully.");
            
            let mut build_cmd = std::process::Command::new("cargo");
            build_cmd.current_dir(&fx_dir).arg("install").arg("--path").arg(".");
            let build_output = run_with_spinner("Recompiling fx engine...", build_cmd);
                
            if !build_output.status.success() {
                eprintln!("Failed to recompile fx.");
            } else {
                println!("fx updated and installed successfully!");
                // Let's also copy it to ~/.local/bin/fx just in case their environment favors it
                let local_bin = format!("{}/.local/bin", home);
                if std::path::Path::new(&local_bin).exists() {
                    let target = format!("{}/fx", local_bin);
                    let _ = std::fs::remove_file(&target);
                    let _ = std::process::Command::new("cp")
                        .arg(format!("{}/target/release/fx", fx_dir))
                        .arg(target)
                        .status();
                }
            }
        }
    } else {
        eprintln!("Error: Could not find ~/fx repository.");
    }
}
