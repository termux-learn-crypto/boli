// BOLI ke regression tests

use boli::ast::{BoliError, Expr, Stmt};
use boli::interpreter::Environment;
use boli::lexer::{Tok, lex};
use boli::parser::parse_source;

// helper: source chalao aur captured lines lo
fn run_out(src: &str) -> Result<Vec<String>, BoliError> {
    let program = parse_source(src)?;
    let mut env = Environment::new();
    env.collect_output();
    env.run_program(&program)?;
    Ok(env.take_output())
}

// helper: same env me do program chalao (state persistence ke liye)
fn run_two(src1: &str, src2: &str) -> (Result<(), BoliError>, Result<(), BoliError>, Vec<String>) {
    let p1 = parse_source(src1).unwrap();
    let p2 = parse_source(src2).unwrap();
    let mut env = Environment::new();
    env.collect_output();
    let r1 = env.run_program(&p1);
    let r2 = env.run_program(&p2);
    (r1, r2, env.take_output())
}

// ---------- Lexer tests ----------

#[test]
fn lexer_simple_tokens() {
    let tokens = lex("likho jodo 5 aur 3").unwrap();
    let toks: Vec<Tok> = tokens.into_iter().map(|t| t.tok).collect();
    assert_eq!(
        toks,
        vec![
            Tok::Likho,
            Tok::Jodo,
            Tok::Number(5.0),
            Tok::Aur,
            Tok::Number(3.0),
            Tok::Eof
        ]
    );
}

#[test]
fn lexer_negative_number() {
    let tokens = lex("jodo jawab -5 aur 3").unwrap();
    let toks: Vec<Tok> = tokens.into_iter().map(|t| t.tok).collect();
    assert_eq!(
        toks,
        vec![
            Tok::Jodo,
            Tok::Ident("jawab".into()),
            Tok::Number(-5.0),
            Tok::Aur,
            Tok::Number(3.0),
            Tok::Eof
        ]
    );
}

#[test]
fn lexer_string_escapes() {
    let tokens = lex("likho \"a\\nb\"").unwrap();
    assert!(matches!(&tokens[0].tok, Tok::Likho));
    assert!(matches!(&tokens[1].tok, Tok::Text(s) if s == "a\nb"));
}

#[test]
fn lexer_rejects_symbols() {
    let err = lex("likho ;").unwrap_err();
    assert!(err.to_string().contains("';' nahi chahiye"));
}

#[test]
fn lexer_unicode_identifiers() {
    let tokens = lex("नमस्ते rakho \"x\"").unwrap();
    assert!(matches!(&tokens[0].tok, Tok::Ident(s) if s == "नमस्ते"));
}

// ---------- Parser tests ----------

#[test]
fn parser_rakho_single_value() {
    let stmts = parse_source("x rakho 5").unwrap();
    match &stmts[0] {
        Stmt::Rakho { name, value, .. } => {
            assert_eq!(name, "x");
            assert!(matches!(value, Expr::Number(5.0, _)));
        }
        _ => panic!("Rakho statement expect kiya tha"),
    }
}

#[test]
fn parser_rakho_multiple_values_makes_list() {
    let stmts = parse_source("cheezein rakho 1 2 3").unwrap();
    match &stmts[0] {
        Stmt::Rakho {
            value: Expr::List(items, _),
            ..
        } => assert_eq!(items.len(), 3),
        other => panic!("List expect kiya tha, mila {:?}", other),
    }
}

#[test]
fn parser_ruk_agla_are_statements() {
    let stmts = parse_source("ruk\nagla").unwrap();
    assert!(matches!(stmts[0], Stmt::Ruk(_)));
    assert!(matches!(stmts[1], Stmt::Agla(_)));
}

#[test]
fn parser_hissa_expression() {
    let stmts = parse_source("y rakho cheezein hissa 1").unwrap();
    match &stmts[0] {
        Stmt::Rakho {
            value: Expr::Index(_, idx, _),
            ..
        } => {
            assert!(matches!(**idx, Expr::Number(1.0, _)));
        }
        other => panic!("Index expression expect kiya tha, mila {:?}", other),
    }
}

#[test]
fn parser_list_assign_syntax() {
    let stmts = parse_source("cheezein hissa 2 rakho 99").unwrap();
    assert!(
        matches!(&stmts[0], Stmt::ListAssign { target, index, .. } if target == "cheezein" && matches!(index, Expr::Number(2.0, _)))
    );
}

#[test]
fn parser_kaam_call_args_are_flat() {
    // BUG-008 fix: f a b => f(a, b), f(a(b)) nahi
    let stmts = parse_source("f a b c").unwrap();
    match &stmts[0] {
        Stmt::KaamCall { name, args, .. } => {
            assert_eq!(name, "f");
            assert_eq!(args.len(), 3);
        }
        _ => panic!("KaamCall expect kiya tha"),
    }
}

#[test]
fn parser_aur_ya_precedence_ast() {
    // ya-level: comparison a ya (comparison b aur c)
    let stmts = parse_source("agar a sahi barabar hai ya b galat barabar hai aur c galat barabar hai to\nlikho \"x\"\nkhatam").unwrap();
    if let Stmt::Agar { cond, .. } = &stmts[0] {
        assert!(
            matches!(cond, boli::ast::Cond::Ya(_, _, _)),
            "root Ya hona chahiye, mila {:?}",
            cond
        );
        if let boli::ast::Cond::Ya(_, right, _) = cond {
            assert!(
                matches!(**right, boli::ast::Cond::Aur(_, _, _)),
                "right side Aur hona chahiye, mila {:?}",
                right
            );
        }
    } else {
        panic!("Agar statement expect kiya tha");
    }
}

// ---------- Interpreter: arithmetic & basic output ----------

#[test]
fn eval_likho_jodo_inline() {
    assert_eq!(run_out("likho jodo 5 aur 3").unwrap(), vec!["8"]);
}

#[test]
fn eval_rakho_and_reuse() {
    assert_eq!(
        run_out("naam rakho \"Vilas\"\nlikho naam").unwrap(),
        vec!["Vilas"]
    );
}

#[test]
fn eval_all_arith_forms() {
    assert_eq!(run_out("jodo a 5 aur 3\nghatao b 10 se 4\nguna c 5 ko 2 se\nbhaag d 10 ko 4 se\nlikho a\nlikho b\nlikho c\nlikho d").unwrap(), vec!["8", "6", "10", "2.5"]);
}

#[test]
fn eval_div_by_zero_error_message() {
    let err = run_out("bhaag jawaab 10 ko 0 se").unwrap_err();
    assert!(err.to_string().contains("zero se bhaag nahi kar sakte"));
}

#[test]
fn eval_unicode_strings() {
    assert_eq!(run_out("likho \"नमस्ते dunia\"").unwrap(), vec!["नमस्ते dunia"]);
}

#[test]
fn eval_negative_numbers() {
    assert_eq!(run_out("likho jodo -5 aur 3").unwrap(), vec!["-2"]);
    assert_eq!(run_out("likho jodo -10 aur -5").unwrap(), vec!["-15"]);
}

// ---------- Interpreter: conditionals ----------

#[test]
fn agar_then_and_else() {
    assert_eq!(
        run_out("agar 5 3 se bada hai to\nlikho \"bada\"\nnahi to\nlikho \"chhota\"\nkhatam")
            .unwrap(),
        vec!["bada"]
    );
    assert_eq!(
        run_out("agar 2 3 se bada hai to\nlikho \"bada\"\nnahi to\nlikho \"chhota\"\nkhatam")
            .unwrap(),
        vec!["chhota"]
    );
}

#[test]
fn agar_nested() {
    let out = run_out("agar 5 3 se bada hai to\nagar 8 9 se bada hai to\nlikho \"b\"\nnahi to\nlikho \"c\"\nkhatam\nnahi to\nlikho \"a\"\nkhatam").unwrap();
    assert_eq!(out, vec!["c"]);
}

#[test]
fn agar_string_barabar() {
    assert_eq!(run_out("naam rakho \"jodo\"\nagar naam \"jodo\" barabar hai to\nlikho \"match\"\nnahi to\nlikho \"no\"\nkhatam").unwrap(), vec!["match"]);
}

#[test]
fn agar_aur_ya_precedence() {
    // a ya b aur c  =>  a ya (b aur c); a=sahi, b=galat, c=galat  => sahi -> then
    // a == sahi hai, b aur c dono galat hain
    let out = run_out("a rakho sahi\nb rakho galat\nc rakho galat\nagar a sahi barabar hai ya b sahi barabar hai aur c sahi barabar hai to\nlikho \"ya wins\"\nnahi to\nlikho \"aur wins\"\nkhatam").unwrap();
    assert_eq!(out, vec!["ya wins"]);
}

// ---------- Interpreter: loops ----------

#[test]
fn range_loop_basic() {
    assert_eq!(
        run_out("1 se 3 tak\nlikho ginti\nkhatam").unwrap(),
        vec!["1", "2", "3"]
    );
}

#[test]
fn range_loop_negative() {
    assert_eq!(
        run_out("-2 se 1 tak\nlikho ginti\nkhatam").unwrap(),
        vec!["-2", "-1", "0", "1"]
    );
}

#[test]
fn range_loop_empty() {
    assert_eq!(
        run_out("5 se 1 tak\nlikho ginti\nkhatam").unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn range_loop_large_integers_exact() {
    // 2^52 se 2^52+2 tak sab exact represent hote hain
    let out = run_out("4503599627370496 se 4503599627370498 tak\nlikho ginti\nkhatam").unwrap();
    assert_eq!(
        out,
        vec!["4503599627370496", "4503599627370497", "4503599627370498"]
    );
}

#[test]
fn range_loop_huge_terminates_via_steps_limit() {
    let program = parse_source("1 se 1000000000000 tak\nkhatam").unwrap();
    let mut env = Environment::with_limits(200, 64, 1024 * 1024);
    let err = env.run_program(&program).unwrap_err();
    assert!(err.to_string().contains("steps"));
}

#[test]
fn while_loop_basic() {
    assert_eq!(run_out("ginti rakho 1\njab tak ginti 3 se chhota ya barabar hai to\nlikho ginti\njodo ginti ginti aur 1\nkhatam").unwrap(), vec!["1", "2", "3"]);
}

#[test]
fn ruk_breaks_range_loop_real() {
    assert_eq!(
        run_out("1 se 5 tak\nagar ginti 3 se bada hai to\nruk\nkhatam\nlikho ginti\nkhatam")
            .unwrap(),
        vec!["1", "2", "3"]
    );
}

#[test]
fn agla_skips_rest_of_iteration() {
    assert_eq!(
        run_out("1 se 5 tak\nagar ginti 3 barabar hai to\nagla\nkhatam\nlikho ginti\nkhatam")
            .unwrap(),
        vec!["1", "2", "4", "5"]
    );
}

#[test]
fn ruk_outside_loop_is_error() {
    let err = run_out("ruk").unwrap_err();
    assert!(err.to_string().contains("'ruk' sirf loop ke andar"));
}

#[test]
fn while_loop_error_restores_state() {
    // error ke baad same env me naya program chalna chahiye
    let (r1, r2, out) = run_two("1 se 3 tak\nlikho xyz\nkhatam", "likho \"after-error\"");
    assert!(r1.is_err());
    assert!(r2.is_ok());
    assert_eq!(out, vec!["after-error"]);
}

#[test]
fn loop_scope_removed_after_success_and_error() {
    // success: ginti loop ke baad nahi honi chahiye
    let (r1, r2, _) = run_two("1 se 3 tak\nkhatam", "likho ginti");
    assert!(r1.is_ok());
    assert!(r2.is_err());
    // error wala path bhi same
    let (r3, r4, _) = run_two("1 se 3 tak\nlikho xyz\nkhatam", "likho ginti");
    assert!(r3.is_err());
    assert!(r4.is_err());
}

// ---------- Interpreter: functions ----------

#[test]
fn kaam_call_wapas_do_via_rakho() {
    // jodo_kaam jaisi kaam se wapas value milti hai
    assert_eq!(run_out("kaam jodo_kaam a b\njodo jawab a aur b\nwapas do jawab\nkhatam\nsum_n rakho jodo_kaam 20 8\nlikho sum_n").unwrap(), vec!["28"]);
}

#[test]
fn kaam_call_zero_args() {
    assert_eq!(
        run_out("kaam jawab_do\nwapas do 42\nkhatam\nlikho jawab_do").unwrap(),
        vec!["42"]
    );
}

#[test]
fn kaam_call_variable_args_flat() {
    // BUG-001 fix: 2 variable arguments seedha pass hote hain
    assert_eq!(
        run_out("kaam two_args x y\nlikho x\nlikho y\nkhatam\ntwo_args 7 9").unwrap(),
        vec!["7", "9"]
    );
}

#[test]
fn kaam_nested_via_variable() {
    assert_eq!(
        run_out("kaam inner\nwapas do 10\nkhatam\nt rakho inner\nlikho t").unwrap(),
        vec!["10"]
    );
}

#[test]
fn kaam_expression_as_argument() {
    assert_eq!(
        run_out("kaam echo x\nlikho x\nkhatam\necho jodo 2 aur 3").unwrap(),
        vec!["5"]
    );
}

#[test]
fn kaam_wrong_arity_error() {
    let err = run_out("kaam need2 a b\nwapas do a\nkhatam\nneed2 1").unwrap_err();
    assert!(err.to_string().contains("2 cheezein chahiye"));
}

#[test]
fn kaam_infinite_recursion_error() {
    let err = run_out("kaam inf x\ninf x\nkhatam\ninf 1").unwrap_err();
    assert!(err.to_string().contains("gehra ho gaya"));
}

#[test]
fn global_not_overwritten_by_kaam_body() {
    // BUG-002 related: kaam me rakho bahar ka global nahi badalna chahiye
    assert_eq!(
        run_out("x rakho 99\nkaam tweak\nx rakho 7\nkhatam\ntweak\nlikho x").unwrap(),
        vec!["99"]
    );
}

#[test]
fn kaam_can_read_global() {
    assert_eq!(
        run_out("y rakho 5\nkaam reader\nlikho y\nkhatam\nreader").unwrap(),
        vec!["5"]
    );
}

// ---------- Interpreter: lists ----------

#[test]
fn list_create_and_print() {
    assert_eq!(
        run_out("cheezein rakho 1 2 3\nlikho cheezein").unwrap(),
        vec!["1, 2, 3"]
    );
}

#[test]
fn list_index_hissa() {
    assert_eq!(
        run_out("cheezein rakho 10 20 30\nlikho cheezein hissa 2").unwrap(),
        vec!["20"]
    );
}

#[test]
fn list_assign_hissa() {
    assert_eq!(
        run_out("cheezein rakho 1 2 3\ncheezein hissa 2 rakho 99\nlikho cheezein").unwrap(),
        vec!["1, 99, 3"]
    );
}

#[test]
fn list_concat_with_jodo() {
    assert_eq!(
        run_out("a rakho 1 2\nb rakho 3 4\nc rakho jodo a aur b\nlikho c").unwrap(),
        vec!["1, 2, 3, 4"]
    );
}

#[test]
fn list_index_out_of_bounds_error() {
    let err = run_out("cheezein rakho 1 2\nlikho cheezein hissa 5").unwrap_err();
    assert!(err.to_string().contains("sirf 2 hisse"));
}

// ---------- Interpreter: steps limit ----------

#[test]
fn steps_limit_stops_infinite_loop() {
    let program = parse_source("jab tak 1 0 se bada hai to\nkhatam").unwrap();
    let mut env = Environment::with_limits(150, 64, 1024 * 1024);
    let err = env.run_program(&program).unwrap_err();
    assert!(err.to_string().contains("steps"));
}

// ---------- BUG 5: non-finite numeric input ----------

#[test]
fn lexer_rejects_overflowing_literal() {
    // f64 range se bahar ka literal "inf" nahi banana chahiye
    let huge = "1".repeat(400);
    let src = format!("likho {}", huge);
    let err = run_out(&src).unwrap_err();
    assert!(err.to_string().contains("bahut bada"));
}

#[test]
fn lexer_accepts_large_finite_literal() {
    let src = "likho 99999999999999999999999999999999999999";
    assert!(run_out(src).is_ok());
}

#[test]
fn lexer_accepts_normal_decimals() {
    assert_eq!(run_out("likho jodo 99.50 aur 0.50").unwrap(), vec!["100"]);
}

#[test]
fn arith_overflow_is_runtime_error_not_inf() {
    // y: 1e15 -> 1e30 -> 1e60 -> 1e120 -> 1e240 -> 1e480 (overflow)
    let src = "y rakho 1000000000000000\nguna y y ko y se\nguna y y ko y se\nguna y y ko y se\nguna y y ko y se\nguna y y ko y se\nlikho y";
    let err = run_out(src).unwrap_err();
    assert!(err.to_string().contains("finite nahi raha"));
}

#[test]
fn arith_normal_finite_results_still_work() {
    assert_eq!(run_out("likho jodo 100 aur 200").unwrap(), vec!["300"]);
    assert_eq!(
        run_out("bhaag jawaab 10 ko 4 se\nlikho jawaab").unwrap(),
        vec!["2.5"]
    );
}

// ---------- misc regression ----------

#[test]
fn unknown_variable_line_number() {
    let err = run_out("likho naam").unwrap_err();
    assert!(err.to_string().contains("line 1"));
}

// ---------- depth limit regression (stack overflow se bachao) ----------

#[test]
fn deeply_nested_agar_errors_not_crashes() {
    let src = "agar 1 1 barabar hai to\n".repeat(200) + "likho 1\n" + &"khatam\n".repeat(200);
    assert!(parse_source(&src).is_err());
}

#[test]
fn deeply_nested_expression_errors_not_crashes() {
    let src = format!("likho {}1", "jodo 1 aur ".repeat(150));
    assert!(parse_source(&src).is_err());
}

#[test]
fn deeply_nested_hissa_chain_errors_not_crashes() {
    let mut src = String::from("naam rakho \"Vilas\"\nlikho naam");
    for _ in 0..300 {
        src.push_str(" hissa 1");
    }
    let program = parse_source(&src).unwrap();
    let mut env = Environment::new();
    env.collect_output();
    assert!(env.run_program(&program).is_err());
}

#[test]
fn moderately_nested_code_still_works() {
    let src = "agar 1 1 barabar hai to\n".repeat(20) + "likho 42\n" + &"khatam\n".repeat(20);
    assert_eq!(run_out(&src).unwrap(), vec!["42"]);
}
