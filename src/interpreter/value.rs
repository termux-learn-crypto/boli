// BOLI ke values (tree-walking interpreter)
// Live bytes budget ke liye value_size bhi yahin hai

use std::collections::BTreeMap;

use crate::ast::BoliError;

// BOLI ke values
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
    Bool(bool),                   // sahi / galat
    Khaali,                       // khaali (null)
    List(Vec<Value>),             // cheezein rakho 1 2 3
    Map(BTreeMap<String, Value>), // mapa rakho "a" 1 "b" 2
}

impl Value {
    pub fn dikhane_laayak(&self) -> String {
        enum Piece<'a> {
            Value(&'a Value),
            Text(&'a str),
            Owned(String),
        }

        let mut out = String::new();
        let mut stack = vec![Piece::Value(self)];
        while let Some(piece) = stack.pop() {
            match piece {
                Piece::Text(text) => out.push_str(text),
                Piece::Owned(text) => out.push_str(&text),
                Piece::Value(value) => match value {
                    Value::Text(text) => out.push_str(text),
                    Value::Number(n) => out.push_str(&number_text(*n)),
                    Value::Bool(b) => out.push_str(if *b { "sahi" } else { "galat" }),
                    Value::Khaali => out.push_str("khaali"),
                    Value::List(items) => {
                        for (from_end, item) in items.iter().rev().enumerate() {
                            if from_end > 0 {
                                stack.push(Piece::Text(", "));
                            }
                            stack.push(Piece::Value(item));
                        }
                    }
                    Value::Map(map) => {
                        for (from_end, (key, item)) in map.iter().rev().enumerate() {
                            if from_end > 0 {
                                stack.push(Piece::Text(", "));
                            }
                            stack.push(Piece::Value(item));
                            stack.push(Piece::Owned(format!("{}: ", key)));
                        }
                    }
                },
            }
        }
        out
    }

    pub(crate) fn type_naam(&self) -> &'static str {
        match self {
            Value::Text(_) => "text",
            Value::Number(_) => "number",
            Value::Bool(_) => "sahi/galat",
            Value::Khaali => "khaali",
            Value::List(_) => "list",
            Value::Map(_) => "map",
        }
    }
}

fn number_text(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 9.007199254740992e15 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

// value ka approximate heap size (live bytes budget ke liye), iterative
pub(crate) fn value_size(value: &Value) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![value];
    while let Some(value) = stack.pop() {
        match value {
            Value::Text(text) => {
                total = total.saturating_add(text.len() as u64).saturating_add(24);
            }
            Value::Number(_) => total = total.saturating_add(16),
            Value::Bool(_) => total = total.saturating_add(8),
            Value::Khaali => total = total.saturating_add(1),
            Value::List(items) => {
                total = total
                    .saturating_add(24)
                    .saturating_add((items.len() as u64).saturating_mul(40));
                for item in items {
                    stack.push(item);
                }
            }
            Value::Map(map) => {
                total = total
                    .saturating_add(24)
                    .saturating_add((map.len() as u64).saturating_mul(40));
                for (key, item) in map {
                    total = total.saturating_add(key.len() as u64).saturating_add(16);
                    stack.push(item);
                }
            }
        }
    }
    total
}

pub(crate) fn check_value_depth(
    value: &Value,
    max_depth: usize,
    line: usize,
) -> Result<(), BoliError> {
    if crate::limits::value_depth(value) > max_depth {
        return Err(BoliError::line(
            line,
            format!(
                "value bahut gehra hai (max {} levels, BOLI_MAX_VALUE_DEPTH)",
                max_depth
            ),
        ));
    }
    Ok(())
}
