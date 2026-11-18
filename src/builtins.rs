// BOLI built-in kaam (standard library) - pure Rust, koi external dependency nahi
// User-defined kaam in names ko shadow kar sakta hai (user kaam ki priority zyada)

use crate::ast::BoliError;
use crate::interpreter::Value;

// Built-in kaam ke naam
pub const BUILTINS: &[&str] = &["lambayi", "upar", "neeche", "text", "sankhya"];

pub fn is_builtin(name: &str) -> bool {
    BUILTINS.contains(&name)
}

// Built-in kaam ko chalao - Err arity/type/domain galti par, Ok(value) success par
pub fn call_builtin(name: &str, args: Vec<Value>, line: usize) -> Result<Value, BoliError> {
    match name {
        "lambayi" => lambayi(&args, line),
        "upar" => text_map(&args, line, |s| s.to_uppercase(), "upar"),
        "neeche" => text_map(&args, line, |s| s.to_lowercase(), "neeche"),
        "text" => text_cast(&args, line),
        "sankhya" => sankhya(&args, line),
        _ => Err(BoliError::line(
            line,
            format!("built-in kaam '{}' nahi mila", name),
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
        Value::Khaali => Ok(Value::Number(0.0)),
        other => Err(BoliError::line(
            line,
            format!(
                "lambayi sirf text ya list par chal sakta hai, {} pe nahi",
                match other {
                    Value::Text(_) => "text",
                    Value::Number(_) => "number",
                    Value::Bool(_) => "sahi/galat",
                    Value::List(_) => "list",
                    Value::Khaali => "khaali",
                }
            ),
        )),
    }
}

fn text_cast(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("text", args, 1, line)?;
    Ok(Value::Text(args[0].dikhane_laayak()))
}

fn text_map(
    args: &[Value],
    line: usize,
    f: fn(&str) -> String,
    name: &str,
) -> Result<Value, BoliError> {
    arity(name, args, 1, line)?;
    match &args[0] {
        Value::Text(s) => Ok(Value::Text(f(s))),
        other => Err(BoliError::line(
            line,
            format!(
                "'{}' sirf text par chal sakta hai, {} pe nahi",
                name,
                match other {
                    Value::Text(_) => "text",
                    Value::Number(_) => "number",
                    Value::Bool(_) => "sahi/galat",
                    Value::List(_) => "list",
                    Value::Khaali => "khaali",
                }
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
                match other {
                    Value::Text(_) => "text",
                    Value::Number(_) => "number",
                    Value::Bool(_) => "sahi/galat",
                    Value::List(_) => "list",
                    Value::Khaali => "khaali",
                }
            ),
        )),
    }
}
