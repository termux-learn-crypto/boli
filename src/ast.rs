// BOLI ka AST (Abstract Syntax Tree)
// Har node program ka ek tukda hai. Sab line numbers ke saath hain
// taaki error messages me "line 2 me ..." sahi dikh sake.

use std::fmt;

// Galti (error) ka type - line number ke saath Hinglish message
#[derive(Debug, Clone)]
pub struct BoliError {
    pub line: Option<usize>,
    pub message: String,
}

impl BoliError {
    // Line number wali galti
    pub fn line(line: usize, message: impl Into<String>) -> Self {
        BoliError { line: Some(line), message: message.into() }
    }
}

impl fmt::Display for BoliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "Galti: line {} me {}", line, self.message),
            None => write!(f, "Galti: {}", self.message),
        }
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
    Bada,           // >
    Chhota,         // <
    BadaYaBarabar,  // >=
    ChhotaYaBarabar,// <=
    Barabar,        // ==
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
    Bool(bool, usize),   // sahi / galat
    Khaali(usize),       // khaali (null)
    Var(String, usize),  // variable ka naam
    // jodo/ghatao/guna/bhaag ka inline hisaab, jaise: likho jodo 5 aur 3
    Arith(ArithOp, Box<Expr>, Box<Expr>, usize),
    // Kaam ka bulana expression me bhi, jaise: likho jodo_kaam 2 3
    Call(String, Vec<Expr>, usize),
}

// Condition - agar aur jab tak me use hoti hai
#[derive(Debug, Clone, PartialEq)]
pub enum Cond {
    Cmp { left: Expr, op: CmpOp, right: Expr, line: usize },
    Aur(Box<Cond>, Box<Cond>, usize), // dono sahi ho tabhi sahi
    Ya(Box<Cond>, Box<Cond>, usize),  // koi ek bhi sahi ho tab bhi sahi
}

// Statement - program ki ek line
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // likho <expr>
    Likho(Expr, usize),
    // <naam> rakho <expr>
    Rakho { name: String, value: Expr, line: usize },
    // pucho <naam> "sawal"
    Pucho { name: String, prompt: String, line: usize },
    // jodo <var> <a> aur <b>  (rakho NAHI, bas jodo/ghatao/guna/bhaag)
    ArithAssign { target: String, op: ArithOp, left: Expr, right: Expr, line: usize },
    // agar <cond> to ... nahi to ... khatam
    Agar { cond: Cond, then_block: Vec<Stmt>, else_block: Option<Vec<Stmt>>, line: usize },
    // 1 se 5 tak ... khatam   (ginti naam ka variable apne aap milta hai)
    RangeLoop { from: Expr, to: Expr, body: Vec<Stmt>, line: usize },
    // jab tak <cond> to ... khatam
    WhileLoop { cond: Cond, body: Vec<Stmt>, line: usize },
    // kaam <naam> <params> ... khatam
    KaamDef { name: String, params: Vec<String>, body: Vec<Stmt>, line: usize },
    // <naam> <args>   (kaam ko bulana)
    KaamCall { name: String, args: Vec<Expr>, line: usize },
    // wapas do <expr>
    Wapas { value: Option<Expr>, line: usize },
}

// Pura program statements ki list hai
pub type Program = Vec<Stmt>;
