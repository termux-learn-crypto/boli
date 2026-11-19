use boli::ast::BoliError;
use boli::interpreter::Environment;
use boli::parser::parse_source;
use boli::vm::Vm;

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    src: &'static str,
    expect_error: bool,
}

fn run_interp(program: &[boli::ast::Stmt]) -> Result<Vec<String>, String> {
    let mut env = Environment::new();
    env.collect_output();
    env.run_program(program)
        .map(|_| env.take_output())
        .map_err(|e: BoliError| e.to_string())
}

fn run_vm(program: &[boli::ast::Stmt]) -> Result<Vec<String>, String> {
    let mut vm = Vm::new();
    vm.collect_output();
    vm.run_program(program)
        .map(|_| vm.take_output())
        .map_err(|e: BoliError| e.to_string())
}

fn check(case: Case) {
    let program =
        parse_source(case.src).unwrap_or_else(|e| panic!("{}: parse fail: {}", case.name, e));
    let interp = run_interp(&program);
    let vm = run_vm(&program);

    assert_eq!(
        interp.is_err(),
        case.expect_error,
        "{}: interpreter unexpected status: {:?}",
        case.name,
        interp
    );
    assert_eq!(
        vm.is_err(),
        case.expect_error,
        "{}: VM unexpected status: {:?}",
        case.name,
        vm
    );

    match (interp, vm) {
        (Ok(a), Ok(b)) => assert_eq!(a, b, "{}: output mismatch", case.name),
        (Err(a), Err(b)) => assert_eq!(a, b, "{}: error mismatch", case.name),
        (a, b) => panic!("{}: backend mismatch: {:?} vs {:?}", case.name, a, b),
    }
}

#[test]
fn vm_corpus_arith_and_values() {
    let cases = [
        Case {
            name: "jodo",
            src: "likho jodo 6 aur 7",
            expect_error: false,
        },
        Case {
            name: "ghatao",
            src: "likho ghatao 100 se 50",
            expect_error: false,
        },
        Case {
            name: "guna",
            src: "likho guna 5 ko 4 se",
            expect_error: false,
        },
        Case {
            name: "bhaag_decimal",
            src: "likho bhaag 10 ko 4 se",
            expect_error: false,
        },
        Case {
            name: "negative",
            src: "likho ghatao 5 se 10",
            expect_error: false,
        },
        Case {
            name: "number_precision",
            src: "likho 28.5\nlikho 0.5",
            expect_error: false,
        },
    ];
    for case in cases {
        check(case);
    }
}

#[test]
fn vm_corpus_conditions_and_logic() {
    let cases = [
        Case {
            name: "agar_warna",
            src: r#"agar 2 1 se bada hai to
    likho "bada"
nahi to
    likho "chhota"
khatam"#,
            expect_error: false,
        },
        Case {
            name: "aur_ya_precedence",
            src: r#"agar sahi sahi barabar hai ya galat sahi barabar hai aur galat galat barabar hai to
    likho "ok"
khatam"#,
            expect_error: false,
        },
        Case {
            name: "nested_agar",
            src: r#"x rakho 7
agar x 5 se bada hai to
    agar x 10 se chhota hai to
        likho "middle"
    khatam
khatam"#,
            expect_error: false,
        },
        Case {
            name: "condition_type_error",
            src: r#"agar "abc" 5 se bada hai to
    likho "x"
khatam"#,
            expect_error: true,
        },
    ];
    for case in cases {
        check(case);
    }
}

#[test]
fn vm_corpus_variables_lists_and_maps() {
    let cases = [
        Case {
            name: "variable",
            src: r#"naam rakho "Vilas"
likho naam"#,
            expect_error: false,
        },
        Case {
            name: "list_index_assign",
            src: r#"cheezein rakho 1 2 3
cheezein hissa 1 rakho 9
likho cheezein
likho cheezein hissa 1
likho lambayi cheezein"#,
            expect_error: false,
        },
        Case {
            name: "map_assign",
            src: r#"mapa rakho map "a" 1
mapa hissa "b" rakho 2
mapa hissa "a" rakho 7
likho mapa
likho chaabiyan mapa
likho lambayi mapa"#,
            expect_error: false,
        },
        Case {
            name: "list_oob",
            src: r#"cheezein rakho 1 2 3
likho cheezein hissa 9"#,
            expect_error: true,
        },
        Case {
            name: "map_missing_key",
            src: r#"mapa rakho map "a" 1
likho mapa hissa "z""#,
            expect_error: true,
        },
        Case {
            name: "name_hint",
            src: r#"naam rakho 1
likho nam"#,
            expect_error: true,
        },
    ];
    for case in cases {
        check(case);
    }
}

#[test]
fn vm_corpus_loops() {
    let cases = [
        Case {
            name: "range_loop",
            src: r#"1 se 5 tak
    likho ginti
khatam"#,
            expect_error: false,
        },
        Case {
            name: "while_loop",
            src: r#"i rakho 1
jab tak i 5 se chhota ya barabar hai to
    likho i
    jodo i i aur 1
khatam"#,
            expect_error: false,
        },
        Case {
            name: "ruk",
            src: r#"1 se 5 tak
    agar ginti 3 barabar hai to
        ruk
    khatam
    likho ginti
khatam"#,
            expect_error: false,
        },
        Case {
            name: "agla",
            src: r#"1 se 5 tak
    agar ginti 3 barabar hai to
        agla
    khatam
    likho ginti
khatam"#,
            expect_error: false,
        },
        Case {
            name: "range_body_ginti_shadow",
            src: r#"1 se 3 tak
    ginti rakho khaali
    ruk
khatam
likho "done""#,
            expect_error: false,
        },
    ];
    for case in cases {
        check(case);
    }
}

#[test]
fn vm_corpus_kaam_and_builtins() {
    let cases = [
        Case {
            name: "kaam_return",
            src: r#"kaam double x
    guna jawab x ko 2 se
    wapas do jawab
khatam
likho double 21"#,
            expect_error: false,
        },
        Case {
            name: "kaam_recursion",
            src: r#"kaam fact n
    agar n 1 se chhota ya barabar hai to
        wapas do 1
    khatam
    ghatao prev n se 1
    guna jawab n ko fact prev se
    wapas do jawab
khatam
likho fact 5"#,
            expect_error: false,
        },
        Case {
            name: "kaam_zero_arg",
            src: r#"kaam jawab_do
    wapas do 42
khatam
likho jawab_do"#,
            expect_error: false,
        },
        Case {
            name: "kaam_missing",
            src: "likho naam_nahi_mila_kaam",
            expect_error: true,
        },
        Case {
            name: "kaam_wrong_arity",
            src: r#"kaam f a
    likho a
khatam
f 1 2"#,
            expect_error: true,
        },
        Case {
            name: "builtins",
            src: r#"likho lambayi "namaste"
likho upar "boli"
likho text 42
likho sankhya "28.5""#,
            expect_error: false,
        },
        Case {
            name: "kaam_as_value",
            src: r#"kaam f x
    wapas do x
khatam
likho f"#,
            expect_error: true,
        },
    ];
    for case in cases {
        check(case);
    }
}

#[test]
fn vm_corpus_errors_and_traces() {
    let cases = [
        Case {
            name: "div_zero",
            src: "likho bhaag 5 ko 0 se",
            expect_error: true,
        },
        Case {
            name: "undefined_var",
            src: "likho naam_nahi_mila_var",
            expect_error: true,
        },
        Case {
            name: "top_level_return",
            src: "wapas do 7",
            expect_error: true,
        },
        Case {
            name: "ruk_outside_loop",
            src: "ruk",
            expect_error: true,
        },
        Case {
            name: "nested_trace",
            src: r#"kaam inner
    likho missing_x
khatam
kaam outer
    inner
khatam
outer"#,
            expect_error: true,
        },
    ];
    for case in cases {
        check(case);
    }
}
