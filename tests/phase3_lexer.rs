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
    let span = token.span;
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        kind,
        value,
        token.line,
        token.col,
        span.start,
        span.end,
        span.start_line,
        span.start_col,
        span.end_line,
        span.end_col
    )
}

fn canonical_rust(source: &str, limits: &Limits) -> Result<Vec<String>, String> {
    match lex_with_limits(source, limits.source, limits.string, limits.tokens) {
        Ok(tokens) => Ok(tokens.iter().map(canonical_token).collect()),
        Err(error) => {
            let span = error.span.as_deref().copied().unwrap_or_default();
            Err(format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                error.kind.unwrap_or(""),
                error.message,
                at(error.line),
                at(error.col),
                span.start,
                span.end,
                span.start_line,
                span.start_col,
                span.end_line,
                span.end_col
            ))
        }
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
         laao \"unicode.boli\"\n\
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
             likho err hissa \"kind\"\n\
             likho err hissa \"message\"\n\
             likho text err hissa \"line\"\n\
             likho text err hissa \"col\"\n\
             likho text err hissa \"start_byte\"\n\
             likho text err hissa \"end_byte\"\n\
             likho text err hissa \"start_line\"\n\
             likho text err hissa \"start_col\"\n\
             likho text err hissa \"end_line\"\n\
             likho text err hissa \"end_col\"\n\
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
            let kind = lines.next().unwrap_or_default();
            let message = lines.next().unwrap_or_default();
            let line = lines.next().unwrap_or_default();
            let col = lines.next().unwrap_or_default();
            let start_byte = lines.next().unwrap_or_default();
            let end_byte = lines.next().unwrap_or_default();
            let start_line = lines.next().unwrap_or_default();
            let start_col = lines.next().unwrap_or_default();
            let end_line = lines.next().unwrap_or_default();
            let end_col = lines.next().unwrap_or_default();
            Err(format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                kind,
                message,
                line,
                col,
                start_byte,
                end_byte,
                start_line,
                start_col,
                end_line,
                end_col
            ))
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

const LAST_FINITE: &str = "179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497791";
const FIRST_INFINITE: &str = "179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497792";

#[test]
fn phase3_negative_number_boundaries() {
    let limits = wide_limits();
    assert_match("-", &limits);
    assert_match("-abc", &limits);
    assert_match("--5", &limits);
    assert_match("-.5", &limits);
    assert_match("5 - 3", &limits);
    assert_match("5--3", &limits);
    assert_match("5-3", &limits);
    assert_match(" likho -5", &limits);
}

#[test]
fn phase3_numeric_finite_boundaries() {
    let limits = wide_limits();
    assert_match("9".repeat(308).as_str(), &limits);
    assert_match(LAST_FINITE, &limits);
    assert_match(FIRST_INFINITE, &limits);
    assert_match(&format!("{}.9", LAST_FINITE), &limits);
    assert_match(&format!("{}.0", FIRST_INFINITE), &limits);
    assert_match("0000000000000000000000000000000000000000001", &limits);
}

#[test]
fn phase3_unicode_byte_limits() {
    let small_source = Limits {
        source: 10,
        string: 1_000_000,
        tokens: 1_000_000,
    };
    assert_match("नमस्ते", &small_source);

    let small_string = Limits {
        source: 1_000_000,
        string: 10,
        tokens: 1_000_000,
    };
    assert_match("\"नमस्ते\"", &small_string);
    assert_match("नमस्ते", &small_string);

    let wide = wide_limits();
    assert_match("नमस्ते क_1", &wide);
}

#[test]
fn phase3_unicode_identifier_policy() {
    let limits = wide_limits();
    assert_match("α βγ Привет مرحبا नमस्ते क_1", &limits);
    assert_match("a\u{0301} क\u{0301}", &limits);
    assert_match("x१२३ ९abc α1", &limits);
    assert_match("likho α1", &limits);
    assert_match("likho 🙂", &limits);
}

fn deterministic_cases(seed: u64) -> Vec<String> {
    let pieces = [
        "likho ", "rakho ", " ", "\n", "5", "-3", "3.14", "abc", "α", "नम", "१", "\u{0301}",
        "\"hi\"", "#x\n", ";",
    ];
    let mut state = seed;
    let mut cases = Vec::new();
    for _ in 0..24 {
        let mut source = String::new();
        for _ in 0..6 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let index = ((state >> 33) as usize) % pieces.len();
            source.push_str(pieces[index]);
        }
        cases.push(source);
    }
    cases
}

#[test]
fn phase3_deterministic_generated_lexer_cases() {
    let limits = wide_limits();
    for source in deterministic_cases(0xB01_2026) {
        assert_match(&source, &limits);
    }
}

#[test]
fn phase3_boli_error_kinds() {
    let limits = wide_limits();
    assert_eq!(boli_error_kind("likho \"abc", &limits), "unterminated_text");
    assert_eq!(boli_error_kind("likho 5-3", &limits), "stuck_minus");
    assert_eq!(boli_error_kind("likho ;", &limits), "unexpected_character");
    assert_eq!(
        boli_error_kind(&format!("likho {}", "9".repeat(400)), &limits),
        "number_not_finite"
    );

    let short = Limits {
        source: 6,
        string: 1_000_000,
        tokens: 1_000_000,
    };
    assert_eq!(boli_error_kind("likho 5", &short), "source_too_large");

    let small_string = Limits {
        source: 1_000_000,
        string: 2,
        tokens: 1_000_000,
    };
    assert_eq!(boli_error_kind("\"abc\"", &small_string), "text_too_large");
    assert_eq!(boli_error_kind("abcdef", &small_string), "word_too_large");

    let few_tokens = Limits {
        source: 1_000_000,
        string: 1_000_000,
        tokens: 2,
    };
    assert_eq!(boli_error_kind("likho 5", &few_tokens), "token_limit");
}

fn boli_error_kind(source: &str, limits: &Limits) -> String {
    let runner = format!(
        "laao \"errors.boli\"\n\
         laao \"token.boli\"\n\
         laao \"reader.boli\"\n\
         laao \"unicode.boli\"\n\
         laao \"lexer.boli\"\n\
         source rakho {}\n\
         max_source rakho {}\n\
         max_string rakho {}\n\
         max_tokens rakho {}\n\
         res rakho lexer_lex source max_source max_string max_tokens\n\
         ok rakho res hissa \"ok\"\n\
         agar ok galat barabar hai to\n\
             err rakho res hissa \"error\"\n\
             likho err hissa \"kind\"\n\
         khatam\n",
        boli_literal(source),
        limits.source,
        limits.string,
        limits.tokens
    );
    run_boli_source(&runner)
        .unwrap_or_else(|error| panic!("BOLI runner failed: {}", error))
        .join("")
}
