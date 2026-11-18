// BOLI - ek Hinglish programming language jo Rust me bani hai
// cargo run se REPL, `cargo run -- run x.hl` se file chale
use boli::interpreter::Environment;
use boli::limits::{DEFAULT_MAX_SOURCE_BYTES, env_limit};
use boli::parser;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

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
    // source file size limit (env BOLI_MAX_SOURCE_BYTES, default 10 MiB)
    let max_source = env_limit("BOLI_MAX_SOURCE_BYTES", DEFAULT_MAX_SOURCE_BYTES);
    match fs::metadata(path) {
        Ok(meta) => {
            if meta.len() > max_source {
                eprintln!(
                    "Galti: file '{}' bahut badi hai ({} bytes, max {})",
                    path,
                    meta.len(),
                    max_source
                );
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Galti: file '{}' nahi khuli: {}", path, e);
            process::exit(1);
        }
    }
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

        // Block depth badlo: agar/jab/tak/kaam-definition kholte hain, khatam band karta hai
        // comment (#) aur quoted text nikaal ke dekho
        let code_part = trimmed.split('#').next().unwrap_or("");
        let mut stripped = String::with_capacity(code_part.len());
        let mut in_text = false;
        for c in code_part.chars() {
            if c == '"' {
                in_text = !in_text;
                stripped.push(' ');
            } else if !in_text {
                stripped.push(c);
            }
        }
        let code_part = stripped.trim();
        let words: Vec<String> = code_part
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        let mut stray = false;
        if let Some(first) = words.first() {
            match first.as_str() {
                "agar" | "jab" => depth += 1, // agar / jab tak
                "kaam" => match words.get(1).map(|s| s.as_str()) {
                    // "kaam rakho x" jaisi variable lines block nahi kholti
                    Some("rakho") => {}
                    // kaam 5 / kaam "x" / kaam sahi => call statement, def nahi
                    Some(w)
                        if w.starts_with('"')
                            || w.starts_with(|c: char| c.is_ascii_digit())
                            || w == "sahi"
                            || w == "galat"
                            || w == "khaali" => {}
                    Some(_) => depth += 1, // kaam <naam> => definition
                    None => {}
                },
                _ => {}
            }
            // range loop: last shabd "tak" ho to block khulta hai
            // ("jab tak ..." pehle hi count ho chuka hai, dobara mat jodo)
            if words.len() > 1
                && words.last().map(|s| s.as_str()) == Some("tak")
                && words.first().map(|s| s.as_str()) != Some("jab")
            {
                depth += 1;
            }
            // har "khatam" ek block band karta hai; bina block ke khatam = Galti, line ignore karo
            for word in &words {
                if word == "khatam" {
                    depth -= 1;
                    if depth < 0 {
                        depth = 0;
                        eprintln!("Galti: 'khatam' bina kuch shuru kiye aaya");
                        buffer.clear();
                        stray = true;
                        break;
                    }
                }
            }
        }
        if stray {
            continue;
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
