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
            Tok::Number(n) => format!("{}", n),
            Tok::Text(s) => format!("\"{}\"", s),
            Tok::Ident(s) => s.clone(),
            Tok::Newline => "line ka ant".to_string(),
            Tok::Eof => "file ka ant".to_string(),
        }
    }
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
                // Simple escape: backslash ke baad quote ya backslash aya to asli character dalo
                if ch == '\\' && i + 1 < chars.len() {
                    let next = chars[i + 1];
                    if next == '"' || next == '\\' {
                        text.push(next);
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

        // Number - ankh laage hue, ek dot ho sakti hai (jaise 99.50)
        if c.is_ascii_digit() {
            let start_line = line;
            let mut num = String::new();
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
            tokens.push(Token { tok: Tok::Number(value), line: start_line });
            continue;
        }

        // Shabd - letters (Devanagari bhi chal sakti hai), digits aur underscore
        if c.is_alphabetic() || c == '_' {
            let start_line = line;
            let mut word = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
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
