# BOLI — Phase 4B.1 Report: Self-hosted Parser Foundation

**Status:** Phase 4B.1 complete. Grammar implementation / Phase 4B.2 was not started.

## 1. Scope

Phase 4B.1 added only the foundation needed before writing the self-hosted parser grammar:

- tagged-map AST for every current Rust AST variant;
- canonical AST text dump for golden/differential tests;
- parser result/error maps with the full token-span field set;
- state-threaded token cursor over the frozen `selfhost/lexer/token.boli` contract;
- honest parser skeleton: empty program succeeds, any other token fails with `parser_not_implemented`, lexer errors pass through unchanged;
- Rust test harness that executes the BOLI modules instead of only grepping their source.

No parser grammar, Rust parser, lexer, VM, CLI, syntax, or token contract was changed.

## 2. Files added

- `selfhost/parser/ast.boli` — AST constructors, zero span, program node, canonical dump helpers.
- `selfhost/parser/errors.boli` — `parser_ok`, `parser_fail`, parser error map, error text, empty-list helper.
- `selfhost/parser/token_stream.boli` — 1-based token cursor, synthetic EOF, newline skipping, `expect*`, atom/ident predicates, state text.
- `selfhost/parser/parser.boli` — parser skeleton and result text helper.
- `selfhost/parser/run.boli` — thin runner: expects an existing `lex_result`, calls `parser_parse_program`, prints the result.
- `tests/phase4b1_parser.rs` — six BOLI-executed tests and one static import-boundary test.
- `docs/PHASE4B_1_REPORT.md` — this report.

Modified: `docs/SELF_HOSTING_ROADMAP.md` (status note only).

## 3. Design decisions

- AST nodes are maps with `tag`, `line`, `col`, `span`, and named fields; this mirrors `src/ast.rs` without changing Rust.
- `Program` uses a zero span: `S(0,0,1,1,1,1)`.
- Dump shape is `TAG|line|col|SPAN|field=value`; nested nodes are `(...)`, lists are `<n>[...]`, optional lists use `khaali`.
- Parser result is `{ok, ast, error, state}`. Errors are copied from token spans with all six span fields plus `token_kind`, `context`, and `recovered`.
- Token state uses a 1-based `pos` for BOLI list indexing. Rust's 0-based `Parser.pos` is not reused.
- Out-of-range reads return the synthetic EOF token `Eof||1|1|0|0|1|1|1|1`.
- `token_stream_next` returns `{ok, token, state}` because BOLI maps are values; mutating a local copy cannot silently update the caller.
- `token_stream_skip_newlines` mutates the parameter map's `pos` field so the loop terminates correctly.
- BOLI calls are flat: `f a b` means `f(a, b)`. Nested calls must be bound to temporaries first. Helpers `ast_field` and `ast_join_parts` exist to keep dump code correct and readable.
- Lists are numeric-key maps; `jodo_text` needs text-key maps, so list joining uses `ast_join_parts` instead.
- `atom_start` follows `src/parser.rs`, which excludes `Tok::Hissa`, despite `docs/PARSER_GRAMMAR.md` listing `"hissa"`. This discrepancy is recorded and not silently "fixed" in code.
- `parser_not_implemented` is a skeleton diagnostic only; it is not the final Phase 4B.2 error taxonomy.
- Parse depth defaults are `64`; condition depth defaults are `128`; options map carries the existing runtime limits.

## 4. Tests

`tests/phase4b1_parser.rs` verifies:

1. all 9 `Expr`, 3 `Cond`, and 15 `Stmt` constructors plus `Program`, via canonical dumps;
2. cursor `peek`, `peek_at`, `next`, `skip_newlines`, and state advance;
3. synthetic EOF, `at_eof`, `expect`, `expect_line_end`, `expect_ident`, atom predicates, and exact error text;
4. skeleton empty-program success and non-EOF `parser_not_implemented`;
5. unchanged propagation of lexer errors;
6. no direct imports of `lexer.boli`, `reader.boli`, or `unicode.boli` from parser modules, and the single allowed `../lexer/token.boli` import.

The AST test also verifies optional-list handling (`else_block=khaali`, `Wapas` with `khaali`) and list-valued fields (`pairs=<1>[...]`).

## 5. Validation

All commands run from `/data/data/com.termux/files/home/boli` with `PATH=/data/data/com.termux/files/usr/bin:$PATH`.

- `cargo fmt --all` applied before final check.
- `cargo fmt --all -- --check` — pass.
- `cargo check --all-targets --locked` — pass.
- `cargo clippy --all-targets --locked -- -D warnings` — pass.
- `cargo test --all-targets --locked --quiet` — **222 passed, 0 failed, 2 ignored**.
- `cargo build --release --locked` — pass.
- `cargo run --quiet -- run selfhost/lexer/demo.boli` — pass, unchanged token output.
- `git diff --check` — pass.

## 6. Limitations / blockers for Phase 4B.2

- The parser cannot parse any statement, expression, condition, or block yet; it only owns the foundation and reports `parser_not_implemented`.
- No differential corpus compares BOLI parser AST dumps against Rust AST yet.
- Runtime/step/call/value-depth limits remain a broader self-hosting blocker.
- VM stabilization still precedes compiler migration.
- The `atom_start` doc/source mismatch needs an explicit decision before grammar work.
- No commit or push was performed for Phase 4B.1.
