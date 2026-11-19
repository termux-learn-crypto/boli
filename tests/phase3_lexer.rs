use boli::ast::BoliError;
use boli::interpreter::{Environment, Value};
use boli::lexer::{Tok, lex_with_limits};
use boli::parser::parse_source;
use std::path::PathBuf;

struct Limits {
    source: u64,
    string: u64,
    tokens: u64,
}

fn wide_limits() -> Limits {
    Limits {
        source: 1_000_000,
        string: 1_000_000,
        tokens: 1_000_000,
    }
}

fn boli_literal(s: &str) -> String {
    let mut out = String::from("\"");
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn escaped(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn canonical_token(token: &boli::lexer::Token) -> String {
    let debug = format!("{:?}", token.tok);
    let kind = debug.split('(').next().unwrap_or(&debug).to_string();
    let value = match &token.tok {
        Tok::Text(text) => escaped(text),
        Tok::Number(number) => Value::Number(*number).dikhane_laayak(),
        Tok::Ident(word) => word.clone(),
        _ => String::new(),
    };
    format!("{}|{}|{}|{}", kind, value, token.line, token.col)
}

fn canonical_rust(source: &str, limits: &Limits) -> Result<Vec<String>, String> {
    match lex_with_limits(source, limits.source, limits.string, limits.tokens) {
        Ok(tokens) => Ok(tokens.iter().map(canonical_token).collect()),
        Err(error) => Err(format!(
            "{}:{}:{}",
            at(error.line),
            at(error.col),
            error.message
        )),
    }
}

fn at(value: Option<usize>) -> usize {
    value.unwrap_or(0)
}

fn run_boli_source(source: &str) -> Result<Vec<String>, String> {
    let program = parse_source(source).map_err(|error: BoliError| error.to_string())?;
    let mut env = Environment::new();
    env.set_source_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("selfhost/lexer"));
    env.collect_output();
    env.run_program(&program)
        .map(|_| env.take_output())
        .map_err(|error: BoliError| error.to_string())
}

fn canonical_boli(source: &str, limits: &Limits) -> Result<Vec<String>, String> {
    let runner = format!(
        "laao \"errors.boli\"\n\
         laao \"token.boli\"\n\
         laao \"reader.boli\"\n\
         laao \"lexer.boli\"\n\
         source rakho {}\n\
         max_source rakho {}\n\
         max_string rakho {}\n\
         max_tokens rakho {}\n\
         res rakho lexer_lex source max_source max_string max_tokens\n\
         ok rakho res hissa \"ok\"\n\
         agar ok sahi barabar hai to\n\
             likho \"OK\"\n\
             tokens rakho res hissa \"tokens\"\n\
             n rakho lambayi tokens\n\
             i rakho 1\n\
             jab tak i n se chhota ya barabar hai to\n\
                 item rakho tokens hissa i\n\
                 line rakho lexer_token_text item\n\
                 likho line\n\
                 jodo i i aur 1\n\
             khatam\n\
         nahi to\n\
             likho \"ERR\"\n\
             err rakho res hissa \"error\"\n\
             kind rakho err hissa \"kind\"\n\
             msg rakho err hissa \"message\"\n\
             ln rakho err hissa \"line\"\n\
             cl rakho err hissa \"col\"\n\
             likho kind\n\
             likho msg\n\
             likho text ln\n\
             likho text cl\n\
         khatam\n",
        boli_literal(source),
        limits.source,
        limits.string,
        limits.tokens
    );
    let mut lines = run_boli_source(&runner)
        .unwrap_or_else(|error| panic!("BOLI runner failed: {}", error))
        .into_iter();
    match lines.next().as_deref() {
        Some("OK") => Ok(lines.collect()),
        Some("ERR") => {
            let _kind = lines.next().unwrap_or_default();
            let message = lines.next().unwrap_or_default();
            let line = lines.next().unwrap_or_default();
            let col = lines.next().unwrap_or_default();
            Err(format!("{}:{}:{}", line, col, message))
        }
        other => panic!("unknown BOLI lexer marker: {:?}", other),
    }
}

fn assert_match(source: &str, limits: &Limits) {
    let rust = canonical_rust(source, limits);
    let boli = canonical_boli(source, limits);
    assert_eq!(rust, boli, "lexer mismatch for source {:?}", source);
}

#[test]
fn phase3_keywords_numbers_and_identifiers() {
    let limits = wide_limits();
    assert_match(
        "likho rakho 42 -7 3.14 abc_123 LIKHO NaMaStE break continue\n",
        &limits,
    );
    assert_match(
        "pucho jodo ghatao guna bhaag agar nahi to khatam se aur ya ko tak jab bada chhota barabar hai kaam wapas do sahi galat khaali hissa map laao ruk agla",
        &limits,
    );
}

#[test]
fn phase3_strings_comments_and_unicode() {
    let limits = wide_limits();
    assert_match(
        r#"likho "a\"b\\c\n" # note
"नमस्ते" "x\ry\tz""#,
        &limits,
    );
    assert_match("\"\" \"abc\"", &limits);
}

#[test]
fn phase3_whitespace_and_columns() {
    let limits = wide_limits();
    assert_match("likho\t5\r\nabc  \n\nxyz", &limits);
}

#[test]
fn phase3_lexer_error_cases() {
    let limits = wide_limits();
    assert_match("likho \"abc", &limits);
    assert_match("likho 5-3", &limits);
    assert_match("likho ;", &limits);
    assert_match(&format!("likho {}", "9".repeat(400)), &limits);
}

#[test]
fn phase3_limit_error_cases() {
    let short = Limits {
        source: 6,
        string: 1_000_000,
        tokens: 1_000_000,
    };
    assert_match("likho 5", &short);

    let small_string = Limits {
        source: 1_000_000,
        string: 2,
        tokens: 1_000_000,
    };
    assert_match("\"abc\"", &small_string);
    assert_match("abcdef", &small_string);

    let few_tokens = Limits {
        source: 1_000_000,
        string: 1_000_000,
        tokens: 2,
    };
    assert_match("likho 5", &few_tokens);
}
