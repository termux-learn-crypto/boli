use boli::ast::BoliError;
use boli::builtins::call_builtin;
use boli::interpreter::{Environment, Value};
use boli::parser::parse_source;
use boli::vm::Vm;

fn run_interp(src: &str) -> Result<Vec<String>, String> {
    let program = parse_source(src).map_err(|e: BoliError| e.to_string())?;
    let mut env = Environment::new();
    env.collect_output();
    env.run_program(&program)
        .map(|_| env.take_output())
        .map_err(|e: BoliError| e.to_string())
}

fn run_vm(src: &str) -> Result<Vec<String>, String> {
    let program = parse_source(src).map_err(|e: BoliError| e.to_string())?;
    let mut vm = Vm::new();
    vm.collect_output();
    vm.run_program(&program)
        .map(|_| vm.take_output())
        .map_err(|e: BoliError| e.to_string())
}

fn assert_parity(src: &str, expected: &[&str]) {
    let want = expected.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let interp = run_interp(src).unwrap_or_else(|e| panic!("interp fail for {}: {}", src, e));
    let vm = run_vm(src).unwrap_or_else(|e| panic!("vm fail for {}: {}", src, e));
    assert_eq!(interp, want, "interp output mismatch for {}", src);
    assert_eq!(vm, want, "vm output mismatch for {}", src);
    assert_eq!(vm, interp, "backend mismatch for {}", src);
}

fn assert_both_error(src: &str, fragment: &str) {
    let interp = run_interp(src).expect_err("interp should fail");
    let vm = run_vm(src).expect_err("vm should fail");
    assert!(interp.contains(fragment), "interp error: {}", interp);
    assert!(vm.contains(fragment), "vm error: {}", vm);
    assert_eq!(interp, vm, "backend error mismatch for {}", src);
}

#[test]
fn stdlib_string_operations() {
    assert_parity(r#"likho kato "namaste" 2 5"#, &["amas"]);
    assert_parity(r#"likho khojo "namaste boli" "boli""#, &["9"]);
    assert_parity(r#"likho khojo "namaste" "xyz""#, &["khaali"]);
    assert_parity(r#"likho hai_text "namaste" "mas""#, &["sahi"]);
    assert_parity(r#"likho badlo "a-b-c" "-" "/""#, &["a/b/c"]);
    assert_parity(
        r#"parts rakho baton "a,b,c" ","
likho jodo_text parts " | ""#,
        &["a | b | c"],
    );
    assert_parity(r#"likho chhilo "  boli  ""#, &["boli"]);
    assert_parity(r#"likho akshar "नमस्ते" 3"#, &["स"]);
    assert_parity(r#"likho escape_text "a\\b""#, &[r"a\\b"]);
    assert_parity(r#"likho unescape_text "a\\nb""#, &["a\nb"]);
}

#[test]
fn stdlib_list_operations() {
    let prefix = "cheezein rakho 1 2 3\n";
    assert_parity(
        &format!("{}likho jodega cheezein 4", prefix),
        &["1, 2, 3, 4"],
    );
    assert_parity(
        &format!("{}likho dalein cheezein 2 9", prefix),
        &["1, 9, 2, 3"],
    );
    assert_parity(&format!("{}likho hatao cheezein 2", prefix), &["1, 3"]);
    assert_parity(&format!("{}likho ant cheezein", prefix), &["3"]);
    assert_parity(&format!("{}likho ant_hatao cheezein", prefix), &["1, 2"]);
    assert_parity(&format!("{}likho tukda cheezein 2 3", prefix), &["2, 3"]);
    assert_parity(&format!("{}likho nakal cheezein", prefix), &["1, 2, 3"]);
    assert_parity(
        &format!(
            "{}khali_list rakho saaf cheezein\nlikho lambayi khali_list",
            prefix
        ),
        &["0"],
    );
    assert_parity(&format!("{}likho ultana cheezein", prefix), &["3, 2, 1"]);
    assert_parity(&format!("{}likho ek_hai cheezein 2", prefix), &["sahi"]);
    assert_parity(&format!("{}likho cheezein hissa 2", prefix), &["2"]);
}

#[test]
fn stdlib_math_operations() {
    assert_parity("likho purnank 2.9", &["2"]);
    assert_parity("likho uparank 2.1", &["3"]);
    assert_parity("likho nirapeksh ghatao 0 se 5", &["5"]);
    assert_parity("likho mool 9", &["3"]);
    assert_parity("likho shesh 10 3", &["1"]);
    assert_parity("likho sabse_chhota 5 2", &["2"]);
    assert_parity("likho sabse_bada 5 2", &["5"]);
}

#[test]
fn stdlib_type_and_test_helpers() {
    assert_parity(r#"likho prakaar "abc""#, &["text"]);
    assert_parity("likho prakaar 5", &["number"]);
    assert_parity(r#"likho prakaar_hai 5 "number""#, &["sahi"]);
    assert_parity(r#"likho sahi_banao """#, &["galat"]);
    assert_parity("likho sahi_banao 1", &["sahi"]);
    assert_parity("likho saman 2 2", &["sahi"]);
}

#[test]
fn stdlib_negative_cases() {
    assert_both_error(r#"likho kato "abc" 2 1"#, "shuru ka index ant se bada hai");
    assert_both_error(
        r#"likho akshar "abc" 0"#,
        "1 se shuru hone wala poora index",
    );
    assert_both_error("likho jodega 1 2", "list chahiye");
    assert_both_error("cheezein rakho 1 2 3\nlikho hatao cheezein 9", "bahar hai");
    assert_both_error("likho mool ghatao 0 se 1", "zero ya positive");
    assert_both_error("likho shesh 5 0", "zero nahi ho sakta");
    assert_both_error("likho saman 1 2", "saman fail");
    assert_both_error(r#"likho unescape_text "a\q""#, "unknown escape");
}

#[test]
fn stdlib_limits_are_enforced() {
    let list = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
    let err = call_builtin("jodega", vec![list, Value::Number(3.0)], 7, 1024, 2).unwrap_err();
    assert!(err.to_string().contains("bahut bada hai (max 2 items)"));

    let err = call_builtin(
        "baton",
        vec![
            Value::Text("aaaa".to_string()),
            Value::Text(",".to_string()),
        ],
        9,
        2,
        10,
    )
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("text bahut bada hai (max 2 bytes)")
    );

    let err = call_builtin(
        "jodo_text",
        vec![
            Value::List(vec![
                Value::Text("a".to_string()),
                Value::Text("b".to_string()),
            ]),
            Value::Text("-".to_string()),
        ],
        11,
        2,
        1,
    )
    .unwrap_err();
    assert!(err.to_string().contains("bahut bada hai (max 1 items)"));
}
