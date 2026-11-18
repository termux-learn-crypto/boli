# Testing BOLI

## How to run the tests

```bash
cargo test          # everything (unit + integration)
cargo test --test boli      # library-level semantics (93 tests)
cargo test --test cli       # CLI/binary end-to-end (19 tests)
```

Full verification matrix (run before a release):

```bash
cargo fmt --check                          # formatting gate
cargo check                                # compile gate
cargo test                                 # 112 tests (93 library + 19 CLI)
cargo clippy --all-targets --all-features -- -D warnings   # lint gate
cargo build --release                      # release artifact gate
```

No extra tooling is required — only `cargo` and the BOLI binary (which `cargo test` builds automatically). The CLI tests use the `CARGO_BIN_EXE_boli` path cargo provides; the library tests use the `Environment`/`parse_source` API directly and capture `likho` output via `Environment::collect_output`.

## What is tested

### Lexer (`tests/boli.rs`)
- token stream for a simple statement
- negative number tokenizing
- string escapes (`\n`, `\"`)
- symbol rejection with Hinglish error
- Unicode identifiers

### Parser (`tests/boli.rs`)
- single-value vs multi-value `rakho` → AST (`Rakho`, `Expr::List`)
- `ruk`/`agla` → `Stmt::Ruk` / `Stmt::Agla`
- `hissa` postfix index AST (`Expr::Index`)
- `ListAssign` statement syntax
- flat call arguments (`f a b` → `(a, b)`, not `a(b)`)
- `aur`/`ya` precedence at AST level (`Ya(a, Aur(b, c))`)

### Safety limits (`tests/boli.rs`)
- literal string size limit (`with_limits` + small `BOLI_MAX_STRING_BYTES`)
- list creation limit and list `jodo` concat limit (`with_list_limit`)
- condition `ya` chain parse-time cap (`Parser::with_limits`) and eval-time cap
- expression/`hissa` chain depth guard
- fuzz-ish test: deterministic xorshift generator feeds random byte soup into the lexer and parser; neither may panic (no third-party fuzzer needed)

### v0.5.0 tests
- `traceback_shows_full_call_chain`, `traceback_innermost_first`, `traceback_absent_for_top_level_errors`, `traceback_preserved_through_nested_propagation`

### v0.4.0 tests
- Builtins: `builtin_lambayi_text_and_list`, `builtin_upar_neeche`, `builtin_text_cast`, `builtin_sankhya`, `builtin_wrong_arity_errors`, `builtin_user_kaam_shadows_builtin`, `builtin_unicode_text_ops`
- Columns/caret: `lexer_error_has_column`, `parser_error_has_column`, `unterminated_text_error_has_position`, `symbol_error_caret_position`, `khaali_message_format_with_col`
- CLI: `cli_syntax_error_shows_line_and_caret`

### v0.3.1 regression tests
- `eval_depth_limit_is_configurable_and_enforced` — `with_eval_depth_limit(10)` errors, `with_eval_depth_limit(500)` passes (and the env var itself is covered by `cli_env_eval_depth_is_respected`)
- `lexer_string_literal_byte_limit`, `lexer_identifier_byte_limit`, `lexer_token_limit`, `lexer_source_byte_limit`, `lexer_degrammar_still_works_after_cursor_rewrite`
- `total_budget_caps_string_storage`, `total_budget_caps_list_storage`, `total_budget_freed_by_scope_pop`, `total_budget_replacement_is_not_double_counted`
- CLI (subprocess, real env): `cli_source_bytes_boundary` (exactly-at vs one-under), `cli_env_eval_depth_is_respected`, `cli_repl_oversized_line_is_rejected_and_repl_survives`

### Interpreter semantics (`tests/boli.rs`)
- inline math (`likho jodo 5 aur 3` → `8`)
- assignments, types, conditions (then/else/nesting/`barabar` strings)
- range loops (basic/negative/empty/2^52-boundary/huge terminates via steps)
- while loops, `ruk`, `agla`, break-outside-loop error
- loop scope cleanup on success **and** on runtime errors; REPL state consistency
- kaam: zero/one/multiple/nested/expression args, arity errors, recursion limit
- global variable not overwritten from inside a kaam; kaam reads global fine
- lists: create/print/index/assign/concat/out-of-bounds error
- steps limit stops infinite loops with Hinglish error
- **BUG 5 regressions:** literal overflow rejected, large finite literals accepted, decimal literals work, arithmetic overflow errors instead of producing ∞, finite results unchanged

### CLI (`tests/cli.rs`, spawned binary)
- `boli run examples/loop.hl` and `examples/calculator2.hl` work end-to-end
- missing file → Hinglish `Galti:` to stderr, exit 1
- REPL: stray `khatam` doesn't latch the depth counter; multi-line blocks still work
- broken pipe (stdout closes early) exits 0 without panic
- **BUG 5 input regressions:** `nan`/`inf`/`Infinity`/`-Infinity` via `pucho` enter as *text*, not numbers; `99.50` parses as a number; a full-range f64 literal stays a number; arithmetic overflow exits non-zero with the Hinglish message

## Environment

- `cargo test` needs a BOLI build; it builds one automatically.
- CLI tests write temp `.hl` files under `CARGO_TARGET_TMPDIR` and use per-test unique file names so the parallel test runner doesn't race.

## Adding a test

Put assertion-style tests in `tests/boli.rs` (use `run_out(src) -> Result<Vec<String>, BoliError>` to get captured output), or black-box CLI tests in `tests/cli.rs` (`run_with_stdin("file_name.hl", src, stdin) -> (status, stdout, stderr)`). Prefix temp files with the test's purpose to avoid collisions.

## Manual regression

```bash
cargo run -- run examples/namaste.hl
cargo run -- run examples/calculator.hl
cargo run -- run examples/calculator2.hl   # input: 6 7 guna
cargo run -- run examples/loop.hl
cargo run                                   # REPL smoke test
```
