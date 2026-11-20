// BOLI ka Lexer - code ko tokens (toshne) me todta hai
// BOLI me koi symbol nahi hota, isliye lexer sirf shabd, number, text, newline aur # comment samajhta hai

use crate::ast::{BoliError, Span};

// Token ke type
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    // Keywords (rakhe hue shabd)
    Likho,
    Rakho,
    Pucho,
    Jodo,
    Ghatao,
    Guna,
    Bhaag,
    Agar,
    Nahi,
    To,
    Khatam,
    Se,
    Aur,
    Ya,
    Ko,
    Tak,
    Jab,
    Bada,
    Chhota,
    Barabar,
    Hai,
    Kaam,
    Wapas,
    Do,
    Sahi,
    Galat,
    Khaali,
    Hissa, // list ka index ya map ki chaabi
    Map,   // map literal banane ka shabd
    Laao,  // module import/include: doosri .hl file laana
    Ruk,   // break
    Agla,  // continue
    // Numbers (jaise 28 ya 99.50)
    Number(f64),
    // Shuru aur band hona wali quotes ke beech ka text
    Text(String),
    // Variable ya kaam ka naam
    Ident(String),
    // Line ka khatam hona (BOLI me line hi statement ka ant hai)
    Newline,
    // File ka khatam hona
    Eof,
}

// Line number ke saath token
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
    pub col: usize, // 1-based char column (Unicode-honest)
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexErrorKind {
    SourceTooLarge,
    TokenLimit,
    TextTooLarge,
    WordTooLarge,
    UnterminatedText,
    NumberNotFinite,
    UnexpectedCharacter,
    StuckMinus,
}

impl LexErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            LexErrorKind::SourceTooLarge => "source_too_large",
            LexErrorKind::TokenLimit => "token_limit",
            LexErrorKind::TextTooLarge => "text_too_large",
            LexErrorKind::WordTooLarge => "word_too_large",
            LexErrorKind::UnterminatedText => "unterminated_text",
            LexErrorKind::NumberNotFinite => "number_not_finite",
            LexErrorKind::UnexpectedCharacter => "unexpected_character",
            LexErrorKind::StuckMinus => "stuck_minus",
        }
    }
}

impl Token {
    // Galti message ke liye token ko dikhane layak banao
    pub fn describe(&self) -> String {
        match &self.tok {
            Tok::Likho => "likho".to_string(),
            Tok::Rakho => "rakho".to_string(),
            Tok::Pucho => "pucho".to_string(),
            Tok::Jodo => "jodo".to_string(),
            Tok::Ghatao => "ghatao".to_string(),
            Tok::Guna => "guna".to_string(),
            Tok::Bhaag => "bhaag".to_string(),
            Tok::Agar => "agar".to_string(),
            Tok::Nahi => "nahi".to_string(),
            Tok::To => "to".to_string(),
            Tok::Khatam => "khatam".to_string(),
            Tok::Se => "se".to_string(),
            Tok::Aur => "aur".to_string(),
            Tok::Ya => "ya".to_string(),
            Tok::Ko => "ko".to_string(),
            Tok::Tak => "tak".to_string(),
            Tok::Jab => "jab".to_string(),
            Tok::Bada => "bada".to_string(),
            Tok::Chhota => "chhota".to_string(),
            Tok::Barabar => "barabar".to_string(),
            Tok::Hai => "hai".to_string(),
            Tok::Kaam => "kaam".to_string(),
            Tok::Wapas => "wapas".to_string(),
            Tok::Do => "do".to_string(),
            Tok::Sahi => "sahi".to_string(),
            Tok::Galat => "galat".to_string(),
            Tok::Khaali => "khaali".to_string(),
            Tok::Hissa => "hissa".to_string(),
            Tok::Map => "map".to_string(),
            Tok::Laao => "laao".to_string(),
            Tok::Ruk => "ruk".to_string(),
            Tok::Agla => "agla".to_string(),
            Tok::Number(n) => format!("{}", n),
            Tok::Text(s) => format!("\"{}\"", s),
            Tok::Ident(s) => s.clone(),
            Tok::Newline => "line ka ant".to_string(),
            Tok::Eof => "file ka ant".to_string(),
        }
    }
}

// Unicode combining marks (Devanagari signs, virama etc.) ko word characters mano
fn is_mark(c: char) -> bool {
    let cp = c as u32;
    (0x0300..=0x036F).contains(&cp)   // combining diacriticals
        || (0x0900..=0x097F).contains(&cp)  // Devanagari block
        || (0x0980..=0x09FF).contains(&cp)  // Bengali
        || (0x0A00..=0x0A7F).contains(&cp)  // Gurmukhi
        || (0x0B00..=0x0B7F).contains(&cp)  // Oriya
        || (0x0C00..=0x0C7F).contains(&cp)  // Telugu
        || (0x0D00..=0x0D7F).contains(&cp)  // Malayalam
        || (0x0E00..=0x0E7F).contains(&cp)  // Thai
        || (0x1AB0..=0x1AFF).contains(&cp)
        || (0x1DC0..=0x1DFF).contains(&cp)
        || (0xFE20..=0xFE2F).contains(&cp)
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || is_mark(c)
}

// Keyword matching: chhote aksharon me compare karo
fn keyword(word: &str) -> Option<Tok> {
    match word {
        "likho" => Some(Tok::Likho),
        "rakho" => Some(Tok::Rakho),
        "pucho" => Some(Tok::Pucho),
        "jodo" => Some(Tok::Jodo),
        "ghatao" => Some(Tok::Ghatao),
        "guna" => Some(Tok::Guna),
        "bhaag" => Some(Tok::Bhaag),
        "agar" => Some(Tok::Agar),
        "nahi" => Some(Tok::Nahi),
        "to" => Some(Tok::To),
        "khatam" => Some(Tok::Khatam),
        "se" => Some(Tok::Se),
        "aur" => Some(Tok::Aur),
        "ya" => Some(Tok::Ya),
        "ko" => Some(Tok::Ko),
        "tak" => Some(Tok::Tak),
        "jab" => Some(Tok::Jab),
        "bada" => Some(Tok::Bada),
        "chhota" => Some(Tok::Chhota),
        "barabar" => Some(Tok::Barabar),
        "hai" => Some(Tok::Hai),
        "kaam" => Some(Tok::Kaam),
        "wapas" => Some(Tok::Wapas),
        "do" => Some(Tok::Do),
        "sahi" => Some(Tok::Sahi),
        "galat" => Some(Tok::Galat),
        "khaali" => Some(Tok::Khaali),
        "hissa" => Some(Tok::Hissa),
        "map" => Some(Tok::Map),
        "laao" => Some(Tok::Laao),
        "ruk" => Some(Tok::Ruk),
        "agla" => Some(Tok::Agla),
        "break" => Some(Tok::Ruk), // English alias
        "continue" => Some(Tok::Agla),
        _ => None,
    }
}

struct Cursor<'a> {
    iter: std::str::CharIndices<'a>,
    cur: Option<(usize, char)>,
    nxt: Option<(usize, char)>,
    len: usize,
    line: usize,
    col: usize,
    start_byte: usize,
    start_line: usize,
    start_col: usize,
}

impl<'a> Cursor<'a> {
    fn new(source: &'a str) -> Self {
        let mut iter = source.char_indices();
        let cur = iter.next();
        let nxt = iter.next();
        Cursor {
            iter,
            cur,
            nxt,
            len: source.len(),
            line: 1,
            col: 1,
            start_byte: 0,
            start_line: 1,
            start_col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.cur.map(|(_, c)| c)
    }

    fn peek_next(&self) -> Option<char> {
        self.nxt.map(|(_, c)| c)
    }

    fn peek_col(&self) -> usize {
        self.col
    }

    fn byte_pos(&self) -> usize {
        self.cur.map(|(i, _)| i).unwrap_or(self.len)
    }

    fn mark(&mut self) {
        self.start_byte = self.byte_pos();
        self.start_line = self.line;
        self.start_col = self.col;
    }

    fn span(&self) -> Span {
        self.span_to(self.byte_pos(), self.line, self.col)
    }

    fn span_to(&self, end_byte: usize, end_line: usize, end_col: usize) -> Span {
        Span {
            start: self.start_byte,
            end: end_byte,
            start_line: self.start_line,
            start_col: self.start_col,
            end_line,
            end_col,
        }
    }

    fn bump(&mut self) -> Option<char> {
        let old = self.cur;
        self.cur = self.nxt;
        self.nxt = self.iter.next();
        match old {
            Some((_, '\n')) => {
                self.line += 1;
                self.col = 1;
            }
            Some(_) => self.col += 1,
            None => {}
        }
        old.map(|(_, c)| c)
    }
}

fn lex_error(
    kind: LexErrorKind,
    line: usize,
    col: usize,
    span: Span,
    message: impl Into<String>,
) -> BoliError {
    BoliError::line_col_kind_span(line, col, kind.as_str(), span, message)
}

fn push_token(
    tokens: &mut Vec<Token>,
    tok: Tok,
    span: Span,
    max_tokens: u64,
) -> Result<(), BoliError> {
    if tokens.len() as u64 >= max_tokens {
        return Err(lex_error(
            LexErrorKind::TokenLimit,
            span.start_line,
            span.start_col,
            span,
            format!("code me bahut zyada tokens hain (max {})", max_tokens),
        ));
    }
    tokens.push(Token {
        tok,
        line: span.start_line,
        col: span.start_col,
        span,
    });
    Ok(())
}

/// Lex with explicit limits (used by tests and by `lex` which reads env).
pub fn lex_with_limits(
    source: &str,
    max_source_bytes: u64,
    max_string_bytes: u64,
    max_tokens: u64,
) -> Result<Vec<Token>, BoliError> {
    if source.len() as u64 > max_source_bytes {
        let span = Span {
            start: 0,
            end: 0,
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 1,
        };
        return Err(lex_error(
            LexErrorKind::SourceTooLarge,
            1,
            1,
            span,
            format!(
                "source bahut bada hai ({} bytes, max {})",
                source.len(),
                max_source_bytes
            ),
        ));
    }

    let mut cursor = Cursor::new(source);
    let mut tokens: Vec<Token> = Vec::new();
    let mut separated = true;

    while let Some(c) = cursor.peek() {
        if c == ' ' || c == '\t' || c == '\r' {
            cursor.bump();
            separated = true;
            continue;
        }

        if c == '\n' {
            cursor.mark();
            let span = cursor.span_to(cursor.byte_pos() + 1, cursor.line, cursor.col + 1);
            push_token(&mut tokens, Tok::Newline, span, max_tokens)?;
            cursor.bump();
            separated = true;
            continue;
        }

        if c == '#' {
            while let Some(x) = cursor.peek() {
                if x == '\n' {
                    break;
                }
                cursor.bump();
            }
            separated = true;
            continue;
        }

        if c == '"' {
            cursor.mark();
            let start_line = cursor.start_line;
            let mut text = String::new();
            cursor.bump();
            let mut band_hua = false;
            loop {
                match cursor.peek() {
                    None => break,
                    Some('"') => {
                        band_hua = true;
                        cursor.bump();
                        break;
                    }
                    Some('\n') => {
                        return Err(lex_error(
                            LexErrorKind::UnterminatedText,
                            start_line,
                            cursor.peek_col(),
                            cursor.span(),
                            "text pura nahi hua, band karne ke liye \" lagao",
                        ));
                    }
                    Some('\\') => match cursor.peek_next() {
                        Some(n @ ('"' | '\\')) => {
                            text.push(n);
                            cursor.bump();
                            cursor.bump();
                        }
                        Some('n') => {
                            text.push('\n');
                            cursor.bump();
                            cursor.bump();
                        }
                        _ => {
                            text.push('\\');
                            cursor.bump();
                        }
                    },
                    Some(ch) => {
                        text.push(ch);
                        cursor.bump();
                    }
                }
                if text.len() as u64 > max_string_bytes {
                    return Err(lex_error(
                        LexErrorKind::TextTooLarge,
                        start_line,
                        cursor.peek_col(),
                        cursor.span(),
                        format!("text bahut bada hai (max {} bytes)", max_string_bytes),
                    ));
                }
            }
            if !band_hua {
                return Err(lex_error(
                    LexErrorKind::UnterminatedText,
                    start_line,
                    cursor.peek_col(),
                    cursor.span(),
                    "text pura nahi hua, band karne ke liye \" lagao",
                ));
            }
            push_token(&mut tokens, Tok::Text(text), cursor.span(), max_tokens)?;
            separated = false;
            continue;
        }

        if c.is_ascii_digit()
            || (c == '-' && cursor.peek_next().is_some_and(|n| n.is_ascii_digit()))
        {
            cursor.mark();
            let start_line = cursor.start_line;
            let start_col = cursor.start_col;
            if c == '-' && !separated {
                let span = cursor.span_to(cursor.start_byte + 1, start_line, start_col + 1);
                return Err(lex_error(
                    LexErrorKind::StuckMinus,
                    start_line,
                    start_col,
                    span,
                    "'-' number ke saath chipka hai, space do ya 'ghatao' likho",
                ));
            }
            let mut num = String::new();
            if c == '-' {
                num.push('-');
                cursor.bump();
            }
            while let Some(x) = cursor.peek() {
                if x.is_ascii_digit() {
                    num.push(x);
                    cursor.bump();
                } else {
                    break;
                }
            }
            if cursor.peek() == Some('.') && cursor.peek_next().is_some_and(|n| n.is_ascii_digit())
            {
                num.push('.');
                cursor.bump();
                while let Some(x) = cursor.peek() {
                    if x.is_ascii_digit() {
                        num.push(x);
                        cursor.bump();
                    } else {
                        break;
                    }
                }
            }
            let value: f64 = num.parse().map_err(|_| {
                lex_error(
                    LexErrorKind::UnexpectedCharacter,
                    start_line,
                    cursor.peek_col(),
                    cursor.span(),
                    format!("'{}' number sahi nahi hai", num),
                )
            })?;
            if !value.is_finite() {
                return Err(lex_error(
                    LexErrorKind::NumberNotFinite,
                    start_line,
                    cursor.peek_col(),
                    cursor.span(),
                    format!("'{}' bahut bada hai, finite number nahi hai", num),
                ));
            }
            push_token(&mut tokens, Tok::Number(value), cursor.span(), max_tokens)?;
            separated = false;
            continue;
        }

        if c.is_alphabetic() || c == '_' || is_mark(c) {
            cursor.mark();
            let start_line = cursor.start_line;
            let mut word = String::new();
            while let Some(x) = cursor.peek() {
                if !is_word_char(x) {
                    break;
                }
                word.push(x);
                cursor.bump();
                if word.len() as u64 > max_string_bytes {
                    return Err(lex_error(
                        LexErrorKind::WordTooLarge,
                        start_line,
                        cursor.peek_col(),
                        cursor.span(),
                        format!("shabd bahut bada hai (max {} bytes)", max_string_bytes),
                    ));
                }
            }
            let lower = word.to_lowercase();
            let token = match keyword(&lower) {
                Some(kw) => kw,
                None => Tok::Ident(word),
            };
            push_token(&mut tokens, token, cursor.span(), max_tokens)?;
            separated = false;
            continue;
        }

        let start_byte = cursor.byte_pos();
        let span = Span {
            start: start_byte,
            end: start_byte + c.len_utf8(),
            start_line: cursor.line,
            start_col: cursor.col,
            end_line: cursor.line,
            end_col: cursor.col + 1,
        };
        return Err(lex_error(
            LexErrorKind::UnexpectedCharacter,
            cursor.line,
            cursor.col,
            span,
            format!(
                "'{}' nahi chahiye, BOLI me sirf shabd, number aur \"text\" use hota hai",
                c
            ),
        ));
    }

    cursor.mark();
    push_token(&mut tokens, Tok::Eof, cursor.span(), max_tokens)?;
    Ok(tokens)
}

// Pura source code tokens me convert karo (env limits: BOLI_MAX_SOURCE_BYTES, BOLI_MAX_STRING_BYTES, BOLI_MAX_TOKENS)
pub fn lex(source: &str) -> Result<Vec<Token>, BoliError> {
    let max_source = crate::limits::env_limit(
        "BOLI_MAX_SOURCE_BYTES",
        crate::limits::DEFAULT_MAX_SOURCE_BYTES,
    );
    let max_string = crate::limits::env_limit(
        "BOLI_MAX_STRING_BYTES",
        crate::limits::DEFAULT_MAX_STRING_BYTES,
    );
    let max_tokens = crate::limits::env_limit("BOLI_MAX_TOKENS", crate::limits::DEFAULT_MAX_TOKENS);
    lex_with_limits(source, max_source, max_string, max_tokens)
}
