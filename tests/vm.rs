// BOLI bytecode VM ke tests — tree-walking interpreter ke parantu pariyay (parity)
// Har test VM pe snippet chala kar expected output verify karta hai

use boli::ast::BoliError;
use boli::interpreter::Environment;
use boli::parser::parse_source;
use boli::vm::Vm;

fn vm_out(src: &str) -> Result<Vec<String>, BoliError> {
    let program = parse_source(src)?;
    let mut vm = Vm::new();
    vm.collect_output();
    vm.run_program(&program)?;
    Ok(vm.take_output())
}

fn interp_out(src: &str) -> Result<Vec<String>, BoliError> {
    let program = parse_source(src)?;
    let mut env = Environment::new();
    env.collect_output();
    env.run_program(&program)?;
    Ok(env.take_output())
}

// ---------- basic arithmetic ----------

#[test]
fn vm_jodo_and_inline() {
    assert_eq!(vm_out("likho jodo 6 aur 7").unwrap(), vec!["13"]);
    assert_eq!(vm_out("likho jodo 100 aur 50").unwrap(), vec!["150"]);
    assert_eq!(vm_out("likho ghatao 100 se 50").unwrap(), vec!["50"]);
}

#[test]
fn vm_guna_and_bhaag() {
    assert_eq!(vm_out("likho guna 5 ko 4 se").unwrap(), vec!["20"]);
    assert_eq!(vm_out("likho bhaag 10 ko 4 se").unwrap(), vec!["2.5"]);
}

#[test]
fn vm_div_by_zero() {
    assert!(vm_out("likho bhaag 5 ko 0 se").is_err());
}

#[test]
fn vm_arith_overflow() {
    let src = "y rakho 1000000000000000\nguna y y ko y se\nguna y y ko y se\nguna y y ko y se\nguna y y ko y se\nguna y y ko y se\nlikho y";
    assert!(vm_out(src).is_err());
}

// ---------- variables and lists/maps ----------

#[test]
fn vm_rakho_reuse() {
    assert_eq!(
        vm_out("naam rakho \"Vilas\"\nlikho naam").unwrap(),
        vec!["Vilas"]
    );
}

#[test]
fn vm_list_create_and_index() {
    assert_eq!(
        vm_out("cheezein rakho 1 2 3\nlikho cheezein\nlikho cheezein hissa 2").unwrap(),
        vec!["1, 2, 3", "2"]
    );
}

#[test]
fn vm_map_literal_and_access() {
    let src = "mapa rakho map \"naam\" \"Vilas\" \"umar\" 28\nlikho mapa\nlikho mapa hissa \"naam\"\nlikho mapa hissa \"umar\"";
    assert_eq!(
        vm_out(src).unwrap(),
        vec!["naam: Vilas, umar: 28", "Vilas", "28"]
    );
}

#[test]
fn vm_map_assign() {
    let src = "mapa rakho map \"a\" 1\nmapa hissa \"b\" rakho 2\nlikho lambayi mapa\nlikho chaabiyan mapa";
    assert_eq!(vm_out(src).unwrap(), vec!["2", "a, b"]);
}

// ---------- control flow ----------

#[test]
fn vm_agar_then_else() {
    let src = "x rakho 5\nagar x 3 se bada hai to\nlikho \"bada\"\nkhatam";
    assert_eq!(vm_out(src).unwrap(), vec!["bada"]);
    let src2 =
        "x rakho 1\nagar x 3 se bada hai to\nlikho \"bada\"\nnahi to\nlikho \"chhota\"\nkhatam";
    assert_eq!(vm_out(src2).unwrap(), vec!["chhota"]);
}

#[test]
fn vm_jab_tak_loop() {
    let src = "i rakho 1\njab tak i 5 se chhota ya barabar hai to\nlikho i\njodo i i aur 1\nkhatam";
    assert!(vm_out(src).unwrap() == vec!["1", "2", "3", "4", "5"]);
}

#[test]
fn vm_range_loop() {
    assert_eq!(
        vm_out("1 se 5 tak\nlikho ginti\nkhatam").unwrap(),
        vec!["1", "2", "3", "4", "5"]
    );
}

#[test]
fn vm_ruk_and_agla() {
    let src = "1 se 5 tak\n    agar ginti 3 barabar hai to\n        ruk\n    khatam\n    likho ginti\nkhatam";
    assert_eq!(vm_out(src).unwrap(), vec!["1", "2"]);
    let src2 = "1 se 5 tak\n    agar ginti 3 barabar hai to\n        agla\n    khatam\n    likho ginti\nkhatam";
    assert_eq!(vm_out(src2).unwrap(), vec!["1", "2", "4", "5"]);
}

#[test]
fn vm_aur_binds_tighter_than_ya() {
    // a ya b aur c = a ya (b aur c)
    let src = "agar galat galat barabar hai ya sahi sahi barabar hai aur sahi sahi barabar hai to\nlikho \"sahi\"\nkhatam";
    assert_eq!(vm_out(src).unwrap(), vec!["sahi"]);
}

// ---------- kaam (functions) ----------

#[test]
fn vm_kaam_call() {
    let src =
        "kaam double x\n    guna jawab x ko 2 se\n    wapas do jawab\nkhatam\nlikho double 21";
    assert_eq!(vm_out(src).unwrap(), vec!["42"]);
}

#[test]
fn vm_kaam_zero_arg() {
    let src = "kaam jawab_do\n    wapas do 42\nkhatam\nlikho jawab_do";
    assert_eq!(vm_out(src).unwrap(), vec!["42"]);
}

#[test]
fn vm_kaam_recursion() {
    let src = "kaam fact n\n    agar n 1 se chhota ya barabar hai to\n        wapas do 1\n    khatam\n    ghatao prev n se 1\n    guna jawab n ko fact prev se\n    wapas do jawab\nkhatam\nlikho fact 5";
    assert_eq!(vm_out(src).unwrap(), vec!["120"]);
}

#[test]
fn vm_kaam_missing_error() {
    assert!(vm_out("likho naam_nahi_mila_kaam").is_err());
}

// ---------- builtins ----------

#[test]
fn vm_builtins() {
    assert_eq!(vm_out("likho lambayi \"namaste\"").unwrap(), vec!["7"]);
    assert_eq!(vm_out("likho upar \"boli\"").unwrap(), vec!["BOLI"]);
    assert_eq!(vm_out("likho text 42").unwrap(), vec!["42"]);
    assert_eq!(vm_out("likho sankhya \"28.5\"").unwrap(), vec!["28.5"]);
}

// ---------- errors & safety ----------

#[test]
fn vm_unknown_variable_error() {
    let err = vm_out("likho naam_nahi_mila_var").unwrap_err();
    assert!(
        err.to_string()
            .contains("naam 'naam_nahi_mila_var' nahi mila")
    );
}

#[test]
fn vm_error_has_traceback() {
    let src = "kaam galti_wala\n    likho naam_nahi_mila_var\nkhatam\n\
               kaam namaste_wala\n    galti_wala\nkhatam\n\
               namaste_wala";
    let err = vm_out(src).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("kaam 'galti_wala' ke andar"), "msg: {}", msg);
    assert!(msg.contains("kaam 'namaste_wala' ke andar"), "msg: {}", msg);
}

#[test]
fn vm_steps_limit_stops_infinite_loop() {
    let mut env = Vm::new();
    let program = parse_source("jab tak sahi sahi barabar hai to\n    likho 1\nkhatam").unwrap();
    env.collect_output();
    let err = env.run_program(&program).unwrap_err();
    assert!(err.to_string().contains("steps"));
}

// ---------- audit fixes (v0.6.1) ----------

#[test]
fn vm_failing_range_program_then_reuse_same_vm() {
    // failing program ke baad bhi same Vm reuse hona chahiye
    let failing = parse_source("1 se 3 tak\n    likho undefined_var\nkhatam").unwrap();
    let good = parse_source("likho \"reuse ok\"").unwrap();
    let mut vm = Vm::new();
    vm.collect_output();
    assert!(vm.run_program(&failing).is_err());
    assert_eq!(vm.stack_depth(), 0);
    vm.run_program(&good).unwrap();
    assert_eq!(vm.take_output(), vec!["reuse ok"]);
    assert_eq!(vm.stack_depth(), 0);
}

#[test]
fn vm_param_bind_budget_failure_does_not_leak_scope() {
    let big = "a".repeat(300);
    let src = format!(
        "kaam f a b\n    likho \"bind-ok\"\nkhatam\nf \"{}\" \"{}\"",
        big, big
    );
    let program = parse_source(&src).unwrap();
    let mut vm = Vm::new().with_total_budget_bytes(512);
    assert!(vm.run_program(&program).is_err());
    let program2 = parse_source("likho \"reuse ok\"").unwrap();
    vm.collect_output();
    vm.run_program(&program2).unwrap();
    assert_eq!(vm.take_output(), vec!["reuse ok"]);
}

#[test]
fn vm_constants_over_u16_boundary_work() {
    // 70,000 distinct number literals -> const indices exceed 65,535
    let mut src = String::new();
    for i in 0..70000 {
        src.push_str(&format!("likho {}\n", i));
    }
    let program = parse_source(&src).unwrap();
    let mut vm = Vm::new().with_max_steps(300000);
    vm.collect_output();
    vm.run_program(&program).unwrap();
    let out = vm.take_output();
    assert_eq!(out.len(), 70000);
    assert_eq!(out[69999], "69999");
}

#[test]
fn vm_list_and_map_assignment_compiles_rhs_once_and_clears_stack() {
    let src = "cheezein rakho 1 2 3\ncheezein hissa 2 rakho 99\ncheezein hissa 2 rakho 100\nmapa rakho map \"a\" 1\nmapa hissa \"b\" rakho 2\nmapa hissa \"a\" rakho 7\nlikho cheezein\nlikho mapa";
    let program = parse_source(src).unwrap();
    let mut vm = Vm::new();
    vm.collect_output();
    vm.run_program(&program).unwrap();
    assert_eq!(vm.take_output(), vec!["1, 100, 3", "a: 7, b: 2"]);
    assert_eq!(vm.stack_depth(), 0);
}

#[test]
fn vm_error_resets_stack_to_baseline() {
    let bad = parse_source("mapa hissa \"x\" rakho 1").unwrap();
    let mut vm = Vm::new();
    vm.collect_output();
    assert!(vm.run_program(&bad).is_err());
    assert_eq!(vm.stack_depth(), 0);
    let good = parse_source("likho \"ok\"").unwrap();
    vm.run_program(&good).unwrap();
    assert_eq!(vm.take_output(), vec!["ok"]);
    assert_eq!(vm.stack_depth(), 0);
}

#[test]
fn vm_steps_reset_between_programs() {
    let program = parse_source("likho 1\nlikho 2").unwrap();
    let mut vm = Vm::new().with_max_steps(4);
    vm.collect_output();
    vm.run_program(&program).unwrap();
    vm.run_program(&program).unwrap();
    assert_eq!(vm.take_output(), vec!["1", "2", "1", "2"]);
}

#[test]
fn vm_matches_interpreter_on_core_corpus() {
    let corpus = [
        "likho jodo 6 aur 7",
        "x rakho 5\nlikho x",
        "cheezein rakho 1 2 3\ncheezein hissa 1 rakho 9\nlikho cheezein",
        "mapa rakho map \"a\" 1\nmapa hissa \"b\" rakho 2\nlikho chaabiyan mapa",
        "kaam f a\n    wapas do jodo a aur 1\nkhatam\nlikho f 4",
        "1 se 3 tak\nlikho ginti\nkhatam",
        "agar 2 1 se bada hai to\nlikho \"haan\"\nwarna\nlikho \"nahin\"\nkhatam",
    ];
    for src in corpus {
        match (interp_out(src), vm_out(src)) {
            (Ok(a), Ok(b)) => assert_eq!(a, b, "backend output mismatch: {}", src),
            (Err(_), Err(_)) => {}
            (a, b) => panic!("backend mismatch for {}: {:?} vs {:?}", src, a, b),
        }
    }
}

#[test]
fn vm_and_interpreter_agree_on_runtime_errors() {
    let corpus = [
        "likho bhaag 1 ko 0 se",
        "likho undefined_cheez",
        "likho cheezein hissa 2",
    ];
    for src in corpus {
        assert!(interp_out(src).is_err(), "interpreter should fail: {}", src);
        assert!(vm_out(src).is_err(), "VM should fail: {}", src);
    }
}
