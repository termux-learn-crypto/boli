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
    List(Vec<Value>), // cheezein rakho 1 2 3
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
            Value::List(items) => items
                .iter()
                .map(|v| v.dikhane_laayak())
                .collect::<Vec<_>>()
                .join(", "),
        }
    }

    // Galti message ke liye type ka naam
    fn type_naam(&self) -> &'static str {
        match self {
            Value::Text(_) => "text",
            Value::Number(_) => "number",
            Value::Bool(_) => "sahi/galat",
            Value::Khaali => "khaali",
            Value::List(_) => "list",
        }
    }
}

// Kaam (function) ki definition
#[derive(Debug, Clone)]
struct Kaam {
    params: Vec<String>,
    body: Vec<Stmt>,
}

// wapas do se bahar nikalne ka signal / loop ke andar ka control flow
enum Flow {
    Normal,
    Wapas(Value, usize),
    Ruk(usize),  // break
    Agla,        // continue
}

// Environment: scopes + kaam (functions) + safety limits
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
    kaam: HashMap<String, Kaam>,
    // safety limits (BOLI_MAX_STEPS etc. env se badal sakte ho)
    steps: u64,
    max_steps: u64,
    max_call_depth: usize,
    max_string_bytes: usize,
    call_depth: usize,
    // tests ke liye output capture (likho run_program ki bajaye yahan push hoga)
    captured: Option<Vec<String>>,
}

impl Environment {
    pub fn new() -> Self {
        // Limits env variables se badal sakte hain (defaults safe hain)
        let max_steps = env_limit("BOLI_MAX_STEPS", 100_000);
        let max_call_depth = env_limit("BOLI_MAX_CALL_DEPTH", 64) as usize;
        let max_string_bytes = env_limit("BOLI_MAX_STRING_BYTES", 1024 * 1024) as usize;
        Environment {
            scopes: vec![HashMap::new()], // global scope
            kaam: HashMap::new(),
            steps: 0,
            max_steps,
            max_call_depth,
            max_string_bytes,
            call_depth: 0,
            captured: None,
        }
    }

    // Tests ke liye: likho ka output collect karna start karo
    #[doc(hidden)]
    pub fn collect_output(&mut self) {
        self.captured = Some(Vec::new());
    }

    // Tests ke liye: ab tak ka collected output nikalo
    #[doc(hidden)]
    pub fn take_output(&mut self) -> Vec<String> {
        self.captured.take().unwrap_or_default()
    }

    // Tests ke liye custom limits
    pub fn with_limits(max_steps: u64, max_call_depth: usize, max_string_bytes: usize) -> Self {
        let mut env = Self::new();
        env.max_steps = max_steps;
        env.max_call_depth = max_call_depth;
        env.max_string_bytes = max_string_bytes;
        env
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

    // rakho: hamesha current (sabse andar wale) scope me likho.
    // Badi scope ki value ko chup-chaap overwrite NAHI karte (Python-style nonlocal assignment bug se bacho).
    fn set(&mut self, name: &str, value: Value) {
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
            Flow::Ruk(line) => Err(BoliError::line(
                line,
                "'ruk' sirf loop ke andar use hota hai",
            )),
            Flow::Agla => Err(BoliError::line(
                1,
                "'agla' sirf loop ke andar use hota hai",
            )),
        }
    }

    // Statements ki list chalakar Flow lautao
    fn run_block(&mut self, stmts: &[Stmt]) -> Result<Flow, BoliError> {
        for stmt in stmts {
            match self.run_stmt(stmt)? {
                Flow::Normal => {}
                flow => return Ok(flow),
            }
        }
        Ok(Flow::Normal)
    }

    fn run_stmt(&mut self, stmt: &Stmt) -> Result<Flow, BoliError> {
        // Har statement ek step count hota hai (infinite loop rokta hai)
        self.steps += 1;
        if self.steps > self.max_steps {
            return Err(BoliError::line(
                stmt.line_for_error(),
                format!("program bahut zyada steps chal gaya (max {}), infinite loop to nahi?", self.max_steps),
            ));
        }
        match stmt {
            // likho <expr>
            Stmt::Likho(expr, _line) => {
                let value = self.eval_expr(expr)?;
                let text = value.dikhane_laayak();
                if let Some(v) = &mut self.captured {
                    v.push(text);
                    return Ok(Flow::Normal);
                }
                // pipe band ho jaaye (jaise | head) to silent exit karo, panic nahi
                if writeln!(io::stdout(), "{}", text).is_err() {
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
                // capture mode me stdin avoid karo (tests hang na hon)
                if self.captured.is_some() {
                    self.set(name, Value::Khaali);
                    return Ok(Flow::Normal);
                }
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
                // Number try karo, phir sahi/galat/khaali, warna text.
                // "nan"/"inf" finite nahi hain isliye woh text hi rahenge (input rules loose hain)
                let value = if trimmed.is_empty() {
                    Value::Khaali
                } else if let Ok(n) = trimmed.parse::<f64>() {
                    if n.is_finite() {
                        Value::Number(n)
                    } else {
                        Value::Text(trimmed.to_string())
                    }
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
                let value = eval_arith(*op, &left_v, &right_v, *line, self.max_string_bytes)?;
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
                // iteration count se chalakar float drift se bacho
                let count = if to_n >= from_n {
                    (to_n - from_n).floor() as i64 + 1
                } else {
                    0
                };
                // ginti naam ka variable loop ke andar milta hai
                self.push_scope();
                // BUG-001 fix: error path par bhi scope pop ho
                let result = (|| {
                    for i in 0..count {
                        self.steps += 1;
                        if self.steps > self.max_steps {
                            return Err(BoliError::line(
                                *line,
                                format!("program bahut zyada steps chal gaya (max {}), infinite loop to nahi?", self.max_steps),
                            ));
                        }
                        // ginti seedha loop ke scope me rakho
                        self.scopes.last_mut().unwrap().insert("ginti".to_string(), Value::Number(from_n + i as f64));
                        match self.run_block(body)? {
                            Flow::Normal => {}
                            Flow::Wapas(value, wline) => return Ok(Flow::Wapas(value, wline)),
                            Flow::Ruk(_) => break,  // ruk = loop se bahar
                            Flow::Agla => {}
                        }
                    }
                    Ok(Flow::Normal)
                })();
                self.pop_scope();
                result
            }
            // jab tak <cond> to ... khatam
            Stmt::WhileLoop { cond, body, .. } => {
                loop {
                    // har iteration guard (body khaali ho to bhi infinite loop se bacho)
                    self.steps += 1;
                    if self.steps > self.max_steps {
                        return Err(BoliError::line(
                            stmt.line_for_error(),
                            format!("program bahut zyada steps chal gaya (max {}), infinite loop to nahi?", self.max_steps),
                        ));
                    }
                    if !self.eval_cond(cond)? {
                        break;
                    }
                    match self.run_block(body)? {
                        Flow::Normal => {}
                        Flow::Wapas(value, line) => return Ok(Flow::Wapas(value, line)),
                        Flow::Ruk(_) => break,
                        Flow::Agla => {}
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
            // ruk (break)
            Stmt::Ruk(line) => Ok(Flow::Ruk(*line)),
            // agla (continue)
            Stmt::Agla(_) => Ok(Flow::Agla),
            // cheezein hissa <n> rakho <value>  =>  list ka ek hissa badlo
            Stmt::ListAssign { target, index, value, line } => {
                self.list_assign(target, index, value, *line)?;
                Ok(Flow::Normal)
            }
        }
    }

    // list ka ek hissa badlo (1 se ginti)
    fn list_assign(&mut self, target: &str, index: &Expr, value: &Expr, line: usize) -> Result<(), BoliError> {
        let idx_v = self.eval_expr(index)?;
        let idx = as_number(&idx_v, line, "hissa ke liye")?;
        if idx.fract() != 0.0 || idx < 1.0 {
            return Err(BoliError::line(line, "hissa 1 se shuru hone wali integer ginti honi chahiye"));
        }
        let i = idx as usize;
        let new_value = self.eval_expr(value)?;
        let mut current = match self.get(target) {
            Some(v) => v,
            None => return Err(BoliError::line(line, format!("naam '{}' nahi mila", target))),
        };
        match &mut current {
            Value::List(items) => {
                if i > items.len() {
                    return Err(BoliError::line(line, format!("'{}' me sirf {} hisse hain, {} maanga", target, items.len(), i)));
                }
                items[i - 1] = new_value;
            }
            _ => {
                return Err(BoliError::line(line, format!("'{}' list nahi hai", target)));
            }
        }
        self.set(target, current);
        Ok(())
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

        // Recursion depth limit (BOLI_MAX_CALL_DEPTH)
        if self.call_depth >= self.max_call_depth {
            return Err(BoliError::line(
                line,
                format!("kaam '{}' bahut zyada gehra ho gaya (max {}), infinite recursion to nahi?", name, self.max_call_depth),
            ));
        }
        self.call_depth += 1;

        // Naya scope: params ko arguments se jodo
        self.push_scope();
        for (param, value) in kaam.params.iter().zip(arg_values) {
            self.scopes.last_mut().unwrap().insert(param.clone(), value);
        }
        let result = self.run_block(&kaam.body);
        self.pop_scope();
        self.call_depth -= 1;

        // wapas do ki value milti hai, warna khaali
        match result? {
            Flow::Normal => Ok(Value::Khaali),
            Flow::Wapas(value, _) => Ok(value),
            Flow::Ruk(rline) => Err(BoliError::line(
                rline,
                "'ruk' sirf loop ke andar use hota hai",
            )),
            Flow::Agla => Err(BoliError::line(
                line,
                "'agla' sirf loop ke andar use hota hai",
            )),
        }
    }

    // Expression ki value nikalo
    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, BoliError> {
        match expr {
            Expr::Number(n, _) => Ok(Value::Number(*n)),
            Expr::Text(s, _) => Ok(Value::Text(s.clone())),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::Khaali(_) => Ok(Value::Khaali),
            Expr::Var(name, line) => {
                // Pehle variable dhundo, phir zero-argument kaam (BUG fix: likho jawab_do)
                if let Some(v) = self.get(name) {
                    return Ok(v);
                }
                if let Some(kaam) = self.kaam.get(name) {
                    if kaam.params.is_empty() {
                        return self.call_kaam(name, Vec::new(), *line);
                    }
                }
                Err(BoliError::line(*line, format!("naam '{}' nahi mila", name)))
            }
            Expr::Arith(op, left, right, line) => {
                let left_v = self.eval_expr(left)?;
                let right_v = self.eval_expr(right)?;
                eval_arith(*op, &left_v, &right_v, *line, self.max_string_bytes)
            }
            Expr::Call(name, args, line) => self.call_kaam(name, args.clone(), *line),
            Expr::List(items, _) => {
                let values = items
                    .iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Value::List(values))
            }
            Expr::Index(target, index, line) => {
                let target_v = self.eval_expr(target)?;
                let idx_v = self.eval_expr(index)?;
                let idx = as_number(&idx_v, *line, "hissa ke liye")?;
                if idx.fract() != 0.0 || idx < 1.0 {
                    return Err(BoliError::line(*line, "hissa 1 se shuru hone wali integer ginti honi chahiye"));
                }
                let i = idx as usize;
                match target_v {
                    Value::List(items) => {
                        if i > items.len() {
                            return Err(BoliError::line(
                                *line,
                                format!("list me sirf {} hisse hain, {} maanga", items.len(), i),
                            ));
                        }
                        Ok(items[i - 1].clone())
                    }
                    Value::Text(s) => {
                        match s.chars().nth(i - 1) {
                            Some(c) => Ok(Value::Text(c.to_string())),
                            None => Err(BoliError::line(
                                *line,
                                format!("text me sirf {} unicode chars hain, {} maanga", s.chars().count(), i),
                            )),
                        }
                    }
                    other => Err(BoliError::line(
                        *line,
                        format!("hissa sirf list ya text pe chal sakta hai, {} pe nahi", other.type_naam()),
                    )),
                }
            }
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

// Env variable se positive number lo, warna default (galat parse pe default)
fn env_limit(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(default)
}

// string size limit ke saath text banane ka helper
fn checked_text(s: String, line: usize, max_string_bytes: usize) -> Result<Value, BoliError> {
    if s.len() > max_string_bytes {
        return Err(BoliError::line(
            line,
            format!("text bahut bada ho gaya (max {} bytes)", max_string_bytes),
        ));
    }
    Ok(Value::Text(s))
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
fn eval_arith(op: ArithOp, left: &Value, right: &Value, line: usize, max_string_bytes: usize) -> Result<Value, BoliError> {
    // Lists jodne ki ijjat (concatenate)
    if op == ArithOp::Jodo {
        if let (Value::List(a), Value::List(b)) = (left, right) {
            let mut items = a.clone();
            items.extend(b.iter().cloned());
            return Ok(Value::List(items));
        }
    }

    // Text jodne ki ijjat (likho ke saath bahut kaam aati hai)
    if op == ArithOp::Jodo {
        if let (Value::Text(a), Value::Text(b)) = (left, right) {
            return checked_text(format!("{}{}", a, b), line, max_string_bytes);
        }
        // number + text = text jodne (jaise "Age: " aur 28)
        if matches!(left, Value::Text(_)) || matches!(right, Value::Text(_)) {
            return checked_text(
                format!("{}{}", left.dikhane_laayak(), right.dikhane_laayak()),
                line,
                max_string_bytes,
            );
        }
    }

    let a = as_number(left, line, op.naam())?;
    let b = as_number(right, line, op.naam())?;

    let result = match op {
        ArithOp::Jodo => a + b,
        ArithOp::Ghatao => a - b,
        ArithOp::Guna => a * b,
        ArithOp::Bhaag => {
            if b == 0.0 {
                return Err(BoliError::line(line, "zero se bhaag nahi kar sakte"));
            }
            a / b
        }
    };
    // Hisab ka result finite hona chahiye (overflow aur infinities rokko)
    if !result.is_finite() {
        return Err(BoliError::line(
            line,
            format!("'{}' ka result finite nahi raha, number bahut bada hai", op.naam()),
        ));
    }
    Ok(Value::Number(result))
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
        (Value::List(a), Value::List(b)) => a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y)),
        // Number aur Text mix ho to number ko text banake mila lo nahi, false hi rehte hain
        _ => false,
    }
}
