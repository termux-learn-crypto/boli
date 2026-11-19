use boli::ast::BoliError;
use boli::interpreter::Environment;
use boli::parser::parse_source;
use boli::vm::Vm;

fn next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state >> 33
}

fn generated_source(state: &mut u64) -> String {
    const TOKENS: &[&str] = &[
        "likho", "rakho", "jodo", "aur", "sahi", "galat", "khaali", "1", "2", "2.5", "\"abc\"",
        "\"", "x", "naam", "kaam", "khatam", "wapas", "do", "agar", "to", "nahi", "jab", "tak",
        "se", "3", "ginti", "ruk", "agla", "cheezein", "mapa", "map", "hissa", "lambayi", "upar",
        "text", "sankhya", "bhaag", "0", "f", "\n", "  ",
    ];
    let count = 1 + (next(state) % 48) as usize;
    let mut src = String::new();
    for _ in 0..count {
        let token = TOKENS[(next(state) as usize) % TOKENS.len()];
        src.push_str(token);
        if next(state).is_multiple_of(7) {
            src.push('\n');
        } else {
            src.push(' ');
        }
    }
    src
}

fn run_pair(src: &str, seed: u64) {
    let program = match parse_source(src) {
        Ok(program) => program,
        Err(_) => return,
    };

    let mut interp = Environment::with_limits(600, 8, 4096);
    interp.collect_output();
    let interp_result = interp
        .run_program(&program)
        .map(|_| interp.take_output())
        .map_err(|e: BoliError| e.to_string());

    let mut vm = Vm::new().with_max_steps(600);
    vm.collect_output();
    let vm_result = vm
        .run_program(&program)
        .map(|_| vm.take_output())
        .map_err(|e: BoliError| e.to_string());

    match (interp_result, vm_result) {
        (Ok(a), Ok(b)) => assert_eq!(a, b, "seed {}: output mismatch\nsource:\n{}", seed, src),
        (Err(_), Err(_)) => {}
        (a, b) => panic!(
            "seed {}: backend status mismatch\n{:?}\n{:?}\nsource:\n{}",
            seed, a, b, src
        ),
    }
}

fn fuzz(cases: u64, start_seed: u64) {
    let mut state = start_seed;
    for index in 0..cases {
        let src = generated_source(&mut state);
        run_pair(&src, start_seed.wrapping_add(index));
    }
}

fn configured_cases() -> u64 {
    std::env::var("BOLI_FUZZ_CASES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(2000)
}

#[test]
fn vm_fuzz_deterministic_smoke() {
    fuzz(configured_cases(), 0x424f_4c49_5631);
}

#[test]
#[ignore]
fn vm_fuzz_100k() {
    fuzz(100_000, 0x424f_4c49_5631);
}
