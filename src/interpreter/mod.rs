// BOLI ka Interpreter - AST ko chalakar output deta hai
// Environment scopes ki list hai: andar ka kaam (function) khatam hote hi scope pop ho jata hai

mod eval;
mod value;

use eval::{F64ToI64Checked, as_number, eval_arith};
pub use value::Value;
use value::value_size;

use crate::ast::{BoliError, Expr, Stmt};
use crate::limits::{
    DEFAULT_MAX_CALL_DEPTH, DEFAULT_MAX_EVAL_DEPTH, DEFAULT_MAX_LIST_ITEMS, DEFAULT_MAX_STEPS,
    DEFAULT_MAX_STRING_BYTES, env_limit,
};
use std::collections::HashMap;
use std::io::{self, Write};

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

struct Scope {
    map: HashMap<String, Value>,
    bytes: u64,
}

impl Scope {
    fn new() -> Self {
        Scope {
            map: HashMap::new(),
            bytes: 0,
        }
    }

    fn total_bytes_add(&mut self, delta: i128) {
        if delta >= 0 {
            self.bytes += delta as u64;
        } else {
            self.bytes = self.bytes.saturating_sub((-delta) as u64);
        }
    }
}

pub struct Environment {
    scopes: Vec<Scope>,
    kaam: HashMap<String, Kaam>,
    steps: u64,
    max_steps: u64,
    max_call_depth: usize,
    max_string_bytes: usize,
    max_list_items: usize,
    max_eval_depth: usize,
    max_condition_depth: usize,
    max_total_bytes: u64,
    total_bytes: u64,
    call_depth: usize,
    eval_depth: usize,
    cond_depth: usize,
    // kaam call trace (outermost -> innermost order)
    trace: Vec<(String, usize)>,
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
        let max_total_bytes = env_limit(
            "BOLI_MAX_TOTAL_BYTES",
            crate::limits::DEFAULT_MAX_TOTAL_BYTES,
        );
        Environment {
            scopes: vec![Scope::new()],
            total_bytes: 0,
            max_total_bytes,
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
            trace: Vec::new(),
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

    pub fn with_total_budget_bytes(mut self, max_total_bytes: u64) -> Self {
        self.max_total_bytes = max_total_bytes;
        self
    }

    pub fn reset_steps(&mut self) {
        self.steps = 0;
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            let popped = self.scopes.pop().expect("global scope hamesha rahega");
            self.total_bytes = self.total_bytes.saturating_sub(popped.bytes);
        }
    }

    fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.map.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    // Live scope bytes ka hisaab: insert/replace par budget check
    fn set(&mut self, name: &str, value: Value, line: usize) -> Result<(), BoliError> {
        let size = value_size(&value);
        let scope = self.scopes.last_mut().expect("global scope hamesha rahega");
        let old_size = scope.map.get(name).map(value_size).unwrap_or(0);
        let new_total = self.total_bytes + size - old_size;
        if new_total > self.max_total_bytes {
            return Err(BoliError::line(
                line,
                format!(
                    "program ki memory budget khatam ho gayi (max {} bytes, BOLI_MAX_TOTAL_BYTES)",
                    self.max_total_bytes
                ),
            ));
        }
        scope.total_bytes_add(size as i128 - old_size as i128);
        scope.map.insert(name.to_string(), value);
        self.total_bytes = new_total;
        Ok(())
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
            Stmt::Rakho { name, value, line } => {
                let value = self.eval_expr(value)?;
                self.set(name, value, *line)?;
                Ok(Flow::Normal)
            }
            Stmt::MapRakho { name, pairs, line } => {
                let mut map = std::collections::BTreeMap::new();
                for (k_expr, v_expr) in pairs {
                    let key_v = self.eval_expr(k_expr)?;
                    let key = match key_v {
                        Value::Text(t) => t,
                        other => {
                            return Err(BoliError::line(
                                *line,
                                format!(
                                    "map ki chaabi text honi chahiye, {} nahi",
                                    other.type_naam()
                                ),
                            ));
                        }
                    };
                    let value = self.eval_expr(v_expr)?;
                    map.insert(key, value);
                }
                self.set(name, Value::Map(map), *line)?;
                Ok(Flow::Normal)
            }
            Stmt::Pucho { name, prompt, line } => {
                if self.captured.is_some() {
                    self.set(name, Value::Khaali, *line)?;
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
                self.set(name, value, *line)?;
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
                self.set(target, value, *line)?;
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
                        self.set("ginti", Value::Number(from_n + i as f64), *line)?;
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
        // Map assignment: mapa hissa "chaabi" rakho value
        {
            let target_v = self.get(target);
            if let Some(Value::Map(_)) = target_v {
                let key_v = self.eval_expr(index)?;
                let key = match key_v {
                    Value::Text(t) => t,
                    other => {
                        return Err(BoliError::line(
                            line,
                            format!(
                                "map ki chaabi text honi chahiye, {} nahi",
                                other.type_naam()
                            ),
                        ));
                    }
                };
                let new_value = self.eval_expr(value)?;
                let mut current = self.get(target).expect("abhi pata hai map hai");
                if let Value::Map(ref mut m) = current {
                    m.insert(key, new_value);
                }
                self.set(target, current, line)?;
                return Ok(());
            }
        }
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
        self.set(target, current, line)?;
        Ok(())
    }

    fn call_kaam(&mut self, name: &str, args: Vec<Expr>, line: usize) -> Result<Value, BoliError> {
        let arg_values: Vec<Value> = args
            .iter()
            .map(|a| self.eval_expr(a))
            .collect::<Result<_, _>>()?;
        let kaam = match self.kaam.get(name).cloned() {
            Some(k) => k,
            None => {
                // user-defined nahi hai to built-in try karo
                if crate::builtins::is_builtin(name) {
                    return crate::builtins::call_builtin(name, arg_values, line);
                }
                let hint = self
                    .nearest_name(name)
                    .map(|n| format!(" Shayad '{}' likhna tha?", n))
                    .unwrap_or_default();
                return Err(BoliError::line(
                    line,
                    format!("kaam '{}' nahi mila.{}", name, hint),
                ));
            }
        };
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
        self.trace.push((name.to_string(), line));
        self.push_scope();
        for (param, value) in kaam.params.iter().zip(arg_values) {
            self.set(param, value, line)?;
        }
        let result = self.run_block(&kaam.body);
        let trace_snapshot = self.trace.clone();
        self.trace.pop();
        self.pop_scope();
        self.call_depth -= 1;
        let result = result.map_err(|mut e| {
            if e.trace.is_empty() {
                e.trace = trace_snapshot.clone();
            }
            e
        });
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
}

// "Did you mean...?" - simple Damerau-Levenshtein over known names
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for (i, row) in dp.iter_mut().enumerate().take(m + 1) {
        row[0] = i;
    }
    for (j, cell) in dp[0].iter_mut().enumerate().take(n + 1) {
        *cell = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}

impl Environment {
    // sabhi known names (scope + kaam table) se sabse chhota edit distance wala return karo
    fn nearest_name(&self, wanted: &str) -> Option<String> {
        if wanted.is_empty() {
            return None;
        }
        let mut candidates: Vec<String> = self.kaam.keys().cloned().collect();
        for scope in &self.scopes {
            for k in scope.map.keys() {
                if !candidates.contains(k) {
                    candidates.push(k.clone());
                }
            }
        }
        for builtin in crate::builtins::BUILTINS {
            candidates.push((*builtin).to_string());
        }
        let mut best: Option<(usize, String)> = None;
        for c in candidates {
            let d = edit_distance(wanted, &c);
            if d <= wanted.chars().count().max(c.chars().count()) / 2 + 1
                && best.as_ref().map(|b| d < b.0).unwrap_or(true)
            {
                best = Some((d, c));
            }
        }
        best.map(|(_, c)| c)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
