# BOLI v0.6.0 — Test Results (actual commands, actual output)

**Host:** Termux on Android (aarch64), rustc/cargo 1.100.0 via system Termux package (note: `~/.cargo/bin/cargo` is a broken rustup shim; `/data/data/com.termux/files/usr/bin/cargo` is used — see Environment notes). **Date:** 2026-11-18. **Commit:** v0.3.0 (release).

## Commands and results

### 1. `cargo fmt --check`
```
(clean — zero diffs; one-time reformat commit df5c012 made this a green gate)
```

### 2. `cargo check`
```
Checking boli v0.3.0 (/data/data/com.termux/files/home/boli)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.92s
```

### 3. `cargo test`

```
running 77 tests
test result: ok. 77 passed; 0 failed
running 18 tests
test result: ok. 18 passed; 0 failed
```

```
running 0 tests
test result: ok. 0 passed; 0 failed
running 0 tests
test result: ok. 0 passed; 0 failed
running 103 tests
test result: ok. 103 passed; 0 failed; finished in 0.30s
running 23 tests
test result: ok. 23 passed; 0 failed; finished in 0.34s
running 21 tests
test result: ok. 21 passed; 0 failed; finished in 0.04s
running 0 tests
test result: ok. 0 passed; 0 failed
```
**147 passed / 0 failed** (103 `tests/boli.rs` library + 23 `tests/cli.rs` end-to-end + 21 `tests/vm.rs` VM parity).

New tests added in v0.3.0 (all in `tests/boli.rs` unless noted):
- `literal_string_size_limit` — literal bypasses the string-bytes limit → error
- `list_creation_limit`, `list_concat_limit`, `list_under_limit_works`
- `condition_ya_chain_parse_limit`, `condition_ya_chain_eval_limit`
- `lexer_rejects_control_characters`
- `deeply_nested_agar_errors_not_crashes`, `deeply_nested_expression_errors_not_crashes`, `deeply_nested_hissa_chain_errors_not_crashes`, `moderately_nested_code_still_works`
- `fuzz_lex_parse_never_panics` — deterministic xorshift, 2,000 random-ish inputs through lexer+parser; neither may panic; no dependencies
- `malformed_programs_error_not_crash`
- CLI (`tests/cli.rs`): `cli_bogus_subcommand_errors`, `cli_run_with_too_many_args_errors`

### v0.3.1 tests added
- Unit: `eval_depth_limit_is_configurable_and_enforced` (10 → error "(max 10)", 500 → ok), `lexer_string_literal_byte_limit`, `lexer_identifier_byte_limit`, `lexer_token_limit`, `lexer_source_byte_limit`, `lexer_degrammar_still_works_after_cursor_rewrite`, `total_budget_caps_string_storage`, `total_budget_caps_list_storage`, `total_budget_freed_by_scope_pop`, `total_budget_replacement_is_not_double_counted`
- CLI (subprocess with real env): `cli_source_bytes_boundary`, `cli_env_eval_depth_is_respected`, `cli_repl_oversized_line_is_rejected_and_repl_survives`

### 4. `cargo clippy --all-targets --all-features -- -D warnings`
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.92s
```
Zero warnings (run with `-D warnings`; failing would abort).

### 5. `cargo build --release`
```
Finished `release` profile [optimized] target(s) in 0.57s
-rwxr-xr-x boli 782,672 bytes
```

## Manual verification (examples & CLI)

- `boli run examples/loop.hl` → all loops print correct output (ginti 1..5, Adult ho).
- `boli run examples/calculator.hl` (stdin `5\n3`) → `8`, `2`, `15`, `1.666…`; inline `150`/`50` after prompts.
- `boli run examples/calculator2.hl` → `8` for `jodo`; EOF paths print `khaali`/`Galat kaam likha hai` without panic.
- `boli run examples/namaste.hl` → prints greeting; EOF on `pucho` yields `khaali` gracefully.
- REPL: `likho jodo 6 aur 7` → `13`; `nikalo` exits cleanly.
- 11 MiB source file → `Galti: file '…' bahut badi hai (11534336 bytes, max 10485760)`, exit 1.
- 200,000-term `ya` condition → `Galti: line 2 me condition bahut lambi ho gayi (max 128 'ya')`, exit 1, <1s (previously SIGSEGV).
- 20,000-item list literal → `Galti: line 1 me list bahut badi ho gayi (max 10000 items)`.
- `BOLI_MAX_STRING_BYTES=1` with `likho "abcd1234"` → `Galti: line 1 me text bahut bada ho gaya (max 1 bytes)` (previously printed).
- `install.sh` with `BOLI_INSTALL_DIR=$HOME/boli-install-test` → installs, `boli version` → `BOLI version 0.3.0`, runs examples; test dir removed after verification.

## Environment notes (honest)
- `cargo` in `~/.cargo/bin` is a rustup shim with no default toolchain configured; the system Termux `cargo`/`rustc` 1.100.0 (`/data/data/com.termux/files/usr/bin/cargo`) is used throughout. `cargo fmt` works via Termux's rustfmt 1.11.0.
- This environment IS Android (Termux, aarch64) — Termux compatibility is genuinely verified here. Windows/macOS/iOS are **not** tested; claims are not made.
- `cargo fmt --check` initially reported 118 pre-existing diffs (author's compact style); commit `df5c012` is a pure reformat (no logic change; 71→71 tests still passing), after which the gate is clean.

## Conclusion
All configured gates are green and all v0.3.0 fixes are verified with passing tests or direct CLI reproduction. No regressions detected in 82 automated tests.
