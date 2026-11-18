// BOLI - ek Hinglish programming language jo Rust me bani hai
// CLI: `cargo run -- run file.hl` se file chalao, `cargo run` se REPL khulega

mod ast;
mod interpreter;
mod lexer;
mod parser;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

use interpreter::Environment;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        // sirf `cargo run` - REPL kholo
        1 => repl(),
        2 => {
            let cmd = args[1].as_str();
            match cmd {
                "help" | "--help" | "-h" => print_help(),
                "version" | "--version" | "-v" => {
                    println!("BOLI version {}", env!("CARGO_PKG_VERSION"))
                }
                // convenience: `cargo run -- file.hl` bhi chalega
                _ if cmd.ends_with(".hl") => run_file(cmd),
                _ => {
                    eprintln!("Galti: '{}' samajh nahi aaya", cmd);
                    print_help();
                    process::exit(1);
                }
            }
        }
        3 if args[1] == "run" => run_file(&args[2]),
        _ => {
            eprintln!("Galti: galat command");
            print_help();
            process::exit(1);
        }
    }
}

// Kya kaam hai: help text
fn print_help() {
    println!("BOLI - Hinglish me programming language");
    println!();
    println!("Use:");
    println!("  cargo run              REPL kholo");
    println!("  cargo run -- run x.hl  x.hl file chalao");
    println!("  cargo run -- help      yeh message dikhao");
    println!("  cargo run -- version   version dikhao");
}

// File padhkar chalao
fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Galti: file '{}' nahi khuli: {}", path, e);
            process::exit(1);
        }
    };

    match parser::parse_source(&source) {
        Ok(program) => {
            let mut env = Environment::new();
            if let Err(e) = env.run_program(&program) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

// REPL - interactive shell
// Multi-line blocks (agar ... khatam) ko estimate karke jodta hai
fn repl() {
    println!("BOLI REPL - likhne ke liye tayyar!");
    println!("REPL se nikalne ke liye 'nikalo' likho.");

    let mut env = Environment::new();
    let mut buffer = String::new();
    let mut depth = 0i32; // kitne block khule hain (agar/jab tak/kaam/tak vs khatam)
    let stdin = io::stdin();

    loop {
        // Prompt: normal ya continuation
        if depth > 0 {
            print!("...> ");
        } else {
            print!("boli> ");
        }
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break, // EOF (Ctrl-D)
            Ok(_) => {}
            Err(_) => {
                eprintln!("Galti: input nahi padh paye");
                break;
            }
        }

        let trimmed = line.trim();

        // Block ke andar blank line chhod do
        if trimmed.is_empty() && depth > 0 {
            continue;
        }
        // REPL se nikalna
        if depth == 0 && (trimmed == "nikalo" || trimmed == "exit" || trimmed == "quit") {
            println!("BOLI band ho raha hai. Alvida!");
            break;
        }

        // Block depth badlo: agar/kaam/tak kholta hai, khatam band karta hai
        for word in trimmed.split(|c: char| c.is_whitespace()) {
            match word.to_lowercase().as_str() {
                "agar" | "kaam" | "tak" => depth += 1,
                "khatam" => depth -= 1,
                _ => {}
            }
        }

        buffer.push_str(&line);

        // Block poora nahi hua to aur lines lo
        if depth > 0 {
            continue;
        }

        // Pura code ab ready hai, chalao
        match parser::parse_source(&buffer) {
            Ok(program) => {
                if let Err(e) = env.run_program(&program) {
                    eprintln!("{}", e);
                }
            }
            Err(e) => eprintln!("{}", e),
        }
        buffer.clear();
        depth = 0;
    }
}
