// BOLI ke values (tree-walking interpreter)
// Live bytes budget ke liye value_size bhi yahin hai

use std::collections::BTreeMap;

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
            Value::Map(m) => m
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v.dikhane_laayak()))
                .collect::<Vec<_>>()
                .join(", "),
        }
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

// value ka approximate heap size (live bytes budget ke liye)
pub(crate) fn value_size(value: &Value) -> u64 {
    match value {
        Value::Text(t) => t.len() as u64 + 24,
        Value::Number(_) => 16,
        Value::Bool(_) => 8,
        Value::Khaali => 1,
        Value::List(items) => {
            24 + items.len() as u64 * 40 + items.iter().map(value_size).sum::<u64>()
        }
        Value::Map(m) => {
            24 + m.len() as u64 * 40
                + m.iter()
                    .map(|(k, v)| k.len() as u64 + 16 + value_size(v))
                    .sum::<u64>()
        }
    }
}
