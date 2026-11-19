// BOLI built-in kaam (standard library) - pure Rust, koi external dependency nahi
// User-defined kaam in names ko shadow kar sakta hai (user kaam ki priority zyada)

use crate::ast::BoliError;
use crate::interpreter::Value;

// Built-in kaam ke naam
pub const BUILTINS: &[&str] = &["lambayi", "upar", "neeche", "text", "sankhya", "chaabiyan"];

pub fn is_builtin(name: &str) -> bool {
    BUILTINS.contains(&name)
}

// Built-in kaam ko chalao - Err arity/type/domain galti par, Ok(value) success par
pub fn call_builtin(
    name: &str,
    args: Vec<Value>,
    line: usize,
    max_string_bytes: usize,
    max_list_items: usize,
) -> Result<Value, BoliError> {
    match name {
        "lambayi" => lambayi(&args, line),
        "upar" => text_map(&args, line, |s| s.to_uppercase(), "upar", max_string_bytes),
        "neeche" => text_map(
            &args,
            line,
            |s| s.to_lowercase(),
            "neeche",
            max_string_bytes,
        ),
        "text" => text_cast(&args, line, max_string_bytes),
        "sankhya" => sankhya(&args, line),
        "chaabiyan" => chaabiyan(&args, line, max_string_bytes, max_list_items),
        _ => Err(BoliError::line(
            line,
            format!("built-in kaam '{}' nahi mila", name),
        )),
    }
}

// Built-in ka text output bhi configured limit ke andar rakhna hai
fn checked_text(s: String, line: usize, max_string_bytes: usize) -> Result<Value, BoliError> {
    if s.len() > max_string_bytes {
        return Err(BoliError::line(
            line,
            format!("text bahut bada hai (max {} bytes)", max_string_bytes),
        ));
    }
    Ok(Value::Text(s))
}

fn type_naam(v: &Value) -> &'static str {
    match v {
        Value::Text(_) => "text",
        Value::Number(_) => "number",
        Value::Bool(_) => "sahi/galat",
        Value::List(_) => "list",
        Value::Map(_) => "map",
        Value::Khaali => "khaali",
    }
}

fn chaabiyan(
    args: &[Value],
    line: usize,
    max_string_bytes: usize,
    max_list_items: usize,
) -> Result<Value, BoliError> {
    arity("chaabiyan", args, 1, line)?;
    match &args[0] {
        Value::Map(m) => {
            if m.len() > max_list_items {
                return Err(BoliError::line(
                    line,
                    format!("map bahut bada hai (max {} chaabiyan)", max_list_items),
                ));
            }
            let mut total = 0usize;
            for k in m.keys() {
                total += k.len();
                if total > max_string_bytes {
                    return Err(BoliError::line(
                        line,
                        format!(
                            "chaabiyan text bahut bada hai (max {} bytes)",
                            max_string_bytes
                        ),
                    ));
                }
            }
            Ok(Value::List(
                m.keys().map(|k| Value::Text(k.clone())).collect::<Vec<_>>(),
            ))
        }
        other => Err(BoliError::line(
            line,
            format!(
                "chaabiyan sirf map par chal sakta hai, {} pe nahi",
                type_naam(other)
            ),
        )),
    }
}

fn arity(name: &str, args: &[Value], want: usize, line: usize) -> Result<(), BoliError> {
    if args.len() != want {
        return Err(BoliError::line(
            line,
            format!(
                "kaam '{}' ko {} cheezein chahiye, {} di gai",
                name,
                want,
                args.len()
            ),
        ));
    }
    Ok(())
}

fn lambayi(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("lambayi", args, 1, line)?;
    match &args[0] {
        Value::Text(s) => Ok(Value::Number(s.chars().count() as f64)),
        Value::List(items) => Ok(Value::Number(items.len() as f64)),
        Value::Map(m) => Ok(Value::Number(m.len() as f64)),
        Value::Khaali => Ok(Value::Number(0.0)),
        other => Err(BoliError::line(
            line,
            format!(
                "lambayi sirf text, list ya map par chal sakta hai, {} pe nahi",
                type_naam(other)
            ),
        )),
    }
}

fn text_cast(args: &[Value], line: usize, max_string_bytes: usize) -> Result<Value, BoliError> {
    arity("text", args, 1, line)?;
    checked_text(args[0].dikhane_laayak(), line, max_string_bytes)
}

fn text_map(
    args: &[Value],
    line: usize,
    f: fn(&str) -> String,
    name: &str,
    max_string_bytes: usize,
) -> Result<Value, BoliError> {
    arity(name, args, 1, line)?;
    match &args[0] {
        Value::Text(s) => checked_text(f(s), line, max_string_bytes),
        other => Err(BoliError::line(
            line,
            format!(
                "'{}' sirf text par chal sakta hai, {} pe nahi",
                name,
                type_naam(other)
            ),
        )),
    }
}

fn sankhya(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("sankhya", args, 1, line)?;
    match &args[0] {
        Value::Text(s) => {
            let n: f64 = s.trim().parse().map_err(|_| {
                BoliError::line(
                    line,
                    format!("'{}' sankhya nahi hai, number me convert nahi ho sakta", s),
                )
            })?;
            if !n.is_finite() {
                return Err(BoliError::line(
                    line,
                    format!("'{}' bahut bada number hai", s),
                ));
            }
            Ok(Value::Number(n))
        }
        Value::Number(n) => Ok(Value::Number(*n)),
        other => Err(BoliError::line(
            line,
            format!(
                "sankhya sirf text par chal sakta hai, {} pe nahi",
                type_naam(other),
            ),
        )),
    }
}
