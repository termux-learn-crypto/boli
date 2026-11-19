// BOLI ka Lexer - code ko tokens (toshne) me todta hai
// BOLI me koi symbol nahi hota, isliye lexer sirf shabd, number, text, newline aur # comment samajhta hai

use crate::ast::BoliError;

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
    cur: Option<char>,
    nxt: Option<char>,
    col: usize, // current char ka 1-based column; newline par reset
}

impl<'a> Cursor<'a> {
    fn new(source: &'a str) -> Self {
        let mut iter = source.char_indices();
        let cur = iter.next().map(|(_, c)| c);
        let nxt = iter.next().map(|(_, c)| c);
        Cursor {
            iter,
            cur,
            nxt,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.cur
    }

    fn peek_next(&self) -> Option<char> {
        self.nxt
    }

    fn peek_col(&self) -> usize {
        self.col
    }

    fn bump(&mut self) -> Option<char> {
        let old = self.cur;
        self.cur = self.nxt;
        self.nxt = self.iter.next().map(|(_, c)| c);
        match old {
            Some('\n') => self.col = 1,
            Some(_) => self.col += 1,
            None => {}
        }
        old
    }
}

fn push_token(
    tokens: &mut Vec<Token>,
    tok: Tok,
    line: usize,
    col: usize,
    max_tokens: u64,
) -> Result<(), BoliError> {
    if tokens.len() as u64 >= max_tokens {
        return Err(BoliError::line_col(
            line,
            col,
            format!("code me bahut zyada tokens hain (max {})", max_tokens),
        ));
    }
    tokens.push(Token { tok, line, col });
    Ok(())
}

/// Lex with explicit limits (used by tests and by `lex` which reads env).
pub fn lex_with_limits(
    source: &str,
    max_source_bytes: u64,
    max_string_bytes: u64,
    max_tokens: u64,
) -> Result<Vec<Token>, BoliError> {
    // Source size limit pehle - badi files ko tokenize karne se pehle hi rok do
    if source.len() as u64 > max_source_bytes {
        return Err(BoliError::line_col(
            1,
            1,
            format!(
                "source bahut bada hai ({} bytes, max {})",
                source.len(),
                max_source_bytes
            ),
        ));
    }

    let mut cursor = Cursor::new(source);
    let mut tokens: Vec<Token> = Vec::new();
    let mut line = 1usize;
    let mut separated = true;

    while let Some(c) = cursor.peek() {
        // Spaces aur tabs - inhe chhod do
        if c == ' ' || c == '\t' || c == '\r' {
            cursor.bump();
            separated = true;
            continue;
        }

        // Nai line
        if c == '\n' {
            let tok_col = cursor.peek_col();
            push_token(&mut tokens, Tok::Newline, line, tok_col, max_tokens)?;
            line += 1;
            cursor.bump();
            separated = true;
            continue;
        }

        // # se lekar line ke ant tak comment hai, usse chhod do
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

        // Text (string) - double quotes ke beech ka hissa
        if c == '"' {
            let start_line = line;
            let tok_col = cursor.peek_col();
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
                        return Err(BoliError::line_col(
                            start_line,
                            cursor.peek_col(),
                            "text pura nahi hua, band karne ke liye \" lagao",
                        ));
                    }
                    Some('\\') => {
                        // Simple escape: backslash ke baad quote/backslash/n aya to asli character dalo
                        match cursor.peek_next() {
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
                        }
                    }
                    Some(ch) => {
                        text.push(ch);
                        cursor.bump();
                    }
                }
                if text.len() as u64 > max_string_bytes {
                    return Err(BoliError::line_col(
                        start_line,
                        cursor.peek_col(),
                        format!("text bahut bada hai (max {} bytes)", max_string_bytes),
                    ));
                }
            }
            if !band_hua {
                return Err(BoliError::line_col(
                    start_line,
                    cursor.peek_col(),
                    "text pura nahi hua, band karne ke liye \" lagao",
                ));
            }
            push_token(
                &mut tokens,
                Tok::Text(text),
                start_line,
                tok_col,
                max_tokens,
            )?;
            separated = false;
            continue;
        }

        // Number - ankh laage hue, ek dot ho sakti hai (jaise 99.50 ya -7)
        // '-' number ka hissa tabhi jab digit ho aur koi atom usse chipka na ho
        if c.is_ascii_digit()
            || (c == '-' && cursor.peek_next().is_some_and(|n| n.is_ascii_digit()))
        {
            let start_line = line;
            let tok_col = cursor.peek_col();
            if c == '-' && !separated {
                return Err(BoliError::line_col(
                    start_line,
                    tok_col,
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
            // Dot sirf tab lo jab uske baad digit ho (warna oh alag galti hai)
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
                BoliError::line_col(
                    start_line,
                    cursor.peek_col(),
                    format!("'{}' number sahi nahi hai", num),
                )
            })?;
            // BOLI me sirf finite numbers chalte hain (inf/nan allowed nahi)
            if !value.is_finite() {
                return Err(BoliError::line_col(
                    start_line,
                    cursor.peek_col(),
                    format!("'{}' bahut bada hai, finite number nahi hai", num),
                ));
            }
            push_token(
                &mut tokens,
                Tok::Number(value),
                start_line,
                tok_col,
                max_tokens,
            )?;
            separated = false;
            continue;
        }

        // Shabd - letters (Devanagari bhi chal sakti hai), digits aur underscore
        if c.is_alphabetic() || c == '_' || is_mark(c) {
            let start_line = line;
            let tok_col = cursor.peek_col();
            let mut word = String::new();
            while let Some(x) = cursor.peek() {
                if !is_word_char(x) {
                    break;
                }
                word.push(x);
                cursor.bump();
                if word.len() as u64 > max_string_bytes {
                    return Err(BoliError::line_col(
                        start_line,
                        cursor.peek_col(),
                        format!("shabd bahut bada hai (max {} bytes)", max_string_bytes),
                    ));
                }
            }
            let lower = word.to_lowercase();
            match keyword(&lower) {
                Some(kw) => push_token(&mut tokens, kw, start_line, tok_col, max_tokens)?,
                None => push_token(
                    &mut tokens,
                    Tok::Ident(word),
                    start_line,
                    tok_col,
                    max_tokens,
                )?,
            }
            separated = false;
            continue;
        }

        // Koi symbol (jaise ; { } ( ) = : .) BOLI me allowed nahi
        return Err(BoliError::line_col(
            line,
            cursor.peek_col(),
            format!(
                "'{}' nahi chahiye, BOLI me sirf shabd, number aur \"text\" use hota hai",
                c
            ),
        ));
    }

    push_token(&mut tokens, Tok::Eof, line, cursor.peek_col(), max_tokens)?;
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
