// BOLI Bytecode VM (experimental — --vm / BOLI_VM=1 se use hota hai)
// AST tree-walker ke saath same semantics; stack machine + name-based locals.
// Line numbers ek side-table (lines) me rakhta hai — ops me line embedded nahi.

use std::collections::HashMap;

use crate::ast::{ArithOp, BoliError, CmpOp, Cond, Expr, Stmt};
use crate::builtins;
use crate::interpreter::value::{Value, value_size};
use crate::limits::{self, env_limit};
use crate::scope::Scope;

#[derive(Debug, Clone)]
enum Op {
    Const(u16),
    LoadName(u16),
    StoreName(u16),
    Out,
    Arith(ArithOp),
    Compare(CmpOp),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    Jump(usize),
    PopScope,
    MakeList(u16),
    MakeMap(u16),
    Index,
    StoreIndex, // target, index, value on stack -> target updated in current scope? needs name
    Call(u16, u16), // name idx, argc
    Return,
    PopValue,
    Pucho(u16, u16), // name idx, prompt idx
    MakeRange,
    GintiNext,
    JumpIfNoLeft(usize),
}

pub(crate) struct KaamDef {
    params: Vec<String>,
    code: Vec<Op>,
    lines: Vec<usize>,
}

const LEFT: &str = "\u{0}left";

// ---------------- Compiler ----------------
struct Compiler {
    constants: Vec<Value>,
    names: Vec<String>,
    kaams: HashMap<String, KaamDef>,
    loop_stack: Vec<(usize, Vec<usize>, bool)>, // (loop_start_addr, break sites, is_range)
    loaded: HashSet<std::path::PathBuf>,
    source_dir: Option<std::path::PathBuf>,
    max_string_bytes: usize,
}

use std::collections::HashSet;

impl Compiler {
    fn new() -> Self {
        let max_string_bytes =
            env_limit("BOLI_MAX_STRING_BYTES", limits::DEFAULT_MAX_STRING_BYTES) as usize;
        Compiler {
            constants: Vec::new(),
            names: Vec::new(),
            kaams: HashMap::new(),
            loop_stack: Vec::new(),
            loaded: HashSet::new(),
            source_dir: None,
            max_string_bytes,
        }
    }

    fn const_value(&mut self, v: Value) -> u16 {
        let idx = self.constants.len();
        self.constants.push(v);
        idx as u16
    }

    fn name(&mut self, n: &str) -> u16 {
        if let Some(idx) = self.names.iter().position(|x| x == n) {
            return idx as u16;
        }
        self.names.push(n.to_string());
        (self.names.len() - 1) as u16
    }

    fn with_dir(source_dir: Option<std::path::PathBuf>) -> Self {
        let mut c = Self::new();
        c.source_dir = source_dir;
        c
    }

    fn compile_expr(
        &mut self,
        expr: &Expr,
        code: &mut Vec<Op>,
        lines: &mut Vec<usize>,
    ) -> Result<(), BoliError> {
        match expr {
            Expr::Number(n, l) => {
                code.push(Op::Const(self.const_value(Value::Number(*n))));
                lines.push(*l);
            }
            Expr::Text(s, l) => {
                if s.len() > self.max_string_bytes {
                    return Err(BoliError::line(
                        *l,
                        format!("text bahut bada hai (max {} bytes)", self.max_string_bytes),
                    ));
                }
                code.push(Op::Const(self.const_value(Value::Text(s.clone()))));
                lines.push(*l);
            }
            Expr::Bool(b, l) => {
                code.push(Op::Const(self.const_value(Value::Bool(*b))));
                lines.push(*l);
            }
            Expr::Khaali(l) => {
                code.push(Op::Const(self.const_value(Value::Khaali)));
                lines.push(*l);
            }
            Expr::Var(name, l) => {
                code.push(Op::LoadName(self.name(name)));
                lines.push(*l);
            }
            Expr::Arith(op, left, right, l) => {
                self.compile_expr(left, code, lines)?;
                self.compile_expr(right, code, lines)?;
                code.push(Op::Arith(*op));
                lines.push(*l);
            }
            Expr::Call(name, args, l) => {
                for a in args {
                    self.compile_expr(a, code, lines)?;
                }
                let idx = self.name(name);
                code.push(Op::Call(idx, args.len() as u16));
                lines.push(*l);
            }
            Expr::List(items, l) => {
                for e in items {
                    self.compile_expr(e, code, lines)?;
                }
                code.push(Op::MakeList(items.len() as u16));
                lines.push(*l);
            }
            Expr::Index(target, index, l) => {
                self.compile_expr(target, code, lines)?;
                self.compile_expr(index, code, lines)?;
                code.push(Op::Index);
                lines.push(*l);
            }
        }
        Ok(())
    }

    fn compile_cond(
        &mut self,
        cond: &Cond,
        code: &mut Vec<Op>,
        lines: &mut Vec<usize>,
    ) -> Result<(), BoliError> {
        match cond {
            Cond::Cmp {
                left, op, right, ..
            } => {
                self.compile_expr(left, code, lines)?;
                self.compile_expr(right, code, lines)?;
                code.push(Op::Compare(*op));
                lines.push(0);
            }
            Cond::Aur(left, right, _) => {
                self.compile_cond(left, code, lines)?;
                let jump_false = code.len();
                code.push(Op::JumpIfFalse(0));
                lines.push(0);
                self.compile_cond(right, code, lines)?;
                let jump_end = code.len();
                code.push(Op::Jump(0));
                lines.push(0);
                let false_start = code.len();
                code.push(Op::Const(self.const_value(Value::Bool(false))));
                lines.push(0);
                let end = code.len();
                code[jump_false] = Op::JumpIfFalse(false_start);
                code[jump_end] = Op::Jump(end);
            }
            Cond::Ya(left, right, _) => {
                self.compile_cond(left, code, lines)?;
                let jump_true = code.len();
                code.push(Op::JumpIfTrue(0));
                lines.push(0);
                self.compile_cond(right, code, lines)?;
                let jump_end = code.len();
                code.push(Op::Jump(0));
                lines.push(0);
                let true_start = code.len();
                code.push(Op::Const(self.const_value(Value::Bool(true))));
                lines.push(0);
                let end = code.len();
                code[jump_true] = Op::JumpIfTrue(true_start);
                code[jump_end] = Op::Jump(end);
            }
        }
        Ok(())
    }

    fn compile_block(
        &mut self,
        stmts: &[Stmt],
        code: &mut Vec<Op>,
        lines: &mut Vec<usize>,
    ) -> Result<(), BoliError> {
        for s in stmts {
            self.compile_stmt(s, code, lines)?;
        }
        Ok(())
    }

    fn compile_stmt(
        &mut self,
        stmt: &Stmt,
        code: &mut Vec<Op>,
        lines: &mut Vec<usize>,
    ) -> Result<(), BoliError> {
        match stmt {
            Stmt::Likho(expr, line) => {
                self.compile_expr(expr, code, lines)?;
                code.push(Op::Out);
                lines.push(*line);
            }
            Stmt::Rakho { name, value, line } => {
                self.compile_expr(value, code, lines)?;
                code.push(Op::StoreName(self.name(name)));
                lines.push(*line);
            }
            Stmt::MapRakho { name, pairs, line } => {
                for (k, v) in pairs {
                    self.compile_expr(k, code, lines)?;
                    self.compile_expr(v, code, lines)?;
                }
                code.push(Op::MakeMap(pairs.len() as u16));
                lines.push(*line);
                code.push(Op::StoreName(self.name(name)));
                lines.push(*line);
            }
            Stmt::Pucho { name, prompt, line } => {
                let prompt_idx = self.const_value(Value::Text(prompt.clone()));
                code.push(Op::Pucho(self.name(name), prompt_idx));
                lines.push(*line);
            }
            Stmt::ArithAssign {
                target,
                op,
                left,
                right,
                line,
            } => {
                self.compile_expr(left, code, lines)?;
                self.compile_expr(right, code, lines)?;
                code.push(Op::Arith(*op));
                lines.push(*line);
                code.push(Op::StoreName(self.name(target)));
                lines.push(*line);
            }
            Stmt::Agar {
                cond,
                then_block,
                else_block,
                ..
            } => {
                self.compile_cond(cond, code, lines)?;
                let jump_else = code.len();
                code.push(Op::JumpIfFalse(0));
                lines.push(0);
                self.compile_block(then_block, code, lines)?;
                let jump_end = code.len();
                code.push(Op::Jump(0));
                lines.push(0);
                let else_start = code.len();
                if let Some(b) = else_block {
                    self.compile_block(b, code, lines)?;
                }
                let end = code.len();
                code[jump_else] = Op::JumpIfFalse(else_start);
                code[jump_end] = Op::Jump(end);
            }
            Stmt::WhileLoop { cond, body, .. } => {
                let loop_start = code.len();
                self.compile_cond(cond, code, lines)?;
                let jump_end = code.len();
                code.push(Op::JumpIfFalse(0));
                lines.push(0);
                self.loop_stack.push((loop_start, Vec::new(), false));
                self.compile_block(body, code, lines)?;
                code.push(Op::Jump(loop_start));
                lines.push(0);
                let end = code.len();
                code[jump_end] = Op::JumpIfFalse(end);
                if let Some((_, breaks, _)) = self.loop_stack.pop() {
                    for b in breaks {
                        code[b] = Op::Jump(end);
                    }
                }
            }
            Stmt::RangeLoop {
                from,
                to,
                body,
                line,
            } => {
                self.compile_expr(from, code, lines)?;
                self.compile_expr(to, code, lines)?;
                // ginti scope: MakeRange pops to/from and creates scope
                code.push(Op::MakeRange);
                lines.push(*line);
                let loop_start = code.len();
                let jump_end = code.len();
                code.push(Op::JumpIfNoLeft(0));
                lines.push(*line);
                self.loop_stack.push((loop_start, Vec::new(), true));
                self.compile_block(body, code, lines)?;
                code.push(Op::GintiNext);
                lines.push(*line);
                code.push(Op::Jump(loop_start));
                lines.push(*line);
                let end = code.len();
                code[jump_end] = Op::JumpIfNoLeft(end);
                code.push(Op::PopScope);
                lines.push(*line);
                if let Some((_, breaks, _)) = self.loop_stack.pop() {
                    for b in breaks {
                        code[b] = Op::Jump(end);
                    }
                }
            }
            Stmt::KaamDef {
                name,
                params,
                body,
                line,
            } => {
                let mut body_code = Vec::new();
                let mut body_lines = Vec::new();
                let saved_stack = std::mem::take(&mut self.loop_stack);
                self.compile_block(body, &mut body_code, &mut body_lines)?;
                self.loop_stack = saved_stack;
                let khaali = self.const_value(Value::Khaali);
                body_code.push(Op::Const(khaali));
                body_lines.push(*line);
                body_code.push(Op::Return);
                body_lines.push(*line);
                if self.kaams.contains_key(name) {
                    return Err(BoliError::line(
                        *line,
                        format!("kaam '{}' pehle se bana hua hai", name),
                    ));
                }
                self.kaams.insert(
                    name.clone(),
                    KaamDef {
                        params: params.clone(),
                        code: body_code,
                        lines: body_lines,
                    },
                );
            }
            Stmt::KaamCall { name, args, line } => {
                for a in args {
                    self.compile_expr(a, code, lines)?;
                }
                let idx = self.name(name);
                code.push(Op::Call(idx, args.len() as u16));
                lines.push(*line);
                code.push(Op::PopValue);
                lines.push(*line);
            }
            Stmt::Wapas { value, line } => {
                match value {
                    Some(e) => self.compile_expr(e, code, lines)?,
                    None => {
                        let k = self.const_value(Value::Khaali);
                        code.push(Op::Const(k));
                        lines.push(*line);
                    }
                }
                code.push(Op::Return);
                lines.push(*line);
            }
            Stmt::Ruk(line) => match self.loop_stack.last_mut() {
                Some((_, breaks, _)) => {
                    let at = code.len();
                    code.push(Op::Jump(0));
                    lines.push(*line);
                    breaks.push(at);
                }
                None => {
                    return Err(BoliError::line(
                        *line,
                        "'ruk' sirf loop ke andar use hota hai",
                    ));
                }
            },
            Stmt::Agla(line) => match self.loop_stack.last() {
                Some((target, _, is_range)) => {
                    let t = *target;
                    let range = *is_range;
                    if range {
                        code.push(Op::GintiNext);
                        lines.push(*line);
                    }
                    code.push(Op::Jump(t));
                    lines.push(*line);
                }
                None => {
                    return Err(BoliError::line(
                        *line,
                        "'agla' sirf loop ke andar use hota hai",
                    ));
                }
            },
            Stmt::ListAssign {
                target,
                index,
                value,
                line,
            } => {
                self.compile_expr(index, code, lines)?;
                self.compile_expr(value, code, lines)?;
                code.push(Op::LoadName(self.name(target)));
                lines.push(*line);
                code.push(Op::StoreIndex);
                lines.push(*line);
                code.push(Op::StoreName(self.name(target)));
                lines.push(*line);
            }
            Stmt::Laao { path, line } => {
                // compile-time import: doosri .hl file ko inline compile karo
                let resolve_base = self.source_dir.clone().unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
                });
                let joined = if std::path::Path::new(path).is_absolute() {
                    std::path::PathBuf::from(path)
                } else {
                    resolve_base.join(path)
                };
                let canonical = std::fs::canonicalize(&joined).map_err(|e| {
                    BoliError::line(*line, format!("file '{}' nahi mili: {}", path, e))
                })?;
                if !self.loaded.insert(canonical.clone()) {
                    return Ok(()); // pehle hi include ho chuka hai
                }
                let max_source =
                    limits::env_limit("BOLI_MAX_SOURCE_BYTES", limits::DEFAULT_MAX_SOURCE_BYTES);
                let meta_len = std::fs::metadata(&canonical).map(|m| m.len()).unwrap_or(0);
                if meta_len > max_source {
                    return Err(BoliError::line(
                        *line,
                        format!(
                            "file '{}' bahut badi hai ({} bytes, max {})",
                            path, meta_len, max_source
                        ),
                    ));
                }
                let source = std::fs::read_to_string(&canonical).map_err(|e| {
                    BoliError::line(*line, format!("file '{}' padh nahi paye: {}", path, e))
                })?;
                let program = crate::parser::parse_source(&source).map_err(|e| {
                    BoliError::line(*line, format!("file '{}' me error: {}", path, e))
                })?;
                self.compile_block(&program, code, lines)?;
            }
        }
        Ok(())
    }
}

pub struct Vm {
    source_dir: Option<std::path::PathBuf>,
    scopes: Vec<Scope>,
    total_bytes: u64,
    max_total_bytes: u64,
    steps: u64,
    max_steps: u64,
    call_depth: usize,
    max_call_depth: usize,
    max_string_bytes: usize,
    max_list_items: usize,
    constants: Vec<Value>,
    names: Vec<String>,
    kaams: HashMap<String, KaamDef>,
    trace: Vec<(String, usize)>,
    captured: Option<Vec<String>>,
    stack: Vec<Value>,
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

impl Vm {
    pub fn new() -> Self {
        let max_steps = env_limit("BOLI_MAX_STEPS", limits::DEFAULT_MAX_STEPS);
        let max_call_depth =
            env_limit("BOLI_MAX_CALL_DEPTH", limits::DEFAULT_MAX_CALL_DEPTH) as usize;
        let max_string_bytes =
            env_limit("BOLI_MAX_STRING_BYTES", limits::DEFAULT_MAX_STRING_BYTES) as usize;
        let max_list_items =
            env_limit("BOLI_MAX_LIST_ITEMS", limits::DEFAULT_MAX_LIST_ITEMS) as usize;
        let max_total_bytes = env_limit("BOLI_MAX_TOTAL_BYTES", limits::DEFAULT_MAX_TOTAL_BYTES);
        Vm {
            source_dir: None,
            scopes: vec![Scope::new()],
            total_bytes: 0,
            max_total_bytes,
            steps: 0,
            max_steps,
            call_depth: 0,
            max_call_depth,
            max_string_bytes,
            max_list_items,
            constants: Vec::new(),
            names: Vec::new(),
            kaams: HashMap::new(),
            trace: Vec::new(),
            captured: None,
            stack: Vec::new(),
        }
    }

    pub fn collect_output(&mut self) {
        self.captured = Some(Vec::new());
    }

    pub fn take_output(&mut self) -> Vec<String> {
        self.captured.take().unwrap_or_default()
    }

    pub fn set_source_dir(&mut self, dir: std::path::PathBuf) {
        self.source_dir = Some(dir);
    }

    pub fn run_program(&mut self, program: &[Stmt]) -> Result<(), BoliError> {
        // compile phase
        // compile phase
        let mut compiler = Compiler::new();
        if let Some(dir) = &self.source_dir {
            compiler = Compiler::with_dir(Some(dir.clone()));
        }
        let mut code = Vec::new();
        let mut lines = Vec::new();
        compiler.compile_block(program, &mut code, &mut lines)?;
        // move compiled artifacts into VM
        self.constants = compiler.constants;
        self.names = compiler.names;
        self.kaams = compiler.kaams;
        self.exec(&code, &lines)?;
        Ok(())
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            let popped = self.scopes.pop().expect("global hamesha rahega");
            self.total_bytes = self.total_bytes.saturating_sub(popped.bytes);
        }
    }

    fn get_local(&self, name: &str) -> Option<Value> {
        for s in self.scopes.iter().rev() {
            if let Some(v) = s.map.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    fn set_local(&mut self, name: &str, value: Value, line: usize) -> Result<(), BoliError> {
        let size = value_size(&value);
        let scope = self.scopes.last_mut().expect("global hamesha rahega");
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
}

impl Vm {
    fn step(&mut self, line: usize) -> Result<(), BoliError> {
        self.steps += 1;
        if self.steps > self.max_steps {
            return Err(BoliError::line(
                line,
                format!(
                    "program bahut zyada steps chal gaya (max {}), infinite loop to nahi?",
                    self.max_steps
                ),
            ));
        }
        Ok(())
    }

    fn exec(&mut self, code: &[Op], lines: &[usize]) -> Result<ExecResult, BoliError> {
        let mut ip = 0usize;
        while ip < code.len() {
            let line = lines.get(ip).copied().unwrap_or(0);
            self.step(line)?;
            match &code[ip] {
                Op::Const(idx) => {
                    self.stack.push(self.constants[*idx as usize].clone());
                }
                Op::LoadName(idx) => {
                    let name = self.names[*idx as usize].clone();
                    match self.get_local(&name) {
                        Some(v) => self.stack.push(v),
                        None => {
                            // zero-arg kaam fallback (interpreter jaisa)
                            if self.kaams.get(&name).is_some_and(|k| k.params.is_empty()) {
                                let v = self.call_kaam(&name, Vec::new(), line)?;
                                self.stack.push(v);
                            } else {
                                return Err(BoliError::line(
                                    line,
                                    format!("naam '{}' nahi mila", name),
                                ));
                            }
                        }
                    }
                }
                Op::StoreName(idx) => {
                    let name = self.names[*idx as usize].clone();
                    let v = self.stack.pop().expect("stack underflow");
                    self.set_local(&name, v, line)?;
                }
                Op::Out => {
                    let v = self.stack.pop().expect("stack underflow");
                    let text = v.dikhane_laayak();
                    if let Some(cap) = &mut self.captured {
                        cap.push(text);
                    } else if writeln_stdout(&text).is_err() {
                        std::process::exit(0); // closed stdout -> clean exit
                    }
                }
                Op::Arith(op) => {
                    let right = self.stack.pop().expect("stack underflow");
                    let left = self.stack.pop().expect("stack underflow");
                    let v = crate::interpreter::eval::eval_arith(
                        *op,
                        &left,
                        &right,
                        line,
                        self.max_string_bytes,
                        self.max_list_items,
                    )?;
                    self.stack.push(v);
                }
                Op::Compare(op) => {
                    let right = self.stack.pop().expect("stack underflow");
                    let left = self.stack.pop().expect("stack underflow");
                    let v = crate::interpreter::eval::compare(*op, &left, &right, line)?;
                    self.stack.push(Value::Bool(v));
                }
                Op::JumpIfFalse(addr) => {
                    let c = self.stack.pop().expect("stack underflow");
                    if !truthy(&c) {
                        ip = *addr;
                        continue;
                    }
                }
                Op::JumpIfTrue(addr) => {
                    let c = self.stack.pop().expect("stack underflow");
                    if truthy(&c) {
                        ip = *addr;
                        continue;
                    }
                }
                Op::Jump(addr) => {
                    ip = *addr;
                    continue;
                }
                Op::PopScope => self.pop_scope(),
                Op::MakeList(n) => {
                    let n = *n as usize;
                    let mut items = Vec::with_capacity(n);
                    for _ in 0..n {
                        items.push(self.stack.pop().expect("stack underflow"));
                    }
                    items.reverse();
                    if items.len() > self.max_list_items {
                        return Err(BoliError::line(
                            line,
                            format!(
                                "list bahut badi ho gayi (max {} items)",
                                self.max_list_items
                            ),
                        ));
                    }
                    self.stack.push(Value::List(items));
                }
                Op::MakeMap(n) => {
                    let n = *n as usize;
                    let mut pairs = Vec::with_capacity(n);
                    for _ in 0..n {
                        let value = self.stack.pop().expect("stack underflow");
                        let key = self.stack.pop().expect("stack underflow");
                        pairs.push((key, value));
                    }
                    pairs.reverse();
                    let mut map = std::collections::BTreeMap::new();
                    for (k, v) in pairs {
                        let key = match k {
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
                        map.insert(key, v);
                    }
                    self.stack.push(Value::Map(map));
                }
                Op::Index => {
                    let index = self.stack.pop().expect("stack underflow");
                    let target = self.stack.pop().expect("stack underflow");
                    let v = crate::interpreter::eval::index_get_pub(&target, &index, line)?;
                    self.stack.push(v);
                }
                Op::StoreIndex => {
                    // push order: index, value, target -> pops: target, value, index
                    let target = self.stack.pop().expect("stack underflow");
                    let value = self.stack.pop().expect("stack underflow");
                    let index = self.stack.pop().expect("stack underflow");
                    let updated =
                        crate::interpreter::eval::index_set_pub(target, &index, value, line)?;
                    self.stack.push(updated);
                }
                Op::Call(name_idx, argc) => {
                    let name = self.names[*name_idx as usize].clone();
                    let argc = *argc as usize;
                    let mut args = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        args.push(self.stack.pop().expect("stack underflow"));
                    }
                    args.reverse();
                    let result = self.call_kaam(&name, args, line)?;
                    self.stack.push(result);
                }
                Op::Return => {
                    let v = self.stack.pop().expect("stack underflow");
                    return Ok(ExecResult::Return(v));
                }
                Op::PopValue => {
                    self.stack.pop().expect("stack underflow");
                }
                Op::Pucho(name_idx, prompt_idx) => {
                    let name = self.names[*name_idx as usize].clone();
                    let prompt = match &self.constants[*prompt_idx as usize] {
                        Value::Text(t) => t.clone(),
                        _ => String::new(),
                    };
                    let v = self.pucho(&prompt, line)?;
                    self.set_local(&name, v, line)?;
                }
                Op::MakeRange => {
                    let to = self.stack.pop().expect("stack underflow");
                    let from = self.stack.pop().expect("stack underflow");
                    let from_n = as_number_pub(&from, "se ke pehle", line)?;
                    let to_n = as_number_pub(&to, "se ke baad", line)?;
                    self.push_scope();
                    let count = range_count_pub(from_n, to_n, line)?;
                    self.scopes
                        .last_mut()
                        .expect("scope")
                        .map
                        .insert("ginti".to_string(), Value::Number(from_n));
                    self.scopes
                        .last_mut()
                        .expect("scope")
                        .map
                        .insert(LEFT.to_string(), Value::Number(count));
                }
                Op::GintiNext => {
                    let ginti = self.get_local("ginti").unwrap_or(Value::Number(1.0));
                    let left = self.get_local(LEFT).unwrap_or(Value::Number(0.0));
                    if let (Value::Number(g), Value::Number(l)) = (ginti, left) {
                        self.set_local("ginti", Value::Number(g + 1.0), line)?;
                        self.set_local(LEFT, Value::Number(l - 1.0), line)?;
                    }
                }
                Op::JumpIfNoLeft(addr) => {
                    let left = self.get_local(LEFT).unwrap_or(Value::Number(0.0));
                    match left {
                        Value::Number(n) if n > 0.0 => {}
                        _ => {
                            ip = *addr;
                            continue;
                        }
                    }
                }
            }
            ip += 1;
        }
        Ok(ExecResult::Normal)
    }

    fn call_kaam(&mut self, name: &str, args: Vec<Value>, line: usize) -> Result<Value, BoliError> {
        let kaam = self
            .kaams
            .get(name)
            .map(|k| (k.params.clone(), k.code.clone(), k.lines.clone()));
        let Some((params, code, klines)) = kaam else {
            // user-defined nahi hai to built-in try karo
            if builtins::is_builtin(name) {
                return builtins::call_builtin(name, args, line);
            }
            return Err(BoliError::line(line, format!("kaam '{}' nahi mila", name)));
        };
        if params.len() != args.len() {
            return Err(BoliError::line(
                line,
                format!(
                    "kaam '{}' ko {} cheezein chahiye, {} di gai",
                    name,
                    params.len(),
                    args.len()
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
        for (param, value) in params.into_iter().zip(args) {
            self.set_local(&param, value, line)?;
        }
        let result = self.exec(&code, &klines);
        let snapshot = self.trace.clone();
        self.trace.pop();
        self.pop_scope();
        self.call_depth -= 1;
        let result = result.map_err(|mut e| {
            if e.trace.is_empty() {
                e.trace = snapshot;
            }
            e
        });
        match result? {
            ExecResult::Return(v) => Ok(v),
            ExecResult::Normal => Ok(Value::Khaali),
        }
    }

    fn pucho(&mut self, prompt: &str, line: usize) -> Result<Value, BoliError> {
        if self.captured.is_some() {
            return Ok(Value::Khaali);
        }
        let _ = writeln_stdout(prompt);
        let input = match limits::read_input_line(self.max_string_bytes as u64) {
            Ok(v) => v,
            Err(_) => return Err(BoliError::line(line, "input nahi mil raha")),
        };
        let line_in = match input {
            limits::InputLine::Eof => String::new(),
            limits::InputLine::TooLarge => {
                return Err(BoliError::line(
                    line,
                    format!(
                        "input text bahut bada hai (max {} bytes)",
                        self.max_string_bytes
                    ),
                ));
            }
            limits::InputLine::Line(s) => s,
        };
        let trimmed = line_in.trim();
        Ok(if trimmed.is_empty() {
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
        })
    }
}

enum ExecResult {
    Normal,
    Return(Value),
}

fn writeln_stdout(text: &str) -> std::io::Result<()> {
    use std::io::Write;
    writeln!(std::io::stdout(), "{}", text)
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        _ => true, // interpreter semantics: non-Bool truthy (compare se hi cond aati hai)
    }
}

fn as_number_pub(v: &Value, kahan: &str, line: usize) -> Result<f64, BoliError> {
    match v {
        Value::Number(n) => Ok(*n),
        other => Err(BoliError::line(
            line,
            format!("{} number chahiye, par {} mila", kahan, other.type_naam()),
        )),
    }
}

fn range_count_pub(from_n: f64, to_n: f64, line: usize) -> Result<f64, BoliError> {
    if to_n >= from_n {
        let distance = (to_n - from_n).floor();
        if !distance.is_finite() {
            return Err(BoliError::line(line, "range bahut badi hai"));
        }
        match to_i64_checked_pub(distance) {
            Some(d) => Ok((d + 1) as f64),
            None => Err(BoliError::line(
                line,
                "range itni badi hai ki chal nahi sakti",
            )),
        }
    } else {
        Ok(0.0)
    }
}

fn to_i64_checked_pub(n: f64) -> Option<i64> {
    if n < i64::MIN as f64 || n >= (i64::MAX as f64) {
        None
    } else {
        Some(n as i64)
    }
}
