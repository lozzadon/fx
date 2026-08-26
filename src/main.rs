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
        } else if args[1] == "store" {
            open_store();
        } else if args[1] == "update" {
            update_system();
        } else if args[1] == "uninstall" {
            if args.len() < 3 {
                eprintln!("Usage: fx uninstall <pkg_name> (e.g. fx-math)");
                std::process::exit(1);
            }
            uninstall_package(&args[2]);
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
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let repo_dir = format!("{}/.fx/fx-pkgs-repo", home);
    let pkgs_dir = format!("{}/fx/packages", home);
    let target_dir = format!("{}/{}", pkgs_dir, pkg_name);

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

    let msg = format!("Copying {} to ~/fx/packages/{}...", pkg_name, pkg_name);
    let mut cmd = std::process::Command::new("cp");
    cmd.arg("-r").arg(&source_dir).arg(&pkgs_dir);
    let _output = run_with_spinner(&msg, cmd);

    println!("Successfully installed {}!", pkg_name);
}

fn update_system() {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    
    // Update fx-pkgs
    let repo_dir = format!("{}/.fx/fx-pkgs-repo", home);
    let pkgs_dir = format!("{}/fx/packages", home);
    if std::path::Path::new(&repo_dir).exists() {
        let mut cmd = std::process::Command::new("git");
        cmd.current_dir(&repo_dir).arg("pull");
        let output = run_with_spinner("Updating fx-pkgs repository...", cmd);
        if !output.status.success() {
            eprintln!("Failed to update fx-pkgs repository.");
        } else {
            // Re-copy any installed packages
            if let Ok(entries) = std::fs::read_dir(&pkgs_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(pkg_name) = path.file_name().and_then(|s| s.to_str()) {
                            let source_dir = format!("{}/{}", repo_dir, pkg_name);
                            if std::path::Path::new(&source_dir).exists() {
                                let msg = format!("Updating package {}...", pkg_name);
                                let mut cp_cmd = std::process::Command::new("cp");
                                cp_cmd.arg("-r").arg(&source_dir).arg(&pkgs_dir);
                                run_with_spinner(&msg, cp_cmd);
                            }
                        }
                    }
                }
            }
            println!("fx-pkgs updated successfully.");
        }
    }

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
            
            let cargo_bin = format!("{}/.cargo/bin/fx", home);
            if std::path::Path::new(&cargo_bin).exists() {
                let _ = std::fs::remove_file(&cargo_bin);
            }
            
            let mut build_cmd = std::process::Command::new("cargo");
            build_cmd.current_dir(&fx_dir).arg("install").arg("--path").arg(".");
            let build_output = run_with_spinner("Recompiling fx engine...", build_cmd);
                
            if !build_output.status.success() {
                eprintln!("Failed to recompile fx.");
            } else {
                println!("fx updated and installed successfully!");
            }
        }
    } else {
        eprintln!("Error: Could not find ~/fx repository.");
    }
}

fn uninstall_package(pkg_name: &str) {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let pkgs_dir = format!("{}/fx/packages", home);
    let target_dir = format!("{}/{}", pkgs_dir, pkg_name);

    if !std::path::Path::new(&target_dir).exists() {
        eprintln!("Package '{}' is not installed.", pkg_name);
        std::process::exit(1);
    }
    
    let msg = format!("Uninstalling {}...", pkg_name);
    let mut cmd = std::process::Command::new("rm");
    cmd.arg("-rf").arg(&target_dir);
    let _output = run_with_spinner(&msg, cmd);

    println!("Successfully uninstalled {}!", pkg_name);
}

fn open_store() {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let repo_dir = format!("{}/.fx/fx-pkgs-repo", home);

    if !std::path::Path::new(&repo_dir).exists() {
        println!("Fetching store directory...");
        let _ = std::process::Command::new("git")
            .arg("clone")
            .arg("https://github.com/lozzadon/fx-pkgs.git")
            .arg(&repo_dir)
            .status();
    } else {
        let _ = std::process::Command::new("git")
            .current_dir(&repo_dir)
            .arg("pull")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    let mut packages = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&repo_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(pkg_name) = path.file_name().and_then(|s| s.to_str()) {
                    if !pkg_name.starts_with('.') {
                        packages.push(pkg_name.to_string());
                    }
                }
            }
        }
    }

    if packages.is_empty() {
        println!("No packages found in the store.");
        return;
    }

    packages.sort();

    use dialoguer::{theme::ColorfulTheme, Select};

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a package to install")
        .default(0)
        .items(&packages)
        .interact()
        .unwrap();

    let selected_pkg = &packages[selection];
    println!("\nInstalling {}...", selected_pkg);
    install_package(selected_pkg);
}
