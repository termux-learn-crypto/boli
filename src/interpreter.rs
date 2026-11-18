// BOLI ka Interpreter - AST ko chalakar output deta hai
// Environment scopes ki list hai: andar ka kaam (function) khatam hote hi scope pop ho jata hai

use std::collections::HashMap;
use std::io::{self, Write};

use crate::ast::{ArithOp, BoliError, CmpOp, Cond, Expr, Stmt};

// BOLI ke values
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
    Bool(bool),     // sahi / galat
    Khaali,         // khaali (null)
}

impl Value {
    // likho ke liye dikhne wala text
    pub fn dikhane_laayak(&self) -> String {
        match self {
            Value::Text(s) => s.clone(),
            // 28.0 ko 28 dikhao, 99.5 waise hi rahega
            Value::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 9.007199254740992e15 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Bool(b) => if *b { "sahi".to_string() } else { "galat".to_string() },
            Value::Khaali => "khaali".to_string(),
        }
    }

    // Galti message ke liye type ka naam
    fn type_naam(&self) -> &'static str {
        match self {
            Value::Text(_) => "text",
            Value::Number(_) => "number",
            Value::Bool(_) => "sahi/galat",
            Value::Khaali => "khaali",
        }
    }
}

// Kaam (function) ki definition
#[derive(Debug, Clone)]
struct Kaam {
    params: Vec<String>,
    body: Vec<Stmt>,
}

// wapas do se bahar nikalne ka signal (line number saath rakhta hai)
enum Flow {
    Normal,
    Wapas(Value, usize),
}

// Environment: scopes + kaam (functions)
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
    kaam: HashMap<String, Kaam>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()], // global scope
            kaam: HashMap::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        // Global scope hamesha rahega, usse kabhi pop nahi karenge
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    // Variable ki value dhundo, andar ke scope se bahar ki taraf
    fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    // rakho: pehle sabse andar wale scope me jo already hai usse badlo,
    // warna naya naam current scope me banao
    fn set(&mut self, name: &str, value: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return;
            }
        }
        self.scopes.last_mut().unwrap().insert(name.to_string(), value);
    }

    // Har baar top-level run ke liye clear statement
    pub fn run_program(&mut self, stmts: &[Stmt]) -> Result<(), BoliError> {
        match self.run_block(stmts)? {
            Flow::Normal => Ok(()),
            Flow::Wapas(_, line) => Err(BoliError::line(
                line,
                "'wapas do' sirf kaam ke andar use hota hai",
            )),
        }
    }

    // Statements ki list chalakar Flow lautao
    fn run_block(&mut self, stmts: &[Stmt]) -> Result<Flow, BoliError> {
        for stmt in stmts {
            if let Flow::Wapas(value, line) = self.run_stmt(stmt)? {
                return Ok(Flow::Wapas(value, line));
            }
        }
        Ok(Flow::Normal)
    }

    fn run_stmt(&mut self, stmt: &Stmt) -> Result<Flow, BoliError> {
        match stmt {
            // likho <expr>
            Stmt::Likho(expr, _line) => {
                let value = self.eval_expr(expr)?;
                // pipe band ho jaaye (jaise | head) to silent exit karo, panic nahi
                if writeln!(io::stdout(), "{}", value.dikhane_laayak()).is_err() {
                    std::process::exit(0);
                }
                Ok(Flow::Normal)
            }
            // <naam> rakho <expr>
            Stmt::Rakho { name, value, .. } => {
                let value = self.eval_expr(value)?;
                self.set(name, value);
                Ok(Flow::Normal)
            }
            // pucho <naam> "sawal"
            Stmt::Pucho { name, prompt, line } => {
                // prompt line ke andar hi print hota hai + pipe band ho jaaye to silent exit
                let prompt_ok = write!(io::stdout(), "{}", prompt).is_ok() && io::stdout().flush().is_ok();
                if !prompt_ok {
                    std::process::exit(0);
                }
                let mut line_in = String::new();
                io::stdin().read_line(&mut line_in).map_err(|_| {
                    BoliError::line(*line, "input nahi mil raha")
                })?;
                let trimmed = line_in.trim();
                // Number try karo, phir sahi/galat/khaali, warna text
                let value = if trimmed.is_empty() {
                    Value::Khaali
                } else if let Ok(n) = trimmed.parse::<f64>() {
                    Value::Number(n)
                } else if trimmed == "sahi" {
                    Value::Bool(true)
                } else if trimmed == "galat" {
                    Value::Bool(false)
                } else if trimmed == "khaali" {
                    Value::Khaali
                } else {
                    Value::Text(trimmed.to_string())
                };
                self.set(name, value);
                Ok(Flow::Normal)
            }
            // jodo/ghatao/guna/bhaag <var> <a> aur/se/ko <b>
            Stmt::ArithAssign { target, op, left, right, line } => {
                let left_v = self.eval_expr(left)?;
                let right_v = self.eval_expr(right)?;
                let value = eval_arith(*op, &left_v, &right_v, *line)?;
                self.set(target, value);
                Ok(Flow::Normal)
            }
            // agar <cond> to ... nahi to ... khatam
            Stmt::Agar { cond, then_block, else_block, .. } => {
                if self.eval_cond(cond)? {
                    self.run_block(then_block)
                } else if let Some(block) = else_block {
                    self.run_block(block)
                } else {
                    Ok(Flow::Normal)
                }
            }
            // 1 se 5 tak ... khatam
            Stmt::RangeLoop { from, to, body, line } => {
                let from_v = self.eval_expr(from)?;
                let to_v = self.eval_expr(to)?;
                let from_n = as_number(&from_v, *line, "se ke pehle")?;
                let to_n = as_number(&to_v, *line, "se ke baad")?;
                // ginti naam ka variable loop ke andar milta hai
                self.push_scope();
                let mut i = from_n;
                while i <= to_n {
                    // ginti seedha loop ke scope me rakho
                    self.scopes.last_mut().unwrap().insert("ginti".to_string(), Value::Number(i));
                    if let Flow::Wapas(value, wline) = self.run_block(body)? {
                        self.pop_scope();
                        return Ok(Flow::Wapas(value, wline));
                    }
                    i += 1.0;
                }
                self.pop_scope();
                Ok(Flow::Normal)
            }
            // jab tak <cond> to ... khatam
            Stmt::WhileLoop { cond, body, .. } => {
                loop {
                    if !self.eval_cond(cond)? {
                        break;
                    }
                    if let Flow::Wapas(value, line) = self.run_block(body)? {
                        return Ok(Flow::Wapas(value, line));
                    }
                }
                Ok(Flow::Normal)
            }
            // kaam <naam> <params> ... khatam  (definition ab register hoti hai)
            Stmt::KaamDef { name, params, body, line } => {
                // Duplicate kaam ka naam warning jaisa error hai
                if self.kaam.contains_key(name) {
                    return Err(BoliError::line(
                        *line,
                        format!("kaam '{}' pehle se bana hua hai", name),
                    ));
                }
                self.kaam.insert(name.clone(), Kaam { params: params.clone(), body: body.clone() });
                Ok(Flow::Normal)
            }
            // kaam ko bulana (statement me value discard hoti hai)
            Stmt::KaamCall { name, args, line } => {
                self.call_kaam(name, args.clone(), *line)?;
                Ok(Flow::Normal)
            }
            // wapas do <expr>
            Stmt::Wapas { value, line } => {
                let value = match value {
                    Some(expr) => self.eval_expr(expr)?,
                    None => Value::Khaali,
                };
                Ok(Flow::Wapas(value, *line))
            }
        }
    }

    // Kaam ko bulana: arguments caller ke scope me evaluate hote hain, value wapas milti hai
    fn call_kaam(&mut self, name: &str, args: Vec<Expr>, line: usize) -> Result<Value, BoliError> {
        let arg_values: Vec<Value> = args
            .iter()
            .map(|a| self.eval_expr(a))
            .collect::<Result<_, _>>()?;

        let kaam = self.kaam.get(name).cloned().ok_or_else(|| {
            BoliError::line(line, format!("kaam '{}' nahi mila", name))
        })?;

        if kaam.params.len() != arg_values.len() {
            return Err(BoliError::line(
                line,
                format!(
                    "kaam '{}' ko {} cheezein chahiye, {} di gai",
                    name,
                    kaam.params.len(),
                    arg_values.len()
                ),
            ));
        }

        // Naya scope: params ko arguments se jodo
        self.push_scope();
        for (param, value) in kaam.params.iter().zip(arg_values) {
            self.scopes.last_mut().unwrap().insert(param.clone(), value);
        }
        let result = self.run_block(&kaam.body);
        self.pop_scope();

        // wapas do ki value milti hai, warna khaali
        match result? {
            Flow::Normal => Ok(Value::Khaali),
            Flow::Wapas(value, _) => Ok(value),
        }
    }

    // Expression ki value nikalo
    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, BoliError> {
        match expr {
            Expr::Number(n, _) => Ok(Value::Number(*n)),
            Expr::Text(s, _) => Ok(Value::Text(s.clone())),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::Khaali(_) => Ok(Value::Khaali),
            Expr::Var(name, line) => self.get(name).ok_or_else(|| {
                BoliError::line(*line, format!("naam '{}' nahi mila", name))
            }),
            Expr::Arith(op, left, right, line) => {
                let left_v = self.eval_expr(left)?;
                let right_v = self.eval_expr(right)?;
                eval_arith(*op, &left_v, &right_v, *line)
            }
            Expr::Call(name, args, line) => self.call_kaam(name, args.clone(), *line),
        }
    }

    // Condition sahi hai ya galat
    fn eval_cond(&mut self, cond: &Cond) -> Result<bool, BoliError> {
        match cond {
            Cond::Cmp { left, op, right, line } => {
                let left_v = self.eval_expr(left)?;
                let right_v = self.eval_expr(right)?;
                Ok(compare(*op, &left_v, &right_v, *line)?)
            }
            Cond::Aur(left, right, _) => Ok(self.eval_cond(left)? && self.eval_cond(right)?),
            Cond::Ya(left, right, _) => Ok(self.eval_cond(left)? || self.eval_cond(right)?),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

// Number laao ya galti do
fn as_number(value: &Value, line: usize, kahan: &str) -> Result<f64, BoliError> {
    match value {
        Value::Number(n) => Ok(*n),
        other => Err(BoliError::line(
            line,
            format!("{} number chahiye, par {} mila", kahan, other.type_naam()),
        )),
    }
}

// jodo/ghatao/guna/bhaag ka hisaab
fn eval_arith(op: ArithOp, left: &Value, right: &Value, line: usize) -> Result<Value, BoliError> {
    // Text jodne ki ijjat (likho ke saath bahut kaam aati hai)
    if op == ArithOp::Jodo {
        if let (Value::Text(a), Value::Text(b)) = (left, right) {
            return Ok(Value::Text(format!("{}{}", a, b)));
        }
        // number + text = text jodne (jaise "Age: " aur 28)
        if matches!(left, Value::Text(_)) || matches!(right, Value::Text(_)) {
            return Ok(Value::Text(format!("{}{}", left.dikhane_laayak(), right.dikhane_laayak())));
        }
    }

    let a = as_number(left, line, op.naam())?;
    let b = as_number(right, line, op.naam())?;

    match op {
        ArithOp::Jodo => Ok(Value::Number(a + b)),
        ArithOp::Ghatao => Ok(Value::Number(a - b)),
        ArithOp::Guna => Ok(Value::Number(a * b)),
        ArithOp::Bhaag => {
            if b == 0.0 {
                return Err(BoliError::line(line, "zero se bhaag nahi kar sakte"));
            }
            Ok(Value::Number(a / b))
        }
    }
}

// bada / chhota / barabar ki tulna
fn compare(op: CmpOp, left: &Value, right: &Value, line: usize) -> Result<bool, BoliError> {
    // barabar har type ke saath chal sakti hai
    if op == CmpOp::Barabar {
        return Ok(values_equal(left, right));
    }

    // bada/chhota ke liye number (ya text) chahiye
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(match op {
            CmpOp::Bada => a > b,
            CmpOp::Chhota => a < b,
            CmpOp::BadaYaBarabar => a >= b,
            CmpOp::ChhotaYaBarabar => a <= b,
            CmpOp::Barabar => a == b,
        }),
        (Value::Text(a), Value::Text(b)) => Ok(match op {
            CmpOp::Bada => a > b,
            CmpOp::Chhota => a < b,
            CmpOp::BadaYaBarabar => a >= b,
            CmpOp::ChhotaYaBarabar => a <= b,
            CmpOp::Barabar => a == b,
        }),
        _ => Err(BoliError::line(
            line,
            format!(
                "'{}' ki tulna {} aur {} ke beech nahi ho sakti",
                op.naam(),
                left.type_naam(),
                right.type_naam()
            ),
        )),
    }
}

// Do values barabar hain ya nahi
fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::Text(a), Value::Text(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Khaali, Value::Khaali) => true,
        // Number aur Text mix ho to number ko text banake mila lo nahi, false hi rehte hain
        _ => false,
    }
}
