// BOLI built-in kaam (standard library) - pure Rust, koi external dependency nahi
// User-defined kaam in names ko shadow kar sakta hai (user kaam ki priority zyada)

use crate::ast::BoliError;
use crate::interpreter::Value;

// Built-in kaam ke naam
pub const BUILTINS: &[&str] = &[
    "lambayi",
    "upar",
    "neeche",
    "text",
    "sankhya",
    "chaabiyan",
    "kato",
    "khojo",
    "hai_text",
    "badlo",
    "baton",
    "jodo_text",
    "chhilo",
    "akshar",
    "escape_text",
    "unescape_text",
    "jodega",
    "dalein",
    "hatao",
    "ant",
    "ant_hatao",
    "tukda",
    "nakal",
    "saaf",
    "ultana",
    "ek_hai",
    "purnank",
    "uparank",
    "nirapeksh",
    "mool",
    "shesh",
    "sabse_chhota",
    "sabse_bada",
    "prakaar",
    "prakaar_hai",
    "sahi_banao",
    "saman",
];

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
        "kato" => slice_text(&args, line, max_string_bytes),
        "khojo" => find_text(&args, line),
        "hai_text" => contains_text(&args, line),
        "badlo" => replace_text(&args, line, max_string_bytes),
        "baton" => split_text(&args, line, max_string_bytes, max_list_items),
        "jodo_text" => join_text(&args, line, max_string_bytes, max_list_items),
        "chhilo" => trim_text(&args, line, max_string_bytes),
        "akshar" => char_at(&args, line, max_string_bytes),
        "escape_text" => escape_text_call(&args, line, max_string_bytes),
        "unescape_text" => unescape_text_call(&args, line, max_string_bytes),
        "jodega" => append_list(&args, line, max_list_items),
        "dalein" => insert_list(&args, line, max_list_items),
        "hatao" => remove_list(&args, line, max_list_items),
        "ant" => last_value(&args, line),
        "ant_hatao" => drop_last(&args, line, max_list_items),
        "tukda" => slice_list(&args, line, max_list_items),
        "nakal" => copy_list(&args, line, max_list_items),
        "saaf" => clear_list(&args, line),
        "ultana" => reverse_list(&args, line, max_list_items),
        "ek_hai" => contains_list(&args, line),
        "purnank" => unary_math(&args, line, "purnank", f64::floor),
        "uparank" => unary_math(&args, line, "uparank", f64::ceil),
        "nirapeksh" => unary_math(&args, line, "nirapeksh", f64::abs),
        "mool" => sqrt(&args, line),
        "shesh" => remainder(&args, line),
        "sabse_chhota" => binary_math(&args, line, "sabse_chhota", f64::min),
        "sabse_bada" => binary_math(&args, line, "sabse_bada", f64::max),
        "prakaar" => type_of(&args, line),
        "prakaar_hai" => is_type(&args, line),
        "sahi_banao" => to_bool(&args, line),
        "saman" => assert_same(&args, line),
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

fn expect_text<'a>(
    args: &'a [Value],
    index: usize,
    name: &str,
    line: usize,
) -> Result<&'a str, BoliError> {
    match &args[index] {
        Value::Text(s) => Ok(s),
        other => Err(BoliError::line(
            line,
            format!("'{}' ko text chahiye, {} diya gaya", name, type_naam(other)),
        )),
    }
}

fn expect_list<'a>(
    args: &'a [Value],
    index: usize,
    name: &str,
    line: usize,
) -> Result<&'a Vec<Value>, BoliError> {
    match &args[index] {
        Value::List(items) => Ok(items),
        other => Err(BoliError::line(
            line,
            format!("'{}' ko list chahiye, {} diya gaya", name, type_naam(other)),
        )),
    }
}

fn expect_number(args: &[Value], index: usize, name: &str, line: usize) -> Result<f64, BoliError> {
    match args[index] {
        Value::Number(n) if n.is_finite() => Ok(n),
        Value::Number(_) => Err(BoliError::line(
            line,
            format!("'{}' ko finite number chahiye", name),
        )),
        ref other => Err(BoliError::line(
            line,
            format!(
                "'{}' ko number chahiye, {} diya gaya",
                name,
                type_naam(other)
            ),
        )),
    }
}

fn index_1(value: f64, name: &str, line: usize, max: usize) -> Result<usize, BoliError> {
    if !value.is_finite() || value.fract() != 0.0 || value < 1.0 {
        return Err(BoliError::line(
            line,
            format!("'{}' ko 1 se shuru hone wala poora index chahiye", name),
        ));
    }
    let index = value as usize;
    if index == 0 || index > max {
        return Err(BoliError::line(
            line,
            format!("'{}' ka index {} se bahar hai (1 se {})", name, value, max),
        ));
    }
    Ok(index - 1)
}

fn slice_bounds(
    start: f64,
    end: f64,
    name: &str,
    line: usize,
    len: usize,
) -> Result<(usize, usize), BoliError> {
    let from = index_1(start, name, line, len)?;
    let to = index_1(end, name, line, len)?;
    if from > to {
        return Err(BoliError::line(
            line,
            format!("'{}' me shuru ka index ant se bada hai", name),
        ));
    }
    Ok((from, to + 1))
}

fn checked_list(
    items: Vec<Value>,
    name: &str,
    line: usize,
    max_list_items: usize,
) -> Result<Value, BoliError> {
    if items.len() > max_list_items {
        return Err(BoliError::line(
            line,
            format!(
                "'{}' ka result bahut bada hai (max {} items)",
                name, max_list_items
            ),
        ));
    }
    Ok(Value::List(items))
}

fn checked_text_parts(
    parts: Vec<String>,
    name: &str,
    line: usize,
    max_string_bytes: usize,
    max_list_items: usize,
) -> Result<Value, BoliError> {
    if parts.len() > max_list_items {
        return Err(BoliError::line(
            line,
            format!(
                "'{}' ka result bahut bada hai (max {} items)",
                name, max_list_items
            ),
        ));
    }
    let mut total = 0usize;
    for part in &parts {
        total = total.saturating_add(part.len());
        if total > max_string_bytes {
            return Err(BoliError::line(
                line,
                format!(
                    "'{}' ka text bahut bada hai (max {} bytes)",
                    name, max_string_bytes
                ),
            ));
        }
    }
    Ok(Value::List(
        parts.into_iter().map(Value::Text).collect::<Vec<_>>(),
    ))
}

fn slice_text(args: &[Value], line: usize, max_string_bytes: usize) -> Result<Value, BoliError> {
    arity("kato", args, 3, line)?;
    let s = expect_text(args, 0, "kato", line)?;
    let start = expect_number(args, 1, "kato", line)?;
    let end = expect_number(args, 2, "kato", line)?;
    let chars = s.chars().collect::<Vec<_>>();
    let (from, to) = slice_bounds(start, end, "kato", line, chars.len())?;
    let out = chars[from..to].iter().collect::<String>();
    checked_text(out, line, max_string_bytes)
}

fn find_text(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("khojo", args, 2, line)?;
    let haystack = expect_text(args, 0, "khojo", line)?;
    let needle = expect_text(args, 1, "khojo", line)?;
    match haystack.find(needle) {
        Some(byte_index) => {
            let char_index = haystack[..byte_index].chars().count() + 1;
            Ok(Value::Number(char_index as f64))
        }
        None => Ok(Value::Khaali),
    }
}

fn contains_text(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("hai_text", args, 2, line)?;
    let haystack = expect_text(args, 0, "hai_text", line)?;
    let needle = expect_text(args, 1, "hai_text", line)?;
    Ok(Value::Bool(haystack.contains(needle)))
}

fn replace_text(args: &[Value], line: usize, max_string_bytes: usize) -> Result<Value, BoliError> {
    arity("badlo", args, 3, line)?;
    let s = expect_text(args, 0, "badlo", line)?;
    let old = expect_text(args, 1, "badlo", line)?;
    let new = expect_text(args, 2, "badlo", line)?;
    checked_text(s.replace(old, new), line, max_string_bytes)
}

fn split_text(
    args: &[Value],
    line: usize,
    max_string_bytes: usize,
    max_list_items: usize,
) -> Result<Value, BoliError> {
    arity("baton", args, 2, line)?;
    let s = expect_text(args, 0, "baton", line)?;
    let sep = expect_text(args, 1, "baton", line)?;
    let parts = s
        .split(sep)
        .map(|part| part.to_string())
        .collect::<Vec<_>>();
    checked_text_parts(parts, "baton", line, max_string_bytes, max_list_items)
}

fn join_text(
    args: &[Value],
    line: usize,
    max_string_bytes: usize,
    max_list_items: usize,
) -> Result<Value, BoliError> {
    arity("jodo_text", args, 2, line)?;
    let items = expect_list(args, 0, "jodo_text", line)?;
    if items.len() > max_list_items {
        return Err(BoliError::line(
            line,
            format!(
                "jodo_text ka result bahut bada hai (max {} items)",
                max_list_items
            ),
        ));
    }
    let sep = expect_text(args, 1, "jodo_text", line)?;
    let mut parts = Vec::with_capacity(items.len());
    for item in items {
        match item {
            Value::Text(part) => parts.push(part.clone()),
            other => {
                return Err(BoliError::line(
                    line,
                    format!("jodo_text ko text list chahiye, {} mila", type_naam(other)),
                ));
            }
        }
    }
    checked_text(parts.join(sep), line, max_string_bytes)
}

fn trim_text(args: &[Value], line: usize, max_string_bytes: usize) -> Result<Value, BoliError> {
    arity("chhilo", args, 1, line)?;
    let s = expect_text(args, 0, "chhilo", line)?;
    checked_text(s.trim().to_string(), line, max_string_bytes)
}

fn char_at(args: &[Value], line: usize, max_string_bytes: usize) -> Result<Value, BoliError> {
    arity("akshar", args, 2, line)?;
    let s = expect_text(args, 0, "akshar", line)?;
    let index = expect_number(args, 1, "akshar", line)?;
    let chars = s.chars().collect::<Vec<_>>();
    let pos = index_1(index, "akshar", line, chars.len())?;
    checked_text(chars[pos].to_string(), line, max_string_bytes)
}

fn escape_text_call(
    args: &[Value],
    line: usize,
    max_string_bytes: usize,
) -> Result<Value, BoliError> {
    arity("escape_text", args, 1, line)?;
    let s = expect_text(args, 0, "escape_text", line)?;
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    checked_text(out, line, max_string_bytes)
}

fn unescape_text_call(
    args: &[Value],
    line: usize,
    max_string_bytes: usize,
) -> Result<Value, BoliError> {
    arity("unescape_text", args, 1, line)?;
    let s = expect_text(args, 0, "unescape_text", line)?;
    let mut chars = s.chars();
    let mut out = String::new();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some(other) => {
                return Err(BoliError::line(
                    line,
                    format!("'unescape_text' me unknown escape \\{}", other),
                ));
            }
            None => {
                return Err(BoliError::line(
                    line,
                    "unescape_text me escape adhoora hai".to_string(),
                ));
            }
        }
    }
    checked_text(out, line, max_string_bytes)
}

fn append_list(args: &[Value], line: usize, max_list_items: usize) -> Result<Value, BoliError> {
    arity("jodega", args, 2, line)?;
    let items = expect_list(args, 0, "jodega", line)?;
    let mut out = items.clone();
    out.push(args[1].clone());
    checked_list(out, "jodega", line, max_list_items)
}

fn insert_list(args: &[Value], line: usize, max_list_items: usize) -> Result<Value, BoliError> {
    arity("dalein", args, 3, line)?;
    let items = expect_list(args, 0, "dalein", line)?;
    let index = expect_number(args, 1, "dalein", line)?;
    let pos = index_1(index, "dalein", line, items.len() + 1)?;
    let mut out = items.clone();
    out.insert(pos, args[2].clone());
    checked_list(out, "dalein", line, max_list_items)
}

fn remove_list(args: &[Value], line: usize, max_list_items: usize) -> Result<Value, BoliError> {
    arity("hatao", args, 2, line)?;
    let items = expect_list(args, 0, "hatao", line)?;
    let index = expect_number(args, 1, "hatao", line)?;
    let pos = index_1(index, "hatao", line, items.len())?;
    let mut out = items.clone();
    out.remove(pos);
    checked_list(out, "hatao", line, max_list_items)
}

fn last_value(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("ant", args, 1, line)?;
    let items = expect_list(args, 0, "ant", line)?;
    Ok(items.last().cloned().unwrap_or(Value::Khaali))
}

fn drop_last(args: &[Value], line: usize, max_list_items: usize) -> Result<Value, BoliError> {
    arity("ant_hatao", args, 1, line)?;
    let items = expect_list(args, 0, "ant_hatao", line)?;
    let mut out = items.clone();
    out.pop();
    checked_list(out, "ant_hatao", line, max_list_items)
}

fn slice_list(args: &[Value], line: usize, max_list_items: usize) -> Result<Value, BoliError> {
    arity("tukda", args, 3, line)?;
    let items = expect_list(args, 0, "tukda", line)?;
    let start = expect_number(args, 1, "tukda", line)?;
    let end = expect_number(args, 2, "tukda", line)?;
    let (from, to) = slice_bounds(start, end, "tukda", line, items.len())?;
    checked_list(items[from..to].to_vec(), "tukda", line, max_list_items)
}

fn copy_list(args: &[Value], line: usize, max_list_items: usize) -> Result<Value, BoliError> {
    arity("nakal", args, 1, line)?;
    let items = expect_list(args, 0, "nakal", line)?;
    checked_list(items.clone(), "nakal", line, max_list_items)
}

fn clear_list(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("saaf", args, 1, line)?;
    expect_list(args, 0, "saaf", line)?;
    Ok(Value::List(Vec::new()))
}

fn reverse_list(args: &[Value], line: usize, max_list_items: usize) -> Result<Value, BoliError> {
    arity("ultana", args, 1, line)?;
    let items = expect_list(args, 0, "ultana", line)?;
    let mut out = items.clone();
    out.reverse();
    checked_list(out, "ultana", line, max_list_items)
}

fn contains_list(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("ek_hai", args, 2, line)?;
    let items = expect_list(args, 0, "ek_hai", line)?;
    Ok(Value::Bool(items.contains(&args[1])))
}

fn unary_math(
    args: &[Value],
    line: usize,
    name: &str,
    f: fn(f64) -> f64,
) -> Result<Value, BoliError> {
    arity(name, args, 1, line)?;
    let n = expect_number(args, 0, name, line)?;
    let out = f(n);
    if !out.is_finite() {
        return Err(BoliError::line(
            line,
            format!("'{}' ka result finite number nahi hai", name),
        ));
    }
    Ok(Value::Number(out))
}

fn sqrt(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("mool", args, 1, line)?;
    let n = expect_number(args, 0, "mool", line)?;
    if n < 0.0 {
        return Err(BoliError::line(
            line,
            "mool ke liye number zero ya positive hona chahiye".to_string(),
        ));
    }
    Ok(Value::Number(n.sqrt()))
}

fn remainder(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("shesh", args, 2, line)?;
    let a = expect_number(args, 0, "shesh", line)?;
    let b = expect_number(args, 1, "shesh", line)?;
    if b == 0.0 {
        return Err(BoliError::line(
            line,
            "shesh me doosra number zero nahi ho sakta".to_string(),
        ));
    }
    Ok(Value::Number(a % b))
}

fn binary_math(
    args: &[Value],
    line: usize,
    name: &str,
    f: fn(f64, f64) -> f64,
) -> Result<Value, BoliError> {
    arity(name, args, 2, line)?;
    let a = expect_number(args, 0, name, line)?;
    let b = expect_number(args, 1, name, line)?;
    Ok(Value::Number(f(a, b)))
}

fn type_of(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("prakaar", args, 1, line)?;
    Ok(Value::Text(type_naam(&args[0]).to_string()))
}

fn is_type(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("prakaar_hai", args, 2, line)?;
    let wanted = expect_text(args, 1, "prakaar_hai", line)?;
    Ok(Value::Bool(type_naam(&args[0]) == wanted))
}

fn to_bool(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("sahi_banao", args, 1, line)?;
    let result = match &args[0] {
        Value::Bool(b) => *b,
        Value::Number(n) => *n != 0.0,
        Value::Text(s) => !s.is_empty(),
        Value::List(items) => !items.is_empty(),
        Value::Map(items) => !items.is_empty(),
        Value::Khaali => false,
    };
    Ok(Value::Bool(result))
}

fn assert_same(args: &[Value], line: usize) -> Result<Value, BoliError> {
    arity("saman", args, 2, line)?;
    if args[0] == args[1] {
        Ok(Value::Bool(true))
    } else {
        Err(BoliError::line(
            line,
            format!(
                "saman fail: {} aur {} same nahi hain",
                type_naam(&args[0]),
                type_naam(&args[1])
            ),
        ))
    }
}
