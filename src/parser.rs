// BOLI ka Parser - tokens se AST banata hai
// Line hi statement ka khatam hai, isliye statements newline pe alag hote hain

use crate::ast::{ArithOp, BoliError, CmpOp, Cond, Expr, Program, Stmt};
use crate::lexer::{Tok, Token, lex};
use crate::limits::{DEFAULT_MAX_CONDITION_DEPTH, DEFAULT_MAX_PARSE_DEPTH, env_limit};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    depth: usize,
    max_depth: usize,
    max_condition_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let max_depth = env_limit("BOLI_MAX_PARSE_DEPTH", DEFAULT_MAX_PARSE_DEPTH) as usize;
        let max_condition_depth =
            env_limit("BOLI_MAX_CONDITION_DEPTH", DEFAULT_MAX_CONDITION_DEPTH) as usize;
        Self::with_limits(tokens, max_depth, max_condition_depth)
    }

    pub fn with_limits(tokens: Vec<Token>, max_depth: usize, max_condition_depth: usize) -> Self {
        Parser {
            tokens,
            pos: 0,
            depth: 0,
            max_depth,
            max_condition_depth,
        }
    }

    // Abhi ka token (bina aage badhe)
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    // Agle n tokens aage dekhne ka (bina aage badhe)
    fn peek_at(&self, offset: usize) -> &Token {
        let idx = (self.pos + offset).min(self.tokens.len() - 1);
        &self.tokens[idx]
    }

    // Abhi ka token le aur aage badh
    fn next(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    // Saare newline chhod do
    fn skip_newlines(&mut self) {
        while self.peek().tok == Tok::Newline {
            self.pos += 1;
        }
    }

    // Token chahiye, warna Hinglish galti
    fn expect(&mut self, wanted: Tok, what: &str) -> Result<Token, BoliError> {
        let tok = self.peek().clone();
        if tok.tok == wanted {
            return Ok(self.next());
        }
        Err(BoliError::line_col(
            tok.line,
            tok.col,
            format!(
                "yahan '{}' chahiye tha, par '{}' mila",
                what,
                tok.describe()
            ),
        ))
    }

    // Statement ke ant me newline (ya file ka ant) hona chahiye
    fn expect_line_end(&mut self) -> Result<(), BoliError> {
        match self.peek().tok {
            Tok::Newline => {
                self.skip_newlines();
                Ok(())
            }
            Tok::Eof => Ok(()),
            _ => {
                let tok = self.peek();
                Err(BoliError::line_col(
                    tok.line,
                    tok.col,
                    format!(
                        "'{}' yahan nahi aa sakta, ek line me ek hi kaam hota hai",
                        tok.describe()
                    ),
                ))
            }
        }
    }

    // Identifier (kaam/variable ka naam) chahiye
    // 'kaam' aur 'hissa' keywords bhi naam ki tarah chal sakte hain
    fn expect_ident(&mut self) -> Result<(String, usize), BoliError> {
        let tok = self.peek().clone();
        match tok.tok {
            Tok::Ident(name) => {
                self.pos += 1;
                Ok((name, tok.line))
            }
            Tok::Kaam => {
                self.pos += 1;
                Ok(("kaam".to_string(), tok.line))
            }
            Tok::Hissa => {
                self.pos += 1;
                Ok(("hissa".to_string(), tok.line))
            }
            _ => Err(BoliError::line_col(
                tok.line,
                tok.col,
                format!("yahan naam chahiye tha, par '{}' mila", tok.describe()),
            )),
        }
    }

    // `rakho` ke baad jo likha hai usse value banao.
    // Ek atom => wahi value; ek se zyada atoms => list (cheezein rakho 1 2 3)
    fn parse_rakho_value(&mut self) -> Result<Expr, BoliError> {
        let line = self.peek().line;
        if !Self::atom_start(&self.peek().tok) {
            let tok = self.peek().clone();
            return Err(BoliError::line_col(
                line,
                tok.col,
                format!(
                    "rakho ke baad value chahiye thi, par '{}' mila",
                    tok.describe()
                ),
            ));
        }
        let mut atoms = vec![self.parse_atom()?];
        while Self::atom_start(&self.peek().tok) {
            atoms.push(self.parse_atom()?);
        }
        if atoms.len() == 1 {
            Ok(atoms.pop().unwrap())
        } else {
            Ok(Expr::List(atoms, line))
        }
    }

    // Pura program parse karo
    pub fn parse_program(&mut self) -> Result<Program, BoliError> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while self.peek().tok != Tok::Eof {
            stmts.push(self.parse_stmt()?);
            self.skip_newlines();
        }
        Ok(stmts)
    }

    // Block parse karo - Khatam ya Nahi pe ruk jata hai (unhe consume NAHI karta)
    fn parse_block(&mut self) -> Result<Vec<Stmt>, BoliError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek().tok {
                Tok::Khatam | Tok::Nahi | Tok::Eof => break,
                _ => stmts.push(self.parse_stmt()?),
            }
        }
        Ok(stmts)
    }

    // Kya yeh token expression ki shuruwat kar sakta hai?
    fn atom_start(tok: &Tok) -> bool {
        matches!(
            tok,
            Tok::Number(_)
                | Tok::Text(_)
                | Tok::Sahi
                | Tok::Galat
                | Tok::Khaali
                | Tok::Ident(_)
                | Tok::Kaam
                | Tok::Jodo
                | Tok::Ghatao
                | Tok::Guna
                | Tok::Bhaag
        )
    }

    fn parse_stmt(&mut self) -> Result<Stmt, BoliError> {
        self.depth += 1;
        if self.depth > self.max_depth {
            self.depth -= 1;
            return Err(BoliError::line_col(
                self.peek().line,
                self.peek().col,
                format!(
                    "code bahut zyada gehra hai (max nesting {})",
                    self.max_depth
                ),
            ));
        }
        let result = self.parse_stmt_inner();
        self.depth -= 1;
        result
    }

    fn parse_stmt_inner(&mut self) -> Result<Stmt, BoliError> {
        let tok = self.peek().clone();
        match tok.tok {
            // likho <expr>
            Tok::Likho => {
                self.next();
                let expr = self.parse_expr()?;
                let line = tok.line;
                self.expect_line_end()?;
                Ok(Stmt::Likho(expr, line))
            }
            // pucho <naam> "sawal"
            Tok::Pucho => {
                self.next();
                let (name, line) = self.expect_ident()?;
                let prompt = match self.peek().tok.clone() {
                    Tok::Text(s) => {
                        self.pos += 1;
                        s
                    }
                    _ => "Pucho: ".to_string(),
                };
                self.expect_line_end()?;
                Ok(Stmt::Pucho { name, prompt, line })
            }
            // jodo/ghatao/guna/bhaag <var> ...  (statement form, rakho nahi)
            Tok::Jodo | Tok::Ghatao | Tok::Guna | Tok::Bhaag => {
                let op = arith_op(&tok.tok).unwrap();
                self.next();
                // Statement form me pehla shabd variable ka naam hona chahiye
                let (target, line) = match self.peek().tok.clone() {
                    Tok::Ident(name) => {
                        self.pos += 1;
                        (name, tok.line)
                    }
                    Tok::Hissa => {
                        self.pos += 1;
                        ("hissa".to_string(), tok.line)
                    }
                    _ => {
                        return Err(BoliError::line_col(
                            tok.line,
                            tok.col,
                            format!(
                                "'{}' ke baad variable ka naam do, jaise '{} jawab 5 aur 3'",
                                op.naam(),
                                op.naam()
                            ),
                        ));
                    }
                };
                let (left, right) = self.parse_arith_tail(op, tok.line)?;
                self.expect_line_end()?;
                Ok(Stmt::ArithAssign {
                    target,
                    op,
                    left,
                    right,
                    line,
                })
            }
            // agar <cond> to ... khatam
            Tok::Agar => {
                self.next();
                let line = tok.line;
                let cond = self.parse_cond()?;
                self.expect(Tok::To, "to")?;
                let then_block = self.parse_block()?;
                let else_block = if self.peek().tok == Tok::Nahi {
                    self.next(); // nahi
                    self.expect(Tok::To, "to")?; // nahi to
                    Some(self.parse_block()?)
                } else {
                    None
                };
                self.expect(Tok::Khatam, "khatam")?;
                self.expect_line_end()?;
                Ok(Stmt::Agar {
                    cond,
                    then_block,
                    else_block,
                    line,
                })
            }
            // jab tak <cond> to ... khatam
            Tok::Jab => {
                self.next();
                let line = tok.line;
                self.expect(Tok::Tak, "tak")?; // jab tak
                let cond = self.parse_cond()?;
                self.expect(Tok::To, "to")?;
                let body = self.parse_block()?;
                self.expect(Tok::Khatam, "khatam")?;
                self.expect_line_end()?;
                Ok(Stmt::WhileLoop { cond, body, line })
            }
            // <num> se <num> tak ... khatam   (range loop)
            Tok::Number(_) => {
                let line = tok.line;
                let from = self.parse_atom()?;
                self.expect(Tok::Se, "se")?;
                let to = self.parse_atom()?;
                self.expect(Tok::Tak, "tak")?;
                let body = self.parse_block()?;
                self.expect(Tok::Khatam, "khatam")?;
                self.expect_line_end()?;
                Ok(Stmt::RangeLoop {
                    from,
                    to,
                    body,
                    line,
                })
            }
            // kaam keyword teen cheezein ho sakti hai:
            //   kaam rakho x   =>  "kaam" naam ka variable
            //   kaam <args>    =>  "kaam" naam ke kaam ka bulana
            //   kaam <naam>    =>  kaam definition (function banate hain)
            Tok::Kaam => {
                let line = tok.line;
                // "kaam hissa 1 rakho x" => variable "kaam" ka list hissa badlo
                if self.peek_at(1).tok == Tok::Hissa {
                    self.next(); // kaam
                    self.next(); // hissa
                    let index = self.parse_operand()?;
                    self.expect(Tok::Rakho, "rakho")?;
                    let value = self.parse_atom()?;
                    self.expect_line_end()?;
                    return Ok(Stmt::ListAssign {
                        target: "kaam".to_string(),
                        index,
                        value,
                        line,
                    });
                }
                // "kaam rakho ..." => variable "kaam"
                if self.peek_at(1).tok == Tok::Rakho {
                    self.next(); // kaam
                    self.next(); // rakho
                    let value = self.parse_rakho_value()?;
                    self.expect_line_end()?;
                    return Ok(Stmt::Rakho {
                        name: "kaam".to_string(),
                        value,
                        line,
                    });
                }
                // "kaam 5" / "kaam \"x\"" => ek kaam ka bulana (definition nahi)
                let call_path = matches!(
                    self.peek_at(1).tok,
                    Tok::Number(_) | Tok::Text(_) | Tok::Sahi | Tok::Galat | Tok::Khaali
                );
                if call_path {
                    self.next(); // kaam
                    let mut args = Vec::new();
                    while Self::atom_start(&self.peek().tok) {
                        args.push(self.atom(false)?);
                    }
                    self.expect_line_end()?;
                    return Ok(Stmt::KaamCall {
                        name: "kaam".to_string(),
                        args,
                        line,
                    });
                }
                // Warna yeh kaam definition hai
                self.next();
                let (name, _) = self.expect_ident()?;
                let mut params = Vec::new();
                while matches!(self.peek().tok, Tok::Ident(_) | Tok::Kaam | Tok::Hissa) {
                    let (param, _) = self.expect_ident()?;
                    params.push(param);
                }
                let body = self.parse_block()?;
                self.expect(Tok::Khatam, "khatam")?;
                self.expect_line_end()?;
                Ok(Stmt::KaamDef {
                    name,
                    params,
                    body,
                    line,
                })
            }
            // wapas do <expr>
            Tok::Wapas => {
                self.next();
                let line = tok.line;
                self.expect(Tok::Do, "do")?; // wapas do
                let value = match self.peek().tok {
                    Tok::Newline | Tok::Eof => None, // wapas do (bina value) = khaali
                    _ => Some(self.parse_expr()?),
                };
                self.expect_line_end()?;
                Ok(Stmt::Wapas { value, line })
            }
            // <naam> rakho <expr>  ya  <naam> hissa <n> rakho <expr>  ya  kaam_call <args>
            // 'hissa' bhi naam ki tarah chal sakta hai (jaise bhaag hissa 10 ko 2 se)
            Tok::Ident(_) | Tok::Hissa => {
                let name = match &tok.tok {
                    Tok::Ident(n) => n.clone(),
                    _ => "hissa".to_string(),
                };
                let line = tok.line;
                // Do dekhne pe pata chalta hai ki yeh assignment hai ya call
                if self.peek_at(1).tok == Tok::Rakho {
                    self.next(); // naam
                    self.next(); // rakho
                    let value = self.parse_rakho_value()?;
                    self.expect_line_end()?;
                    return Ok(Stmt::Rakho { name, value, line });
                }
                // <naam> hissa <index> rakho <expr>  =>  list ka ek hissa badlo
                if self.peek_at(1).tok == Tok::Hissa {
                    self.next(); // naam
                    self.next(); // hissa
                    let index = self.parse_operand()?;
                    self.expect(Tok::Rakho, "rakho")?;
                    // value me list nahi, seedha single atom hi expect karte hain
                    let value = self.parse_atom()?;
                    self.expect_line_end()?;
                    return Ok(Stmt::ListAssign {
                        target: name,
                        index,
                        value,
                        line,
                    });
                }
                // Warna yeh kaam ka bulana (function call) hai
                self.next(); // kaam ka naam
                let mut args = Vec::new();
                while Self::atom_start(&self.peek().tok) {
                    args.push(self.atom(false)?);
                }
                self.expect_line_end()?;
                Ok(Stmt::KaamCall { name, args, line })
            }
            // ruk (break) aur agla (continue) - loop ke andar use hote hain
            Tok::Ruk => {
                self.next();
                let line = tok.line;
                self.expect_line_end()?;
                Ok(Stmt::Ruk(line))
            }
            Tok::Agla => {
                self.next();
                let line = tok.line;
                self.expect_line_end()?;
                Ok(Stmt::Agla(line))
            }
            Tok::Khatam => Err(BoliError::line_col(
                tok.line,
                tok.col,
                "'khatam' bina kuch shuru kiye aaya, pehle agar/jab tak/kaam likho",
            )),
            Tok::Nahi => Err(BoliError::line_col(
                tok.line,
                tok.col,
                "'nahi to' sirf agar ke andar use hota hai",
            )),
            _ => Err(BoliError::line_col(
                tok.line,
                tok.col,
                format!(
                    "'{}' yahan se statement nahi shuru ho sakta",
                    tok.describe()
                ),
            )),
        }
    }

    // Expression - likho aur wapas do me use hoti hai
    fn parse_expr(&mut self) -> Result<Expr, BoliError> {
        let tok = self.peek().clone();
        // likho jodo 5 aur 3 jaisa inline hisaab
        if let Some(op) = arith_op(&tok.tok) {
            self.next();
            let line = tok.line;
            let (left, right) = self.parse_arith_tail(op, line)?;
            return Ok(Expr::Arith(op, Box::new(left), Box::new(right), line));
        }
        self.parse_atom()
    }

    // Sabse chhota hissa - number, text, variable, ya nested hisaab
    fn parse_atom(&mut self) -> Result<Expr, BoliError> {
        self.atom(true)
    }

    // Condition ke operands ke liye: variable ke baad atom = ALAG Operand, call nahi
    // jaise: "ginti2 5 se chhota hai" ka matlab ginti2 < 5 hai
    fn parse_operand(&mut self) -> Result<Expr, BoliError> {
        self.atom(false)
    }

    fn atom(&mut self, allow_call: bool) -> Result<Expr, BoliError> {
        let mut expr = self.atom_base(allow_call)?;
        // hissa (index) postfix: cheezein hissa 1  =>  cheezein ka pehla hissa
        while self.peek().tok == Tok::Hissa {
            let hissa_tok = self.next();
            let index = self.parse_operand()?;
            expr = Expr::Index(Box::new(expr), Box::new(index), hissa_tok.line);
        }
        Ok(expr)
    }

    // Ek base operand (index se pehle wala hissa)
    fn atom_base(&mut self, allow_call: bool) -> Result<Expr, BoliError> {
        self.depth += 1;
        if self.depth > self.max_depth {
            self.depth -= 1;
            return Err(BoliError::line_col(
                self.peek().line,
                self.peek().col,
                format!(
                    "expression bahut zyada gehra hai (max nesting {})",
                    self.max_depth
                ),
            ));
        }
        let result = self.atom_base_inner(allow_call);
        self.depth -= 1;
        result
    }

    fn atom_base_inner(&mut self, allow_call: bool) -> Result<Expr, BoliError> {
        let tok = self.peek().clone();
        match tok.tok {
            Tok::Number(n) => {
                self.pos += 1;
                Ok(Expr::Number(n, tok.line))
            }
            Tok::Text(s) => {
                self.pos += 1;
                Ok(Expr::Text(s, tok.line))
            }
            Tok::Sahi => {
                self.pos += 1;
                Ok(Expr::Bool(true, tok.line))
            }
            Tok::Galat => {
                self.pos += 1;
                Ok(Expr::Bool(false, tok.line))
            }
            Tok::Khaali => {
                self.pos += 1;
                Ok(Expr::Khaali(tok.line))
            }
            // 'kaam' aur 'hissa' value ki jagah bhi naam ki tarah chal sakte hain
            Tok::Ident(_) | Tok::Kaam | Tok::Hissa => {
                let name = match &tok.tok {
                    Tok::Ident(n) => n.clone(),
                    _ if tok.tok == Tok::Kaam => "kaam".to_string(),
                    _ => "hissa".to_string(),
                };
                self.pos += 1;
                // Agar baad me atom shuru ho raha hai to yeh kaam ka bulana hai
                // (arguments FLAT hote hain: f a b  =>  f(a, b), f a(b) nahi)
                if allow_call && Self::atom_start(&self.peek().tok) {
                    let mut args = Vec::new();
                    while Self::atom_start(&self.peek().tok) {
                        args.push(self.atom(false)?);
                    }
                    return Ok(Expr::Call(name, args, tok.line));
                }
                Ok(Expr::Var(name, tok.line))
            }
            // Nested hisaab: likho jodo 5 aur jodo 3 aur 4
            Tok::Jodo | Tok::Ghatao | Tok::Guna | Tok::Bhaag => {
                let op = arith_op(&tok.tok).unwrap();
                self.next();
                let line = tok.line;
                let (left, right) = self.parse_arith_tail(op, line)?;
                Ok(Expr::Arith(op, Box::new(left), Box::new(right), line))
            }
            _ => Err(BoliError::line_col(
                tok.line,
                tok.col,
                format!("yahan value chahiye thi, par '{}' mila", tok.describe()),
            )),
        }
    }

    // arith operation ka baaki hissa: <a> aur/se/ko ... <b> se
    fn parse_arith_tail(&mut self, op: ArithOp, line: usize) -> Result<(Expr, Expr), BoliError> {
        let left = self.parse_atom()?;
        match op {
            ArithOp::Jodo => {
                self.expect(Tok::Aur, "aur")?;
            }
            ArithOp::Ghatao => {
                self.expect(Tok::Se, "se")?;
            }
            ArithOp::Guna | ArithOp::Bhaag => {
                self.expect(Tok::Ko, "ko")?;
            }
        }
        let right = self.parse_atom()?;
        // guna/bhaag me antim 'se' bhi chahiye (5 ko 2 se)
        if op == ArithOp::Guna || op == ArithOp::Bhaag {
            self.expect(Tok::Se, "se")?;
        }
        let _ = line;
        Ok((left, right))
    }

    // Condition: ya (low precedence) upar aur (high precedence) neeche
    // "a ya b aur c" ka matlab hai: a ya (b aur c)
    fn parse_cond(&mut self) -> Result<Cond, BoliError> {
        let mut cond = self.parse_aur()?;
        let mut chain = 1usize;
        loop {
            let tok = self.peek().clone();
            if tok.tok == Tok::Ya {
                self.next();
                let line = tok.line;
                let right = self.parse_aur()?;
                chain += 1;
                if chain > self.max_condition_depth {
                    return Err(BoliError::line_col(
                        line,
                        tok.col,
                        format!(
                            "condition bahut lambi ho gayi (max {} 'ya')",
                            self.max_condition_depth
                        ),
                    ));
                }
                cond = Cond::Ya(Box::new(cond), Box::new(right), line);
            } else {
                break;
            }
        }
        Ok(cond)
    }

    // aur (AND) bind karta hai: comparison (aur comparison)*
    fn parse_aur(&mut self) -> Result<Cond, BoliError> {
        let mut cond = self.parse_comparison()?;
        let mut chain = 1usize;
        loop {
            let tok = self.peek().clone();
            if tok.tok == Tok::Aur {
                self.next();
                let line = tok.line;
                let right = self.parse_comparison()?;
                chain += 1;
                if chain > self.max_condition_depth {
                    return Err(BoliError::line_col(
                        line,
                        tok.col,
                        format!(
                            "condition bahut lambi ho gayi (max {} 'aur')",
                            self.max_condition_depth
                        ),
                    ));
                }
                cond = Cond::Aur(Box::new(cond), Box::new(right), line);
            } else {
                break;
            }
        }
        Ok(cond)
    }

    // Ek tulna: <expr> <expr> [se] bada/chhota/barabar [ya barabar] hai
    // jaise: umar 18 se bada hai   =>  umar > 18
    fn parse_comparison(&mut self) -> Result<Cond, BoliError> {
        let left = self.parse_operand()?;
        let line = self.peek().line;
        let right = self.parse_operand()?;

        // 'se' optional hai: "umar 18 se bada hai" ya "umar 18 bada hai" dono chalega
        if self.peek().tok == Tok::Se {
            self.next();
        }

        let op = match self.peek().tok {
            Tok::Bada => {
                self.next();
                // bada ya barabar ?
                if self.peek().tok == Tok::Ya && self.peek_at(1).tok == Tok::Barabar {
                    self.next(); // ya
                    self.next(); // barabar
                    CmpOp::BadaYaBarabar
                } else {
                    CmpOp::Bada
                }
            }
            Tok::Chhota => {
                self.next();
                if self.peek().tok == Tok::Ya && self.peek_at(1).tok == Tok::Barabar {
                    self.next(); // ya
                    self.next(); // barabar
                    CmpOp::ChhotaYaBarabar
                } else {
                    CmpOp::Chhota
                }
            }
            Tok::Barabar => {
                self.next();
                // barabar ya barabar matlab >= nahi, isliye seedha Barabar
                CmpOp::Barabar
            }
            _ => {
                let tok = self.peek().clone();
                return Err(BoliError::line_col(
                    tok.line,
                    tok.col,
                    format!(
                        "condition me 'bada', 'chhota' ya 'barabar' chahiye tha, par '{}' mila",
                        tok.describe()
                    ),
                ));
            }
        };

        self.expect(Tok::Hai, "hai")?;
        Ok(Cond::Cmp {
            left,
            op,
            right,
            line,
        })
    }
}

// Tok se ArithOp me conversion (None agar arith keyword nahi hai)
fn arith_op(tok: &Tok) -> Option<ArithOp> {
    match tok {
        Tok::Jodo => Some(ArithOp::Jodo),
        Tok::Ghatao => Some(ArithOp::Ghatao),
        Tok::Guna => Some(ArithOp::Guna),
        Tok::Bhaag => Some(ArithOp::Bhaag),
        _ => None,
    }
}

// Source code ko direct AST me (lex + parse, ek hi jagah se)
pub fn parse_source(source: &str) -> Result<Program, BoliError> {
    let tokens = lex(source)?;
    Parser::new(tokens).parse_program()
}
