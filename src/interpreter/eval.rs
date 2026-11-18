// BOLI expression/condition evaluation (Environment ke methods)
// eval_expr / eval_cond aur unke free helper functions

use super::Environment;
use super::value::Value;
use crate::ast::{ArithOp, BoliError, CmpOp, Cond, Expr};

impl Environment {
    pub(crate) fn eval_expr(&mut self, expr: &Expr) -> Result<Value, BoliError> {
        // Bahut gehra nested expression/AST se stack overflow nahi hona chahiye
        self.eval_depth += 1;
        if self.eval_depth > self.max_eval_depth {
            self.eval_depth -= 1;
            return Err(BoliError::line(
                expr.line_for_error(),
                format!(
                    "expression bahut zyada gehra hai (max {})",
                    self.max_eval_depth
                ),
            ));
        }
        let result = self.eval_expr_inner(expr);
        self.eval_depth -= 1;
        result
    }

    pub(crate) fn eval_expr_inner(&mut self, expr: &Expr) -> Result<Value, BoliError> {
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

    pub(crate) fn eval_cond(&mut self, cond: &Cond) -> Result<bool, BoliError> {
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

    pub(crate) fn eval_cond_inner(&mut self, cond: &Cond) -> Result<bool, BoliError> {
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

pub(crate) trait F64ToI64Checked {
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

pub(crate) fn cond_line(cond: &Cond) -> usize {
    match cond {
        Cond::Cmp { line, .. } => *line,
        Cond::Aur(_, _, line) => *line,
        Cond::Ya(_, _, line) => *line,
    }
}

pub(crate) fn checked_text(
    s: String,
    line: usize,
    max_string_bytes: usize,
) -> Result<Value, BoliError> {
    if s.len() > max_string_bytes {
        return Err(BoliError::line(
            line,
            format!("text bahut bada ho gaya (max {} bytes)", max_string_bytes),
        ));
    }
    Ok(Value::Text(s))
}

pub(crate) fn as_number(value: &Value, line: usize, kahan: &str) -> Result<f64, BoliError> {
    match value {
        Value::Number(n) => Ok(*n),
        other => Err(BoliError::line(
            line,
            format!("{} number chahiye, par {} mila", kahan, other.type_naam()),
        )),
    }
}

pub(crate) fn eval_arith(
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

pub(crate) fn compare(
    op: CmpOp,
    left: &Value,
    right: &Value,
    line: usize,
) -> Result<bool, BoliError> {
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

pub(crate) fn values_equal(left: &Value, right: &Value) -> bool {
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
