// BOLI ka Lexer - code ko tokens (toshne) me todta hai
// BOLI me koi symbol nahi hota, isliye lexer sirf shabd, number, text, newline aur # comment samajhta hai

use crate::ast::BoliError;

// Token ke type
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    // Keywords (rakhe hue shabd)
    Likho, Rakho, Pucho,
    Jodo, Ghatao, Guna, Bhaag,
    Agar, Nahi, To, Khatam,
    Se, Aur, Ya, Ko, Tak, Jab,
    Bada, Chhota, Barabar, Hai,
    Kaam, Wapas, Do,
    Sahi, Galat, Khaali,
    Hissa,      // list ka index (1 se shuru)
    Ruk,        // break
    Agla,       // continue
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
        "ruk" => Some(Tok::Ruk),
        "agla" => Some(Tok::Agla),
        "break" => Some(Tok::Ruk),    // English alias
        "continue" => Some(Tok::Agla),
        _ => None,
    }
}

// Pura source code tokens me convert karo
pub fn lex(source: &str) -> Result<Vec<Token>, BoliError> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens: Vec<Token> = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;

    while i < chars.len() {
        let c = chars[i];

        // Spaces aur tabs - inhe chhod do
        if c == ' ' || c == '\t' || c == '\r' {
            i += 1;
            continue;
        }

        // Nai line
        if c == '\n' {
            tokens.push(Token { tok: Tok::Newline, line });
            line += 1;
            i += 1;
            continue;
        }

        // # se lekar line ke ant tak comment hai, usse chhod do
        if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Text (string) - double quotes ke beech ka hissa
        if c == '"' {
            let start_line = line;
            let mut text = String::new();
            i += 1;
            let mut band_hua = false;
            while i < chars.len() {
                let ch = chars[i];
                if ch == '"' {
                    band_hua = true;
                    i += 1;
                    break;
                }
                if ch == '\n' {
                    return Err(BoliError::line(
                        start_line,
                        "text pura nahi hua, band karne ke liye \" lagao",
                    ));
                }
                // Simple escape: backslash ke baad quote/backslash/n aya to asli character dalo
                if ch == '\\' && i + 1 < chars.len() {
                    let next = chars[i + 1];
                    if next == '"' || next == '\\' {
                        text.push(next);
                        i += 2;
                        continue;
                    }
                    if next == 'n' {
                        text.push('\n');
                        i += 2;
                        continue;
                    }
                }
                text.push(ch);
                i += 1;
            }
            if !band_hua {
                return Err(BoliError::line(
                    start_line,
                    "text pura nahi hua, band karne ke liye \" lagao",
                ));
            }
            tokens.push(Token { tok: Tok::Text(text), line: start_line });
            continue;
        }

        // Number - ankh laage hue, ek dot ho sakti hai (jaise 99.50 ya -7)
        // '-' sirf tab number ka hissa jab uske baad digit ho (BOLI me minus symbol allowed nahi)
        if c.is_ascii_digit() || (c == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let start_line = line;
            let mut num = String::new();
            if c == '-' {
                num.push('-');
                i += 1;
            }
            while i < chars.len() && chars[i].is_ascii_digit() {
                num.push(chars[i]);
                i += 1;
            }
            // Dot sirf tab lo jab uske baad digit ho (warna oh alag galti hai)
            if i < chars.len() && chars[i] == '.'
                && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()
            {
                num.push('.');
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    num.push(chars[i]);
                    i += 1;
                }
            }
            let value: f64 = num.parse().map_err(|_| {
                BoliError::line(start_line, format!("'{}' number sahi nahi hai", num))
            })?;
            // BOLI me sirf finite numbers chalte hain (inf/nan allowed nahi)
            if !value.is_finite() {
                return Err(BoliError::line(
                    start_line,
                    format!("'{}' bahut bada hai, finite number nahi hai", num),
                ));
            }
            tokens.push(Token { tok: Tok::Number(value), line: start_line });
            continue;
        }

        // Shabd - letters (Devanagari bhi chal sakti hai), digits aur underscore
        if c.is_alphabetic() || c == '_' || is_mark(c) {
            let start_line = line;
            let mut word = String::new();
            while i < chars.len() && is_word_char(chars[i]) {
                word.push(chars[i]);
                i += 1;
            }
            let lower = word.to_lowercase();
            match keyword(&lower) {
                Some(kw) => tokens.push(Token { tok: kw, line: start_line }),
                None => tokens.push(Token { tok: Tok::Ident(word), line: start_line }),
            }
            continue;
        }

        // Koi symbol (jaise ; { } ( ) = : .) BOLI me allowed nahi
        return Err(BoliError::line(
            line,
            format!("'{}' nahi chahiye, BOLI me sirf shabd, number aur \"text\" use hota hai", c),
        ));
    }

    tokens.push(Token { tok: Tok::Eof, line });
    Ok(tokens)
}
