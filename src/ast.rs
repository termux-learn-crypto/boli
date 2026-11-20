// BOLI ka AST (Abstract Syntax Tree)
// Har node program ka ek tukda hai. Sab line numbers ke saath hain
// taaki error messages me "line 2 me ..." sahi dikh sake.

use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

// Galti (error) ka type - line number (aur lexer/parser phase par column) ke saath Hinglish message
#[derive(Debug, Clone)]
pub struct BoliError {
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub message: String,
    pub kind: Option<&'static str>,
    pub span: Option<Box<Span>>,
    // Runtime kaam-call trace (innermost sabse pehle print hota hai)
    pub trace: Vec<(String, usize)>,
}

impl BoliError {
    // Line number wali galti
    pub fn line(line: usize, message: impl Into<String>) -> Self {
        BoliError {
            line: Some(line),
            col: None,
            message: message.into(),
            kind: None,
            span: None,
            trace: Vec::new(),
        }
    }

    // Line + column wali galti (lexer/parser phase)
    pub fn line_col(line: usize, col: usize, message: impl Into<String>) -> Self {
        BoliError {
            line: Some(line),
            col: Some(col),
            message: message.into(),
            kind: None,
            span: None,
            trace: Vec::new(),
        }
    }

    pub fn line_col_kind_span(
        line: usize,
        col: usize,
        kind: &'static str,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        BoliError {
            line: Some(line),
            col: Some(col),
            message: message.into(),
            kind: Some(kind),
            span: Some(Box::new(span)),
            trace: Vec::new(),
        }
    }
}

impl fmt::Display for BoliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, self.col) {
            (Some(line), Some(col)) => {
                write!(f, "Galti: line {}, col {} me {}", line, col, self.message)?
            }
            (Some(line), None) => write!(f, "Galti: line {} me {}", line, self.message)?,
            (None, _) => write!(f, "Galti: {}", self.message)?,
        }
        for (name, call_line) in self.trace.iter().rev() {
            write!(f, "\n    kaam '{}' ke andar (line {})", name, call_line)?;
        }
        Ok(())
    }
}

impl std::error::Error for BoliError {}

// Hisab ke operator ka naam
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArithOp {
    Jodo,   // +
    Ghatao, // -
    Guna,   // *
    Bhaag,  // /
}

impl ArithOp {
    pub fn naam(&self) -> &'static str {
        match self {
            ArithOp::Jodo => "jodo",
            ArithOp::Ghatao => "ghatao",
            ArithOp::Guna => "guna",
            ArithOp::Bhaag => "bhaag",
        }
    }
}

// Tulna (comparison) ke operator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CmpOp {
    Bada,            // >
    Chhota,          // <
    BadaYaBarabar,   // >=
    ChhotaYaBarabar, // <=
    Barabar,         // ==
}

impl CmpOp {
    pub fn naam(&self) -> &'static str {
        match self {
            CmpOp::Bada => "bada",
            CmpOp::Chhota => "chhota",
            CmpOp::BadaYaBarabar => "bada ya barabar",
            CmpOp::ChhotaYaBarabar => "chhota ya barabar",
            CmpOp::Barabar => "barabar",
        }
    }
}

// Expression - value dene wala hissa
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64, usize),
    Text(String, usize),
    Bool(bool, usize),  // sahi / galat
    Khaali(usize),      // khaali (null)
    Var(String, usize), // variable ka naam
    // jodo/ghatao/guna/bhaag ka inline hisaab, jaise: likho jodo 5 aur 3
    Arith(ArithOp, Box<Expr>, Box<Expr>, usize),
    // Kaam ka bulana expression me bhi, jaise: likho jodo_kaam 2 3
    Call(String, Vec<Expr>, usize),
    // List: cheezein rakho 1 2 3
    List(Vec<Expr>, usize),
    // List ka ek hissa: cheezein hissa 1  (1 se ginti)
    Index(Box<Expr>, Box<Expr>, usize),
}

// Condition - agar aur jab tak me use hoti hai
#[derive(Debug, Clone, PartialEq)]
pub enum Cond {
    Cmp {
        left: Expr,
        op: CmpOp,
        right: Expr,
        line: usize,
    },
    Aur(Box<Cond>, Box<Cond>, usize), // dono sahi ho tabhi sahi
    Ya(Box<Cond>, Box<Cond>, usize),  // koi ek bhi sahi ho tab bhi sahi
}

// Statement - program ki ek line
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // likho <expr>
    Likho(Expr, usize),
    // <naam> rakho <expr>
    Rakho {
        name: String,
        value: Expr,
        line: usize,
    },
    // pucho <naam> "sawal"
    Pucho {
        name: String,
        prompt: String,
        line: usize,
    },
    // jodo <var> <a> aur <b>  (rakho NAHI, bas jodo/ghatao/guna/bhaag)
    ArithAssign {
        target: String,
        op: ArithOp,
        left: Expr,
        right: Expr,
        line: usize,
    },
    // agar <cond> to ... nahi to ... khatam
    Agar {
        cond: Cond,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
        line: usize,
    },
    // 1 se 5 tak ... khatam   (ginti naam ka variable apne aap milta hai)
    RangeLoop {
        from: Expr,
        to: Expr,
        body: Vec<Stmt>,
        line: usize,
    },
    // jab tak <cond> to ... khatam
    WhileLoop {
        cond: Cond,
        body: Vec<Stmt>,
        line: usize,
    },
    // kaam <naam> <params> ... khatam
    KaamDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        line: usize,
    },
    // <naam> <args>   (kaam ko bulana)
    KaamCall {
        name: String,
        args: Vec<Expr>,
        line: usize,
    },
    // wapas do <expr>
    Wapas {
        value: Option<Expr>,
        line: usize,
    },
    // ruk   (break - loop se bahar)
    Ruk(usize),
    // agla  (continue - agli ginti pe jao)
    Agla(usize),
    // laao "<path>.hl"   (module import - current environment me)
    Laao {
        path: String,
        line: usize,
    },
    // cheezein hissa 2 rakho 99   (list ke ek hisse ko badlo)
    ListAssign {
        target: String,
        index: Expr,
        value: Expr,
        line: usize,
    },
    // mapa rakho "naam" "Vilas" "umar" 28   (map literal)
    MapRakho {
        name: String,
        pairs: Vec<(Expr, Expr)>,
        line: usize,
    },
}

// Pura program statements ki list hai
pub type Program = Vec<Stmt>;

impl Expr {
    // Error line ke liye best-effort line number
    pub fn line_for_error(&self) -> usize {
        match self {
            Expr::Number(_, line) => *line,
            Expr::Text(_, line) => *line,
            Expr::Bool(_, line) => *line,
            Expr::Khaali(line) => *line,
            Expr::Var(_, line) => *line,
            Expr::Arith(_, _, _, line) => *line,
            Expr::Call(_, _, line) => *line,
            Expr::List(_, line) => *line,
            Expr::Index(_, _, line) => *line,
        }
    }
}

impl Stmt {
    // Error line ke liye best-effort line number
    pub fn line_for_error(&self) -> usize {
        match self {
            Stmt::Likho(_, line) => *line,
            Stmt::Rakho { line, .. } => *line,
            Stmt::Pucho { line, .. } => *line,
            Stmt::ArithAssign { line, .. } => *line,
            Stmt::Agar { line, .. } => *line,
            Stmt::RangeLoop { line, .. } => *line,
            Stmt::WhileLoop { line, .. } => *line,
            Stmt::KaamDef { line, .. } => *line,
            Stmt::KaamCall { line, .. } => *line,
            Stmt::Wapas { line, .. } => *line,
            Stmt::Ruk(line) => *line,
            Stmt::Agla(line) => *line,
            Stmt::Laao { line, .. } => *line,
            Stmt::ListAssign { line, .. } => *line,
            Stmt::MapRakho { line, .. } => *line,
        }
    }
}
