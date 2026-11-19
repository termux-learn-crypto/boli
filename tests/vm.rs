// BOLI bytecode VM ke tests — tree-walking interpreter ke parantu pariyay (parity)
// Har test VM pe snippet chala kar expected output verify karta hai

use boli::ast::BoliError;
use boli::interpreter::Environment;
use boli::parser::parse_source;
use boli::vm::Vm;
use std::path::PathBuf;

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

fn vm_test_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
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

#[test]
fn entry_file_self_import_runs_once_in_interpreter_and_vm() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("vm_entry_self_import.hl");
    std::fs::write(
        &path,
        "likho \"ENTRY_ONCE\"\nlaao \"vm_entry_self_import.hl\"\n",
    )
    .unwrap();
    let source = std::fs::read_to_string(&path).unwrap();
    let program = parse_source(&source).unwrap();

    let mut env = Environment::new();
    env.set_source_dir(dir.clone());
    env.register_source_file(&path).unwrap();
    env.collect_output();
    env.run_program(&program).unwrap();
    let env_output = env.take_output();
    assert_eq!(
        env_output
            .iter()
            .filter(|line| line.contains("ENTRY_ONCE"))
            .count(),
        1
    );

    let mut vm = Vm::new();
    vm.set_source_dir(dir);
    vm.register_source_file(&path).unwrap();
    vm.collect_output();
    vm.run_program(&program).unwrap();
    let vm_output = vm.take_output();
    assert_eq!(
        vm_output
            .iter()
            .filter(|line| line.contains("ENTRY_ONCE"))
            .count(),
        1
    );
}

fn run_in_dir(
    src: &str,
    dir: PathBuf,
) -> (
    Result<Vec<String>, BoliError>,
    Result<Vec<String>, BoliError>,
) {
    let program = parse_source(src).unwrap();

    let mut env = Environment::new();
    env.set_source_dir(dir.clone());
    env.collect_output();
    let interp = env.run_program(&program).map(|_| env.take_output());

    let mut vm = Vm::new();
    vm.set_source_dir(dir);
    vm.collect_output();
    let vm = vm.run_program(&program).map(|_| vm.take_output());

    (interp, vm)
}

#[test]
fn vm_range_body_ginti_shrink_does_not_panic() {
    let src = "1 se 3 tak\n    ginti rakho khaali\n    ruk\nkhatam\nlikho \"done\"";
    assert_eq!(vm_out(src).unwrap(), vec!["done"]);
    assert_eq!(interp_out(src).unwrap(), vec!["done"]);
}

#[test]
fn vm_dead_branch_kaam_and_import_parity() {
    let dir = vm_test_dir("vm_dead_branch");
    std::fs::write(
        dir.join("vm_dead_module.hl"),
        "kaam dead_imported\naap 1\nkhatam\n",
    )
    .unwrap();

    let kaam_src = "agar galat sahi barabar hai to\n    kaam dead_kaam\n        wapas do 1\n    khatam\nkhatam\nlikho dead_kaam";
    assert!(interp_out(kaam_src).is_err());
    assert!(vm_out(kaam_src).is_err());

    let import_src = "agar galat sahi barabar hai to\n    laao \"vm_dead_module.hl\"\nkhatam\nlikho dead_imported";
    let (interp, vm) = run_in_dir(import_src, dir);
    assert!(interp.is_err(), "interpreter should fail: {:?}", interp);
    assert!(vm.is_err(), "VM should fail: {:?}", vm);
}

#[test]
fn vm_deep_value_errors_instead_of_stack_overflow() {
    let mut src = String::from("cheezein rakho 1\n");
    for _ in 0..80 {
        src.push_str("cheezein rakho 1 cheezein\n");
    }
    let err = vm_out(&src).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("value bahut gehra hai") && msg.contains("BOLI_MAX_VALUE_DEPTH"),
        "unexpected error: {}",
        msg
    );
}

#[test]
fn vm_top_level_return_matches_interpreter() {
    let src = "wapas do 7";
    let interp = interp_out(src).unwrap_err();
    let vm = vm_out(src).unwrap_err();
    assert_eq!(interp.to_string(), vm.to_string());
    assert!(
        vm.to_string()
            .contains("'wapas do' sirf kaam ke andar use hota hai")
    );
}

#[test]
fn vm_uncalled_kaam_with_flow_statement_compiles() {
    let src = "kaam f\n    ruk\nkhatam\nlikho \"ok\"";
    assert_eq!(interp_out(src).unwrap(), vec!["ok"]);
    assert_eq!(vm_out(src).unwrap(), vec!["ok"]);

    let called = "kaam f\n    ruk\nkhatam\nf";
    assert!(interp_out(called).is_err());
    assert!(vm_out(called).is_err());
}

#[test]
fn vm_condition_error_line_matches_interpreter() {
    let src = "likho \"start\"\nagar \"abc\" 5 se bada hai to\n    likho \"x\"\nkhatam";
    let interp = interp_out(src).unwrap_err();
    let vm = vm_out(src).unwrap_err();
    assert_eq!(interp.line, Some(2), "interp: {}", interp);
    assert_eq!(vm.line, Some(2), "vm: {}", vm);
    assert_eq!(interp.to_string(), vm.to_string());
}

#[test]
fn vm_missing_name_hint_matches_interpreter() {
    let src = "naam rakho 1\nlikho nam";
    let interp = interp_out(src).unwrap_err();
    let vm = vm_out(src).unwrap_err();
    let msg = vm.to_string();
    assert!(
        msg.contains("naam 'nam' nahi mila. Shayad 'naam' likhna tha?"),
        "unexpected error: {}",
        msg
    );
    assert_eq!(interp.to_string(), msg);
}

#[test]
fn vm_param_kaam_as_value_matches_interpreter() {
    let src = "kaam f x\n    wapas do x\nkhatam\nlikho f";
    let interp = interp_out(src).unwrap_err();
    let vm = vm_out(src).unwrap_err();
    assert_eq!(interp.to_string(), vm.to_string());
    assert!(vm.to_string().contains("seedha call karo"));
}

#[test]
fn vm_import_runs_once_defines_kaams_and_wraps_errors() {
    let dir = vm_test_dir("vm_import_runtime");
    std::fs::write(
        dir.join("vm_runtime_module.hl"),
        "likho \"MODULE_ONCE\"\nkaam imported_kaam\n    wapas do 42\nkhatam\n",
    )
    .unwrap();

    let src = "laao \"vm_runtime_module.hl\"\nlaao \"vm_runtime_module.hl\"\nlikho imported_kaam";
    let (interp, vm) = run_in_dir(src, dir.clone());
    assert_eq!(interp.unwrap(), vec!["MODULE_ONCE", "42"]);
    assert_eq!(vm.unwrap(), vec!["MODULE_ONCE", "42"]);

    std::fs::write(dir.join("vm_inner_bad.hl"), "likho undefined_cheez\n").unwrap();
    std::fs::write(dir.join("vm_outer_bad.hl"), "laao \"vm_inner_bad.hl\"\n").unwrap();
    let (interp_err, vm_err) = run_in_dir("laao \"vm_outer_bad.hl\"", dir);
    assert!(interp_err.is_err());
    assert!(vm_err.is_err());
    let msg = vm_err.unwrap_err().to_string();
    assert!(msg.contains("file 'vm_outer_bad.hl' me error:"), "{}", msg);
    assert!(msg.contains("file 'vm_inner_bad.hl' me error:"), "{}", msg);
}
