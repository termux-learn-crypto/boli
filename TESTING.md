# Testing BOLI

## How to run the tests

```bash
cargo test                  # everything (unit + integration)
cargo test --lib            # bounded-input + suggestion unit tests (4 tests)
cargo test --test boli      # library-level semantics (114 tests)
cargo test --test cli       # CLI/binary end-to-end (34 tests)
cargo test --test vm        # bytecode VM + parity (39 tests)
```

Full verification matrix (run before a release):

```bash
cargo fmt --check                          # formatting gate
cargo check --all-targets                  # compile gate (all targets)
cargo test --all-targets --locked --quiet   # 216 passed / 2 ignored (includes 11 phase3_lexer tests)
cargo clippy --all-targets -- -D warnings                  # lint gate
cargo build --release                      # release artifact gate
```

No extra tooling is required — only `cargo` and the BOLI binary (which `cargo test` builds automatically). The CLI tests use the `CARGO_BIN_EXE_boli` path cargo provides; the library tests use the `Environment`/`parse_source` API directly and capture `likho` output via `Environment::collect_output`.

Test counts can be regenerated from the current tree with:

```bash
cargo test -- --list | grep ': test$' | wc -l
```

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

### v0.6.5 production readiness
- One CLI production smoke test exercises the cargo-built binary end-to-end; the release gates (fmt/check/test/clippy -D warnings/release build) run in CI on ubuntu, macOS and Windows with `--locked`, and tagged releases publish SHA-256 checksummed artifacts.

### v0.6.4 audit regressions
- `deeply_nested_values_error_before_stack_overflow`, VM deep-value parity test: 5,000 nested maps fail with a bounded-depth Galti instead of SIGSEGV.
- Interpreter import no longer calls the resetting `run_program`; step budget survives `laao`, matching the VM.
- CLI regressions cover `u64::MAX` source cap, escaped quotes in REPL strings, content-only line caps and a real broken pipe.
- Parser/lexer regressions cover shared `aur`/`ya` depth, `hissa` depth, panic-free public parser, stuck `5-3`, and map error columns.
- VM regressions cover range `ginti` accounting, dead-branch `kaam`/`laao`, top-level return, condition lines, hint text and uncalled `ruk`/`agla`.

### v0.6.3 audit regressions
- `src/interpreter/mod.rs` unit: bounded Levenshtein respects its cutoff; oversized identifiers are skipped by `nearest_name()` while normal hints still work.
- `cli_repl_oversized_multiline_buffer_is_dropped_and_repl_survives`: many opening lines without `khatam` hit `BOLI_MAX_REPL_BUFFER_BYTES`, the block is cleared, and the next command runs.
- `cli_entry_file_self_import_runs_once_in_both_engines` and `entry_file_self_import_runs_once_in_interpreter_and_vm`: entry self-import executes top-level output once in interpreter and VM.

### v0.6.2 audit regressions
- `src/limits.rs` unit: over-limit lines with the newline in the same chunk and across chunks are drained; the next line is readable.
- `cli_import_root_confines_laao`: allowed import plus `../` and absolute escapes are checked for both interpreter and `BOLI_VM=1`.
- `vm_error_resets_stack_to_baseline`, `vm_steps_reset_between_programs`, `vm_failing_range_program_then_reuse_same_vm`: a reused VM is clean after errors and step counters restart.
- `vm_list_and_map_assignment_compiles_rhs_once_and_clears_stack`: list/map index assignment single-evaluation and stack cleanup.
- `vm_matches_interpreter_on_core_corpus`, `vm_and_interpreter_agree_on_runtime_errors`: both backends agree on output and error text for a shared corpus.

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

Put assertion-style tests in `tests/boli.rs` (use `run_out(src) -> Result<Vec<String>, BoliError>` to get captured output), VM/parity tests in `tests/vm.rs` (`run_vm(src)` / `interp_out(src)`), black-box CLI tests in `tests/cli.rs` (`run_with_stdin("file_name.hl", src, stdin) -> (status, stdout, stderr)`), or pure helper tests beside the code in `src/limits.rs`. Prefix temp files with the test's purpose to avoid collisions.

## Manual regression

```bash
cargo run -- run examples/namaste.hl
cargo run -- run examples/calculator.hl
cargo run -- run examples/calculator2.hl   # input: 6 7 guna
cargo run -- run examples/loop.hl
cargo run                                   # REPL smoke test
```
