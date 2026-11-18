// BOLI - ek Hinglish programming language jo Rust me bani hai
// cargo run se REPL, `cargo run -- run x.hl` se file chale
use boli::interpreter::Environment;

// stdout crash nahi hona chahiye (broken pipe -> clean exit), print!/println! se EPIPE panic hotas
fn out_line(text: &str) -> bool {
    writeln!(io::stdout(), "{}", text).is_ok()
}

fn out(text: &str) -> bool {
    write!(io::stdout(), "{}", text).is_ok() && io::stdout().flush().is_ok()
}
use boli::limits::{DEFAULT_MAX_SOURCE_BYTES, env_limit};
use boli::parser;

use std::env;
use std::fs;
use std::io::{self, BufRead, Read, Write};
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
                    let _ = out_line(&format!("BOLI version {}", env!("CARGO_PKG_VERSION")));
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
    let _ = out_line("BOLI - Hinglish me programming language");
    let _ = out_line("");
    let _ = out_line("Use:");
    let _ = out_line("  cargo run              REPL kholo");
    let _ = out_line("  cargo run -- run x.hl  x.hl file chalao");
    let _ = out_line("  cargo run -- help      yeh message dikhao");
    let _ = out_line("  cargo run -- version   version dikhao");
    let _ = out_line("");
    let _ = out_line("Built-in kaams: lambayi, upar, neeche, text, sankhya");
    let _ = out_line("  lambayi <text|list>   text ke characters / list ke items");
    let _ = out_line("  upar <text>           UPPERCASE");
    let _ = out_line("  neeche <text>         lowercase");
    let _ = out_line("  text <anything>       kisi bhi value ko text me");
    let _ = out_line("  sankhya <text>        text ko number me (galat text par error)");
}

// File padhkar chalao
fn run_file(path: &str) {
    // source file size limit (env BOLI_MAX_SOURCE_BYTES, default 10 MiB)
    // metadata check pe sirf bharosa nahi - read bhi max+1 tak hi hota hai
    // (file read ke beech grow ho sakti hai; TOCTOU se bachne ke liye)
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
    let source = match read_capped(path, max_source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    match parser::parse_source(&source) {
        Ok(program) => {
            let mut env = Environment::new();
            if let Err(e) = env.run_program(&program) {
                show_boli_error(&source, &e);
                process::exit(1);
            }
        }
        Err(e) => {
            show_boli_error(&source, &e);
            process::exit(1);
        }
    }
}

// max_source bytes tak hi file padho; exceed ho to Galti error
// read ke dauraan file grow ho to bhi cap lagu rehta hai
fn read_capped(path: &str, max_source: u64) -> Result<String, String> {
    let file =
        fs::File::open(path).map_err(|e| format!("Galti: file '{}' nahi khuli: {}", path, e))?;
    let mut buf: Vec<u8> = Vec::new();
    file.take(max_source + 1)
        .read_to_end(&mut buf)
        .map_err(|e| format!("Galti: file '{}' padh nahi paye: {}", path, e))?;
    if buf.len() as u64 > max_source {
        return Err(format!(
            "Galti: file '{}' bahut badi hai ({} bytes se zyada, max {})",
            path,
            buf.len() - 1,
            max_source
        ));
    }
    String::from_utf8(buf).map_err(|_| format!("Galti: file '{}' UTF-8 nahi hai", path))
}

// Error dikhane ka helper: agar line+col pata hai to source line aur caret bhi print karta hai
fn show_boli_error(source: &str, e: &boli::ast::BoliError) {
    eprintln!("{}", e);
    if let (Some(line), Some(col)) = (e.line, e.col) {
        // EOF jaise errors line ke baahar point kar sakte hain - last line par clamp karo
        let (line_str, caret_col) = match source.lines().nth(line - 1) {
            Some(s) => (s.to_string(), col),
            None => match source.lines().last() {
                Some(s) => (s.to_string(), s.chars().count() + 1),
                None => (String::new(), 1),
            },
        };
        if !line_str.is_empty() {
            eprintln!("    {}", line_str);
            eprintln!("    {}^", " ".repeat(caret_col - 1));
        }
    }
}

// REPL - interactive shell
// Multi-line blocks (agar ... khatam) ko estimate karke jodta hai
// REPL input line ko max_bytes tak padhna - bahut badi line ko drain karke reject karta hai
enum LineRead {
    Eof,          // EOF (Ctrl-D)
    Line(String), // normal line (newline tak)
    TooLong,      // bahut badi line - poori consume + discard kar di
}

fn read_repl_line(stdin: &mut io::StdinLock, max_bytes: u64) -> io::Result<LineRead> {
    let mut buf: Vec<u8> = Vec::new();
    loop {
        let chunk = match stdin.fill_buf() {
            Ok(c) => c,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        if chunk.is_empty() {
            return Ok(if buf.is_empty() {
                LineRead::Eof
            } else {
                LineRead::Line(String::from_utf8_lossy(&buf).into_owned())
            });
        }
        match chunk.iter().position(|&b| b == b'\n') {
            Some(pos) => {
                if buf.len() as u64 + (pos + 1) as u64 > max_bytes {
                    stdin.consume(pos + 1); // poori line drain, reject karo
                    return Ok(LineRead::TooLong);
                }
                buf.extend_from_slice(&chunk[..=pos]); // trailing \n bhi rakhlo (read_line jaisa)
                stdin.consume(pos + 1);
                return Ok(LineRead::Line(String::from_utf8_lossy(&buf).into_owned()));
            }
            None => {
                if buf.len() as u64 + chunk.len() as u64 > max_bytes {
                    // poori line drain kar do taaki agli line clean ho
                    let consumed = chunk.len();
                    stdin.consume(consumed);
                    loop {
                        let c = stdin.fill_buf()?;
                        if c.is_empty() {
                            return Ok(LineRead::TooLong);
                        }
                        match c.iter().position(|&b| b == b'\n') {
                            Some(p) => {
                                stdin.consume(p + 1);
                                return Ok(LineRead::TooLong);
                            }
                            None => {
                                let len = c.len();
                                stdin.consume(len);
                            }
                        }
                    }
                }
                buf.extend_from_slice(chunk);
                let len = chunk.len();
                stdin.consume(len);
            }
        }
    }
}

fn repl() {
    if !out_line("BOLI REPL - likhne ke liye tayyar!") {
        return; // stdout band hai - kuch print karne ka point nahi
    }
    let _ = out_line("REPL se nikalne ke liye 'nikalo' likho.");

    let mut env = Environment::new();
    let mut buffer = String::new();
    let mut depth = 0i32; // kitne block khule hain (agar/jab tak/kaam/tak vs khatam)
    let stdin = io::stdin();

    loop {
        // Prompt: normal ya continuation
        let prompt_ok = if depth > 0 {
            out("...> ")
        } else {
            out("boli> ")
        };
        if !prompt_ok {
            return; // stdout band hai
        }

        let max_repl_line = env_limit(
            "BOLI_MAX_REPL_LINE_BYTES",
            boli::limits::DEFAULT_MAX_REPL_LINE_BYTES,
        );
        let mut stdin_lock = stdin.lock();
        let line = match read_repl_line(&mut stdin_lock, max_repl_line) {
            Ok(LineRead::Eof) => break, // EOF (Ctrl-D)
            Ok(LineRead::Line(l)) => l,
            Ok(LineRead::TooLong) => {
                eprintln!(
                    "Galti: line bahut badi hai (max {} bytes) - yeh line chhod di, agli try karo",
                    max_repl_line
                );
                continue;
            }
            Err(_) => {
                eprintln!("Galti: input nahi padh paye");
                break;
            }
        };
        drop(stdin_lock);

        let trimmed = line.trim();

        // Block ke andar blank line chhod do
        if trimmed.is_empty() && depth > 0 {
            continue;
        }
        // REPL se nikalna
        if depth == 0 && (trimmed == "nikalo" || trimmed == "exit" || trimmed == "quit") {
            let _ = out_line("BOLI band ho raha hai. Alvida!");
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
        let program_source = buffer.clone();
        match parser::parse_source(&program_source) {
            Ok(program) => {
                if let Err(e) = env.run_program(&program) {
                    show_boli_error(&program_source, &e);
                }
            }
            Err(e) => show_boli_error(&program_source, &e),
        }
        buffer.clear();
        depth = 0;
    }
}
