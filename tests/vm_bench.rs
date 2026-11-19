use boli::interpreter::Environment;
use boli::parser::parse_source;
use boli::vm::Vm;
use std::time::Instant;

fn workload(name: &str, src: &str) {
    let program = parse_source(src).unwrap_or_else(|e| panic!("{}: parse fail: {}", name, e));

    let start = Instant::now();
    let mut interp = Environment::with_limits(200_000, 32, 1_000_000);
    interp.collect_output();
    interp
        .run_program(&program)
        .unwrap_or_else(|e| panic!("{}: interp fail: {}", name, e));
    let interp_out = interp.take_output();
    let interp_time = start.elapsed();

    let start = Instant::now();
    let mut vm = Vm::new().with_max_steps(200_000);
    vm.collect_output();
    vm.run_program(&program)
        .unwrap_or_else(|e| panic!("{}: vm fail: {}", name, e));
    let vm_out = vm.take_output();
    let vm_time = start.elapsed();

    assert_eq!(interp_out, vm_out, "{}: output mismatch", name);
    println!(
        "{}: interp {:?}, vm {:?}, output lines {}",
        name,
        interp_time,
        vm_time,
        interp_out.len()
    );
}

#[test]
#[ignore]
fn vm_benchmark_smoke() {
    workload(
        "loop_1000",
        "i rakho 0\njab tak i 1000 se chhota hai to\n    jodo i i aur 1\nkhatam\nlikho i",
    );
    workload(
        "list_40",
        "cheezein rakho 1\ni rakho 0\njab tak i 40 se chhota hai to\n    cheezein rakho 1 cheezein\n    jodo i i aur 1\nkhatam\nlikho lambayi cheezein",
    );
    workload(
        "map_100",
        r#"mapa rakho map "a" 1
i rakho 1
jab tak i 100 se chhota hai to
    mapa hissa "a" rakho i
    jodo i i aur 1
khatam
likho lambayi mapa"#,
    );
    workload(
        "recursion_15",
        "kaam fib n\n    agar n 1 se chhota ya barabar hai to\n        wapas do n\n    khatam\n    ghatao a n se 1\n    ghatao b n se 2\n    wapas do jodo fib a aur fib b\nkhatam\nlikho fib 15",
    );
}
