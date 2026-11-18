// BOLI ka Interpreter - AST ko chalakar output deta hai
// Environment scopes ki list hai: andar ka kaam (function) khatam hote hi scope pop ho jata hai

use std::collections::HashMap;
use std::io::{self, Write};

use crate::ast::{ArithOp, BoliError, CmpOp, Cond, Expr, Stmt};
use crate::limits::{
    DEFAULT_MAX_CALL_DEPTH, DEFAULT_MAX_EVAL_DEPTH, DEFAULT_MAX_LIST_ITEMS, DEFAULT_MAX_STEPS,
    DEFAULT_MAX_STRING_BYTES, env_limit,
};

// BOLI ke values
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
    Bool(bool),       // sahi / galat
    Khaali,           // khaali (null)
    List(Vec<Value>), // cheezein rakho 1 2 3
}

impl Value {
    pub fn dikhane_laayak(&self) -> String {
        match self {
            Value::Text(s) => s.clone(),
            Value::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 9.007199254740992e15 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Bool(b) => {
                if *b {
                    "sahi".to_string()
                } else {
                    "galat".to_string()
                }
            }
            Value::Khaali => "khaali".to_string(),
            Value::List(items) => items
                .iter()
                .map(|v| v.dikhane_laayak())
                .collect::<Vec<_>>()
                .join(", "),
        }
    }

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

#[derive(Debug, Clone)]
struct Kaam {
    params: Vec<String>,
    body: Vec<Stmt>,
}

enum Flow {
    Normal,
    Wapas(Value, usize),
    Ruk(usize),
    Agla(usize),
}

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
    kaam: HashMap<String, Kaam>,
    steps: u64,
    max_steps: u64,
    max_call_depth: usize,
    max_string_bytes: usize,
    max_list_items: usize,
    max_eval_depth: usize,
    max_condition_depth: usize,
    call_depth: usize,
    eval_depth: usize,
    cond_depth: usize,
    captured: Option<Vec<String>>,
}

impl Environment {
    pub fn new() -> Self {
        let max_steps = env_limit("BOLI_MAX_STEPS", DEFAULT_MAX_STEPS);
        let max_call_depth = env_limit("BOLI_MAX_CALL_DEPTH", DEFAULT_MAX_CALL_DEPTH) as usize;
        let max_string_bytes =
            env_limit("BOLI_MAX_STRING_BYTES", DEFAULT_MAX_STRING_BYTES) as usize;
        let max_list_items = env_limit("BOLI_MAX_LIST_ITEMS", DEFAULT_MAX_LIST_ITEMS) as usize;
        let max_eval_depth = env_limit("BOLI_MAX_EVAL_DEPTH", DEFAULT_MAX_EVAL_DEPTH) as usize;
        let max_condition_depth = env_limit(
            "BOLI_MAX_CONDITION_DEPTH",
            crate::limits::DEFAULT_MAX_CONDITION_DEPTH,
        ) as usize;
        Environment {
            scopes: vec![HashMap::new()],
            kaam: HashMap::new(),
            steps: 0,
            max_steps,
            max_call_depth,
            max_string_bytes,
            max_list_items,
            max_eval_depth,
            max_condition_depth,
            call_depth: 0,
            eval_depth: 0,
            cond_depth: 0,
            captured: None,
        }
    }

    #[doc(hidden)]
    pub fn collect_output(&mut self) {
        self.captured = Some(Vec::new());
    }

    #[doc(hidden)]
    pub fn take_output(&mut self) -> Vec<String> {
        self.captured.take().unwrap_or_default()
    }

    pub fn with_limits(max_steps: u64, max_call_depth: usize, max_string_bytes: usize) -> Self {
        let mut env = Self::new();
        env.max_steps = max_steps;
        env.max_call_depth = max_call_depth;
        env.max_string_bytes = max_string_bytes;
        env
    }

    pub fn with_list_limit(mut self, max_list_items: usize) -> Self {
        self.max_list_items = max_list_items;
        self
    }

    pub fn with_eval_depth_limit(mut self, max_eval_depth: usize) -> Self {
        self.max_eval_depth = max_eval_depth;
        self
    }

    pub fn with_condition_depth_limit(mut self, max_condition_depth: usize) -> Self {
        self.max_condition_depth = max_condition_depth;
        self
    }

    pub fn reset_steps(&mut self) {
        self.steps = 0;
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    fn set(&mut self, name: &str, value: Value) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), value);
    }

    pub fn run_program(&mut self, stmts: &[Stmt]) -> Result<(), BoliError> {
        self.reset_steps();
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
            Flow::Agla(line) => Err(BoliError::line(
                line,
                "'agla' sirf loop ke andar use hota hai",
            )),
        }
    }

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
        self.steps += 1;
        if self.steps > self.max_steps {
            return Err(BoliError::line(
                stmt.line_for_error(),
                format!(
                    "program bahut zyada steps chal gaya (max {}), infinite loop to nahi?",
                    self.max_steps
                ),
            ));
        }

        match stmt {
            Stmt::Likho(expr, _line) => {
                let value = self.eval_expr(expr)?;
                let text = value.dikhane_laayak();
                if let Some(v) = &mut self.captured {
                    v.push(text);
                    return Ok(Flow::Normal);
                }
                if writeln!(io::stdout(), "{}", text).is_err() {
                    std::process::exit(0);
                }
                Ok(Flow::Normal)
            }
            Stmt::Rakho { name, value, .. } => {
                let value = self.eval_expr(value)?;
                self.set(name, value);
                Ok(Flow::Normal)
            }
            Stmt::Pucho { name, prompt, line } => {
                if self.captured.is_some() {
                    self.set(name, Value::Khaali);
                    return Ok(Flow::Normal);
                }
                let prompt_ok =
                    write!(io::stdout(), "{}", prompt).is_ok() && io::stdout().flush().is_ok();
                if !prompt_ok {
                    std::process::exit(0);
                }
                // input ko chunked-read karke max_string_bytes pe kaato (unbounded alloc nahi)
                let mut input_buf: Vec<u8> = Vec::new();
                {
                    use std::io::BufRead;
                    let mut stdin = io::stdin().lock();
                    loop {
                        let chunk = stdin
                            .fill_buf()
                            .map_err(|_| BoliError::line(*line, "input nahi mil raha"))?;
                        if chunk.is_empty() {
                            break; // EOF
                        }
                        match chunk.iter().position(|&b| b == b'\n') {
                            Some(pos) => {
                                input_buf.extend_from_slice(&chunk[..pos]);
                                stdin.consume(pos + 1);
                                break;
                            }
                            None => {
                                if input_buf.len() + chunk.len() > self.max_string_bytes {
                                    return Err(BoliError::line(
                                        *line,
                                        format!(
                                            "input text bahut bada hai (max {} bytes)",
                                            self.max_string_bytes
                                        ),
                                    ));
                                }
                                input_buf.extend_from_slice(chunk);
                                let len = chunk.len();
                                stdin.consume(len);
                            }
                        }
                    }
                }
                if input_buf.len() > self.max_string_bytes {
                    return Err(BoliError::line(
                        *line,
                        format!(
                            "input text bahut bada hai (max {} bytes)",
                            self.max_string_bytes
                        ),
                    ));
                }
                let line_in = String::from_utf8_lossy(&input_buf).into_owned();
                let trimmed = line_in.trim();
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
            Stmt::ArithAssign {
                target,
                op,
                left,
                right,
                line,
            } => {
                let left_v = self.eval_expr(left)?;
                let right_v = self.eval_expr(right)?;
                let value = eval_arith(
                    *op,
                    &left_v,
                    &right_v,
                    *line,
                    self.max_string_bytes,
                    self.max_list_items,
                )?;
                self.set(target, value);
                Ok(Flow::Normal)
            }
            Stmt::Agar {
                cond,
                then_block,
                else_block,
                ..
            } => {
                if self.eval_cond(cond)? {
                    self.run_block(then_block)
                } else if let Some(block) = else_block {
                    self.run_block(block)
                } else {
                    Ok(Flow::Normal)
                }
            }
            Stmt::RangeLoop {
                from,
                to,
                body,
                line,
            } => {
                let from_v = self.eval_expr(from)?;
                let to_v = self.eval_expr(to)?;
                let from_n = as_number(&from_v, *line, "se ke pehle")?;
                let to_n = as_number(&to_v, *line, "se ke baad")?;

                let count: i64 = if to_n >= from_n {
                    let distance = (to_n - from_n).floor();
                    if !distance.is_finite() {
                        return Err(BoliError::line(*line, "range bahut badi hai"));
                    }
                    match distance.to_i64_checked() {
                        Some(d) => d.checked_add(1).ok_or_else(|| {
                            BoliError::line(*line, "range itni badi hai ki chal nahi sakti")
                        })?,
                        None => return Err(BoliError::line(*line, "range bahut badi hai")),
                    }
                } else {
                    0
                };

                self.push_scope();
                let result = (|| {
                    for i in 0..count {
                        self.steps += 1;
                        if self.steps > self.max_steps {
                            return Err(BoliError::line(
                                *line,
                                format!(
                                    "program bahut zyada steps chal gaya (max {}), infinite loop to nahi?",
                                    self.max_steps
                                ),
                            ));
                        }
                        self.scopes
                            .last_mut()
                            .unwrap()
                            .insert("ginti".to_string(), Value::Number(from_n + i as f64));
                        match self.run_block(body)? {
                            Flow::Normal => {}
                            Flow::Wapas(value, wline) => return Ok(Flow::Wapas(value, wline)),
                            Flow::Ruk(_) => break,
                            Flow::Agla(_) => {}
                        }
                    }
                    Ok(Flow::Normal)
                })();
                self.pop_scope();
                result
            }
            Stmt::WhileLoop { cond, body, .. } => {
                loop {
                    self.steps += 1;
                    if self.steps > self.max_steps {
                        return Err(BoliError::line(
                            stmt.line_for_error(),
                            format!(
                                "program bahut zyada steps chal gaya (max {}), infinite loop to nahi?",
                                self.max_steps
                            ),
                        ));
                    }
                    if !self.eval_cond(cond)? {
                        break;
                    }
                    match self.run_block(body)? {
                        Flow::Normal => {}
                        Flow::Wapas(value, line) => return Ok(Flow::Wapas(value, line)),
                        Flow::Ruk(_) => break,
                        Flow::Agla(_) => {}
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::KaamDef {
                name,
                params,
                body,
                line,
            } => {
                if self.kaam.contains_key(name) {
                    return Err(BoliError::line(
                        *line,
                        format!("kaam '{}' pehle se bana hua hai", name),
                    ));
                }
                self.kaam.insert(
                    name.clone(),
                    Kaam {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
                Ok(Flow::Normal)
            }
            Stmt::KaamCall { name, args, line } => {
                self.call_kaam(name, args.clone(), *line)?;
                Ok(Flow::Normal)
            }
            Stmt::Wapas { value, line } => {
                let value = match value {
                    Some(expr) => self.eval_expr(expr)?,
                    None => Value::Khaali,
                };
                Ok(Flow::Wapas(value, *line))
            }
            Stmt::Ruk(line) => Ok(Flow::Ruk(*line)),
            Stmt::Agla(line) => Ok(Flow::Agla(*line)),
            Stmt::ListAssign {
                target,
                index,
                value,
                line,
            } => {
                self.list_assign(target, index, value, *line)?;
                Ok(Flow::Normal)
            }
        }
    }

    fn list_assign(
        &mut self,
        target: &str,
        index: &Expr,
        value: &Expr,
        line: usize,
    ) -> Result<(), BoliError> {
        let idx_v = self.eval_expr(index)?;
        let idx = as_number(&idx_v, line, "hissa ke liye")?;
        if idx.fract() != 0.0 || idx < 1.0 {
            return Err(BoliError::line(
                line,
                "hissa 1 se shuru hone wali integer ginti honi chahiye",
            ));
        }
        let i = idx as usize;
        let new_value = self.eval_expr(value)?;
        let mut current = match self.get(target) {
            Some(v) => v,
            None => {
                return Err(BoliError::line(
                    line,
                    format!("naam '{}' nahi mila", target),
                ));
            }
        };
        match &mut current {
            Value::List(items) => {
                if i > items.len() {
                    return Err(BoliError::line(
                        line,
                        format!(
                            "'{}' me sirf {} hisse hain, {} maanga",
                            target,
                            items.len(),
                            i
                        ),
                    ));
                }
                items[i - 1] = new_value;
            }
            _ => return Err(BoliError::line(line, format!("'{}' list nahi hai", target))),
        }
        self.set(target, current);
        Ok(())
    }

    fn call_kaam(&mut self, name: &str, args: Vec<Expr>, line: usize) -> Result<Value, BoliError> {
        let arg_values: Vec<Value> = args
            .iter()
            .map(|a| self.eval_expr(a))
            .collect::<Result<_, _>>()?;
        let kaam = self
            .kaam
            .get(name)
            .cloned()
            .ok_or_else(|| BoliError::line(line, format!("kaam '{}' nahi mila", name)))?;
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
        if self.call_depth >= self.max_call_depth {
            return Err(BoliError::line(
                line,
                format!(
                    "kaam '{}' bahut zyada gehra ho gaya (max {}), infinite recursion to nahi?",
                    name, self.max_call_depth
                ),
            ));
        }
        self.call_depth += 1;
        self.push_scope();
        for (param, value) in kaam.params.iter().zip(arg_values) {
            self.scopes.last_mut().unwrap().insert(param.clone(), value);
        }
        let result = self.run_block(&kaam.body);
        self.pop_scope();
        self.call_depth -= 1;
        match result? {
            Flow::Normal => Ok(Value::Khaali),
            Flow::Wapas(value, _) => Ok(value),
            Flow::Ruk(rline) => Err(BoliError::line(
                rline,
                "'ruk' sirf loop ke andar use hota hai",
            )),
            Flow::Agla(aline) => Err(BoliError::line(
                aline,
                "'agla' sirf loop ke andar use hota hai",
            )),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, BoliError> {
        // Bahut gehra nested expression/AST se stack overflow nahi hona chahiye
        self.eval_depth += 1;
        if self.eval_depth > 256 {
            self.eval_depth -= 1;
            return Err(BoliError::line(
                expr.line_for_error(),
                "expression bahut zyada gehra hai (max 256)",
            ));
        }
        let result = self.eval_expr_inner(expr);
        self.eval_depth -= 1;
        result
    }

    fn eval_expr_inner(&mut self, expr: &Expr) -> Result<Value, BoliError> {
        match expr {
            Expr::Number(n, _) => Ok(Value::Number(*n)),
            Expr::Text(s, line) => checked_text(s.clone(), *line, self.max_string_bytes),
            Expr::Bool(b, _) => Ok(Value::Bool(*b)),
            Expr::Khaali(_) => Ok(Value::Khaali),
            Expr::Var(name, line) => {
                if let Some(v) = self.get(name) {
                    return Ok(v);
                }
                match self.kaam.get(name) {
                    Some(kaam) if kaam.params.is_empty() => {
                        return self.call_kaam(name, Vec::new(), *line);
                    }
                    Some(kaam) => {
                        return Err(BoliError::line(
                            *line,
                            format!(
                                "kaam '{}' ko {} cheezein chahiye, seedha call karo",
                                name,
                                kaam.params.len()
                            ),
                        ));
                    }
                    None => {}
                }
                Err(BoliError::line(*line, format!("naam '{}' nahi mila", name)))
            }
            Expr::Arith(op, left, right, line) => {
                let left_v = self.eval_expr(left)?;
                let right_v = self.eval_expr(right)?;
                eval_arith(
                    *op,
                    &left_v,
                    &right_v,
                    *line,
                    self.max_string_bytes,
                    self.max_list_items,
                )
            }
            Expr::Call(name, args, line) => self.call_kaam(name, args.clone(), *line),
            Expr::List(items, line) => {
                if items.len() > self.max_list_items {
                    return Err(BoliError::line(
                        *line,
                        format!(
                            "list bahut badi ho gayi (max {} items)",
                            self.max_list_items
                        ),
                    ));
                }
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
                    return Err(BoliError::line(
                        *line,
                        "hissa 1 se shuru hone wali integer ginti honi chahiye",
                    ));
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
                    Value::Text(s) => match s.chars().nth(i - 1) {
                        Some(c) => Ok(Value::Text(c.to_string())),
                        None => Err(BoliError::line(
                            *line,
                            format!(
                                "text me sirf {} unicode chars hain, {} maanga",
                                s.chars().count(),
                                i
                            ),
                        )),
                    },
                    other => Err(BoliError::line(
                        *line,
                        format!(
                            "hissa sirf list ya text pe chal sakta hai, {} pe nahi",
                            other.type_naam()
                        ),
                    )),
                }
            }
        }
    }

    fn eval_cond(&mut self, cond: &Cond) -> Result<bool, BoliError> {
        self.cond_depth += 1;
        if self.cond_depth > self.max_condition_depth {
            self.cond_depth -= 1;
            return Err(BoliError::line(
                cond_line(cond),
                format!(
                    "condition bahut zyada gehra hai (max {})",
                    self.max_condition_depth
                ),
            ));
        }
        let result = self.eval_cond_inner(cond);
        self.cond_depth -= 1;
        result
    }

    fn eval_cond_inner(&mut self, cond: &Cond) -> Result<bool, BoliError> {
        match cond {
            Cond::Cmp {
                left,
                op,
                right,
                line,
            } => {
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

trait F64ToI64Checked {
    fn to_i64_checked(self) -> Option<i64>;
}

impl F64ToI64Checked for f64 {
    fn to_i64_checked(self) -> Option<i64> {
        if self < i64::MIN as f64 || self >= (i64::MAX as f64) {
            None
        } else {
            Some(self as i64)
        }
    }
}

fn cond_line(cond: &Cond) -> usize {
    match cond {
        Cond::Cmp { line, .. } => *line,
        Cond::Aur(_, _, line) => *line,
        Cond::Ya(_, _, line) => *line,
    }
}

fn checked_text(s: String, line: usize, max_string_bytes: usize) -> Result<Value, BoliError> {
    if s.len() > max_string_bytes {
        return Err(BoliError::line(
            line,
            format!("text bahut bada ho gaya (max {} bytes)", max_string_bytes),
        ));
    }
    Ok(Value::Text(s))
}

fn as_number(value: &Value, line: usize, kahan: &str) -> Result<f64, BoliError> {
    match value {
        Value::Number(n) => Ok(*n),
        other => Err(BoliError::line(
            line,
            format!("{} number chahiye, par {} mila", kahan, other.type_naam()),
        )),
    }
}

fn eval_arith(
    op: ArithOp,
    left: &Value,
    right: &Value,
    line: usize,
    max_string_bytes: usize,
    max_list_items: usize,
) -> Result<Value, BoliError> {
    if op == ArithOp::Jodo
        && let (Value::List(a), Value::List(b)) = (left, right)
    {
        let mut items = a.clone();
        items.extend(b.iter().cloned());
        if items.len() > max_list_items {
            return Err(BoliError::line(
                line,
                format!("list bahut badi ho gayi (max {} items)", max_list_items),
            ));
        }
        return Ok(Value::List(items));
    }
    if op == ArithOp::Jodo {
        if let (Value::Text(a), Value::Text(b)) = (left, right) {
            return checked_text(format!("{}{}", a, b), line, max_string_bytes);
        }
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
    if !result.is_finite() {
        return Err(BoliError::line(
            line,
            format!(
                "'{}' ka result finite nahi raha, number bahut bada hai",
                op.naam()
            ),
        ));
    }
    Ok(Value::Number(result))
}

fn compare(op: CmpOp, left: &Value, right: &Value, line: usize) -> Result<bool, BoliError> {
    if op == CmpOp::Barabar {
        return Ok(values_equal(left, right));
    }
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

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::Text(a), Value::Text(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Khaali, Value::Khaali) => true,
        (Value::List(a), Value::List(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        _ => false,
    }
}
