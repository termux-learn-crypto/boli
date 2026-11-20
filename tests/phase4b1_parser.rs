use boli::ast::BoliError;
use boli::interpreter::Environment;
use boli::parser::parse_source;
use std::fs;
use std::path::PathBuf;

fn parser_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("selfhost/parser")
}

fn run_boli(source: &str) -> Result<Vec<String>, String> {
    let program = parse_source(source).map_err(|error: BoliError| error.to_string())?;
    let mut env = Environment::new();
    env.set_source_dir(parser_dir());
    env.collect_output();
    env.run_program(&program)
        .map(|_| env.take_output())
        .map_err(|error: BoliError| error.to_string())
}

fn after_marker<'a>(lines: &'a [String], marker: &str) -> &'a str {
    let idx = lines
        .iter()
        .position(|line| line == marker)
        .unwrap_or_else(|| panic!("missing marker {}", marker));
    lines
        .get(idx + 1)
        .unwrap_or_else(|| panic!("missing line after {}", marker))
}

#[test]
fn phase4b1_ast_constructors_cover_all_variants() {
    let out = run_boli(
        r#"
laao "ast.boli"

sp rakho map "start" 0 "end" 1 "start_line" 1 "start_col" 1 "end_line" 1 "end_col" 2
empty rakho ast_empty_list 0
n1 rakho ast_expr_number 5 sp
nt rakho ast_expr_text "abc" sp
nb rakho ast_expr_bool sahi sp
nk rakho ast_expr_khaali sp
nv rakho ast_expr_var "x" sp
na rakho ast_expr_arith "jodo" n1 n1 sp
nc rakho ast_expr_call "f" empty sp
nl rakho ast_expr_list empty sp
ni rakho ast_expr_index nv n1 sp

cd rakho ast_cond_cmp n1 "barabar" nt sp
ca rakho ast_cond_aur cd cd sp
cy rakho ast_cond_ya cd cd sp

s1 rakho ast_stmt_likho n1 sp
s2 rakho ast_stmt_rakho "x" n1 sp
s3 rakho ast_stmt_pucho "x" "Sawal: " sp
s4 rakho ast_stmt_arith_assign "x" "jodo" n1 n1 sp
body rakho jodega empty s1
e1 rakho ast_stmt_agar cd body khaali sp
s5 rakho ast_stmt_range_loop n1 n1 body sp
s6 rakho ast_stmt_while_loop cd body sp
params rakho jodega empty "a"
s7 rakho ast_stmt_kaam_def "f" params body sp
s8 rakho ast_stmt_kaam_call "f" empty sp
s9 rakho ast_stmt_wapas n1 sp
s10 rakho ast_stmt_wapas khaali sp
s11 rakho ast_stmt_ruk sp
s12 rakho ast_stmt_agla sp
s13 rakho ast_stmt_laao "lib.hl" sp
pair rakho empty
pair rakho jodega pair n1
pair rakho jodega pair n1
pairs rakho jodega empty pair
s14 rakho ast_stmt_list_assign "x" n1 n1 sp
s15 rakho ast_stmt_map_rakho "m" pairs sp

statements rakho jodega empty s1
statements rakho jodega statements s2
statements rakho jodega statements s3
statements rakho jodega statements s4
statements rakho jodega statements e1
statements rakho jodega statements s5
statements rakho jodega statements s6
statements rakho jodega statements s7
statements rakho jodega statements s8
statements rakho jodega statements s9
statements rakho jodega statements s10
statements rakho jodega statements s11
statements rakho jodega statements s12
statements rakho jodega statements s13
statements rakho jodega statements s14
statements rakho jodega statements s15
program rakho ast_program statements

likho "EXPR"
likho ast_dump n1
likho ast_dump nt
likho ast_dump nb
likho ast_dump nk
likho ast_dump nv
likho ast_dump na
likho ast_dump nc
likho ast_dump nl
likho ast_dump ni
likho "COND"
likho ast_dump cd
likho ast_dump ca
likho ast_dump cy
likho "STMT"
likho ast_dump s1
likho ast_dump s2
likho ast_dump s3
likho ast_dump s4
likho ast_dump e1
likho ast_dump s5
likho ast_dump s6
likho ast_dump s7
likho ast_dump s8
likho ast_dump s9
likho ast_dump s10
likho ast_dump s11
likho ast_dump s12
likho ast_dump s13
likho ast_dump s14
likho ast_dump s15
likho "PROGRAM"
likho ast_dump program
"#,
    )
    .unwrap();

    let expr_start = out.iter().position(|l| l == "EXPR").unwrap() + 1;
    let cond_start = out.iter().position(|l| l == "COND").unwrap() + 1;
    let stmt_start = out.iter().position(|l| l == "STMT").unwrap() + 1;
    let program_start = out.iter().position(|l| l == "PROGRAM").unwrap() + 1;

    let exprs = &out[expr_start..cond_start - 1];
    let conds = &out[cond_start..stmt_start - 1];
    let stmts = &out[stmt_start..program_start - 1];

    assert_eq!(exprs.len(), 9);
    assert_eq!(conds.len(), 3);
    assert_eq!(stmts.len(), 16);

    let expr_tags = [
        "Number|1|1|S(0,1,1,1,1,2)|value=5",
        "Text|1|1|S(0,1,1,1,1,2)|value=abc",
        "Bool|1|1|S(0,1,1,1,1,2)|value=sahi",
        "Khaali|1|1|S(0,1,1,1,1,2)",
        "Var|1|1|S(0,1,1,1,1,2)|name=x",
        "Arith|1|1|S(0,1,1,1,1,2)|op=jodo",
        "Call|1|1|S(0,1,1,1,1,2)|name=f|args=<0>[]",
        "List|1|1|S(0,1,1,1,1,2)|items=<0>[]",
        "Index|1|1|S(0,1,1,1,1,2)|target=(Var|",
    ];
    for (actual, expected) in exprs.iter().zip(expr_tags.iter()) {
        assert!(
            actual.starts_with(expected),
            "expr dump mismatch: {}",
            actual
        );
    }

    assert!(conds[0].starts_with("Cmp|1|1|S(0,1,1,1,1,2)|left=(Number|"));
    assert!(conds[1].starts_with("Aur|1|1|S(0,1,1,1,1,2)|left=(Cmp|"));
    assert!(conds[2].starts_with("Ya|1|1|S(0,1,1,1,1,2)|left=(Cmp|"));

    let stmt_tags = [
        "Likho|",
        "Rakho|",
        "Pucho|",
        "ArithAssign|",
        "Agar|",
        "RangeLoop|",
        "WhileLoop|",
        "KaamDef|",
        "KaamCall|",
        "Wapas|",
        "Wapas|",
        "Ruk|",
        "Agla|",
        "Laao|",
        "ListAssign|",
        "MapRakho|",
    ];
    for (actual, expected) in stmts.iter().zip(stmt_tags.iter()) {
        assert!(
            actual.starts_with(expected),
            "stmt dump mismatch: {}",
            actual
        );
    }
    assert!(stmts[4].contains("|else_block=khaali"));
    assert!(stmts[10].contains("|value=khaali"));
    assert!(stmts[15].starts_with("MapRakho|"));
    assert!(stmts[15].contains("|pairs=<1>["));

    let program = out.last().unwrap();
    assert!(program.starts_with("Program|1|1|S(0,0,1,1,1,1)|statements=<16>["));
}

#[test]
fn phase4b1_token_cursor_peek_next_and_skip_newlines() {
    let out = run_boli(
        r#"
laao "token_stream.boli"

t1 rakho lexer_token "Number" "5" 1 1 0 1 1 1 1 2
t2 rakho lexer_token "Newline" "" 1 2 1 2 1 2 1 3
t3 rakho lexer_token "Ident" "x" 2 1 2 3 2 1 2 2
tokens rakho stream_empty_list 0
tokens rakho jodega tokens t1
tokens rakho jodega tokens t2
tokens rakho jodega tokens t3
state rakho parser_state_new tokens 64 128

likho "STATE0"
likho parser_state_text state
p0 rakho token_stream_peek state
likho lexer_token_text p0
p2 rakho token_stream_peek_at state 2
likho lexer_token_text p2
n rakho token_stream_next state
likho lexer_token_text n hissa "token"
likho parser_state_text n hissa "state"
skipped rakho token_stream_skip_newlines n hissa "state"
likho parser_state_text skipped
p3 rakho token_stream_peek skipped
likho lexer_token_text p3
"#,
    )
    .unwrap();

    assert_eq!(
        after_marker(&out, "STATE0"),
        "STATE|pos=1|depth=0|max_depth=64|max_condition_depth=128|condition_nodes=0|errors=0"
    );
    assert_eq!(out.get(2).unwrap(), "Number|5|1|1|0|1|1|1|1|2");
    assert_eq!(out.get(3).unwrap(), "Ident|x|2|1|2|3|2|1|2|2");
    assert_eq!(out.get(4).unwrap(), "Number|5|1|1|0|1|1|1|1|2");
    assert_eq!(
        out.get(5).unwrap(),
        "STATE|pos=2|depth=0|max_depth=64|max_condition_depth=128|condition_nodes=0|errors=0"
    );
    assert_eq!(
        out.get(6).unwrap(),
        "STATE|pos=3|depth=0|max_depth=64|max_condition_depth=128|condition_nodes=0|errors=0"
    );
    assert_eq!(out.get(7).unwrap(), "Ident|x|2|1|2|3|2|1|2|2");
}

#[test]
fn phase4b1_token_cursor_expect_and_synthetic_eof() {
    let out = run_boli(
        r#"
laao "token_stream.boli"

empty rakho stream_empty_list 0
state rakho parser_state_new empty 64 128
eof rakho token_stream_peek state
eof_flag rakho token_stream_at_eof state

likho "EOF"
likho lexer_token_text eof
likho text eof_flag

bad rakho token_stream_expect state "To" "to"
likho "EXPECT"
likho text bad hissa "ok"
likho parser_error_text bad hissa "error"

le rakho token_stream_expect_line_end state
likho text le hissa "ok"

id rakho token_stream_expect_ident state
likho text id hissa "ok"
likho parser_error_text id hissa "error"

likho "ATOM"
atom1 rakho token_stream_atom_start "Hissa"
atom2 rakho token_stream_atom_start "Kaam"
atom3 rakho token_stream_atom_start "Number"
likho text atom1
likho text atom2
likho text atom3
"#,
    )
    .unwrap();

    assert_eq!(after_marker(&out, "EOF"), "Eof||1|1|0|0|1|1|1|1");
    assert_eq!(out.get(2).unwrap(), "sahi");
    assert_eq!(after_marker(&out, "EXPECT"), "galat");
    assert_eq!(out.get(4).unwrap(), "galat");
    assert_eq!(
        out.get(5).unwrap(),
        "parser_expected_token|yahan 'to' chahiye tha, par 'file ka ant' mila|1|1|0|0|1|1|1|1"
    );
    assert_eq!(out.get(6).unwrap(), "sahi");
    assert_eq!(out.get(7).unwrap(), "galat");
    assert_eq!(
        out.get(8).unwrap(),
        "parser_expected_ident|yahan naam chahiye tha, par 'file ka ant' mila|1|1|0|0|1|1|1|1"
    );
    assert_eq!(after_marker(&out, "ATOM"), "galat");
    assert_eq!(out.get(11).unwrap(), "sahi");
    assert_eq!(out.get(12).unwrap(), "sahi");
}

#[test]
fn phase4b1_parser_skeleton_empty_program_and_not_implemented() {
    let out = run_boli(
        r#"
laao "../lexer/errors.boli"
seed rakho 0 0
empty rakho hatao seed 1
empty rakho hatao empty 1
lex_result rakho lexer_ok empty
laao "run.boli"
"#,
    )
    .unwrap();
    assert_eq!(
        out.first().unwrap(),
        "RESULT|ok=sahi|ast=Program|1|1|S(0,0,1,1,1,1)|statements=<0>[]|error=khaali|state=STATE|pos=1|depth=0|max_depth=64|max_condition_depth=128|condition_nodes=0|errors=0"
    );

    let out = run_boli(
        r#"
laao "../lexer/errors.boli"
t rakho map "kind" "Likho" "value" "" "line" 1 "col" 1 "start_byte" 0 "end_byte" 5 "start_line" 1 "start_col" 1 "end_line" 1 "end_col" 6
seed rakho 0 0
empty rakho hatao seed 1
empty rakho hatao empty 1
tokens rakho jodega empty t
lex_result rakho lexer_ok tokens
laao "run.boli"
"#,
    )
    .unwrap();
    assert_eq!(
        out.first().unwrap(),
        "RESULT|ok=galat|ast=khaali|error=parser_not_implemented|statement parser abhi tayyar nahi hai|1|1|0|5|1|1|1|6|state=STATE|pos=1|depth=0|max_depth=64|max_condition_depth=128|condition_nodes=0|errors=0"
    );
}

#[test]
fn phase4b1_parser_propagates_lexer_error_unchanged() {
    let out = run_boli(
        r#"
laao "../lexer/errors.boli"
err rakho lexer_make_error "source_too_large" "bad" 1 1 0 1 1 1 1 2
lex_result rakho map "ok" galat "tokens" khaali "state" khaali "error" err
laao "run.boli"
"#,
    )
    .unwrap();
    assert_eq!(
        out.first().unwrap(),
        "RESULT|ok=galat|ast=khaali|error=source_too_large|bad|1|1|0|1|1|1|1|2|state=khaali"
    );
}

#[test]
fn phase4b1_parser_does_not_import_lexer_internals() {
    let dir = parser_dir();
    for name in [
        "ast.boli",
        "errors.boli",
        "token_stream.boli",
        "parser.boli",
        "run.boli",
    ] {
        let source = fs::read_to_string(dir.join(name)).unwrap();
        for forbidden in [
            "laao \"lexer.boli\"",
            "laao \"reader.boli\"",
            "laao \"unicode.boli\"",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} imports forbidden module {}",
                name,
                forbidden
            );
        }
    }
    let stream = fs::read_to_string(dir.join("token_stream.boli")).unwrap();
    assert!(stream.contains("laao \"../lexer/token.boli\""));
}
