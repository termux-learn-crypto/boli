# BOLI v0.6.5 — Test Results (actual commands, actual output)

**Host:** Termux on Android (aarch64), rustc/cargo 1.100.0 via the system Termux package. **Date:** 2026-11-19. **Version:** `BOLI version 0.6.5`. **Note:** `~/.cargo/bin/cargo` is a broken rustup shim; commands below use `/data/data/com.termux/files/usr/bin/cargo` (exported via `PATH`).

This is the current status page. Older reports remain as historical snapshots: [COMPLETE_AUDIT.md](COMPLETE_AUDIT.md), [SECURITY_REPORT.md](SECURITY_REPORT.md), [UPGRADE_ROADMAP.md](UPGRADE_ROADMAP.md).

## Commands and results

### 1. `cargo fmt --check`
```
(clean — zero diffs)
```

### 2. `cargo check --all-targets`
```
    Checking boli v0.6.5 (/data/data/com.termux/files/home/boli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.24s
```

### 3. `cargo clippy --all-targets -- -D warnings`
```
    Checking boli v0.6.5 (/data/data/com.termux/files/home/boli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.82s
```
Zero warnings.

### 4. `cargo test`
```
running 4 tests
test limits::tests::oversized_line_across_chunks_is_drained ... ok
test limits::tests::oversized_line_with_newline_in_same_chunk_is_drained ... ok
test interpreter::tests::bounded_distance_respects_limit ... ok
test interpreter::tests::nearest_name_skips_oversized_identifiers ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 114 tests
test result: ok. 114 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 34 tests
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 39 tests
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Historical v0.6.5 baseline above: **191 passed / 0 failed** (4 unit + 114 `tests/boli.rs` library + 34 `tests/cli.rs` end-to-end + 39 `tests/vm.rs` VM/parity).

Current Phase 3.2 gate: **215 passed / 0 failed / 2 ignored** via `cargo test --all-targets --locked --quiet`; the increase over the Phase 3.1 baseline (214 passed) is the additional `tests/phase3_lexer.rs` deterministic Unicode/generated-case test.

Regenerate the count with:
```bash
cargo test -- --list | grep ': test$' | wc -l
```

### 5. `cargo build --release`
```
   Compiling boli v0.6.5 (/data/data/com.termux/files/home/boli)
    Finished `release` profile [optimized] target(s) in 24.73s
```

### 6. `./target/release/boli version`
```
BOLI version 0.6.5
```

## What v0.6.5 added

- Production-readiness packaging: pinned Rust 1.100.0 toolchain, `--locked` CI builds, crates.io metadata, and tag-gated releases with SHA-256 checksums.
- One CLI production smoke test brings the CLI suite to 34 tests and the total to 191.
- The CI matrix covers ubuntu/macOS/Windows; the command output above was produced locally on Termux aarch64 only — the multi-OS matrix was not run locally.

## What v0.6.4 added to verification

- Deep nested map programs fail with a bounded-depth Galti in both engines instead of SIGSEGV.
- CLI regressions cover `u64::MAX` source limits, escaped REPL quotes, content-only line caps and a real closed stdout pipe.
- Parser/lexer regressions cover shared `aur`/`ya` depth, `hissa` depth, panic-free public parser input, stuck `5-3`, and map error columns.
- VM regressions cover range `ginti` accounting, runtime-only `kaam`/`laao` effects, top-level return, condition lines, hints and uncalled `ruk`/`agla`.

## What v0.6.3 added to verification

- `interpreter::tests::bounded_distance_respects_limit` and `nearest_name_skips_oversized_identifiers`: suggestions are cutoff-bounded and oversized names are ignored.
- `cli_repl_oversized_multiline_buffer_is_dropped_and_repl_survives`: aggregate REPL overflow clears the partial block and the next command still runs.
- `cli_entry_file_self_import_runs_once_in_both_engines` and `entry_file_self_import_runs_once_in_interpreter_and_vm`: canonical entry registration prevents duplicate top-level execution.

## Manual verification (examples & CLI)

- `boli run examples/loop.hl` → loops print correct output.
- `boli run examples/calculator.hl` with stdin `5\n3` → `8`, `2`, `15`, `1.666…`.
- `boli run examples/calculator2.hl` → `8` for `jodo`; EOF paths stay graceful.
- `boli run examples/namaste.hl` → greeting prints; EOF on `pucho` yields `khaali`.
- REPL: `likho jodo 6 aur 7` → `13`; `nikalo` exits cleanly.
- 11 MiB source file → Hinglish Galti with size and cap; exit 1.
- 200,000-term `ya` condition → Hinglish Galti in under a second.
- 20,000-item list literal → Hinglish Galti.
- Oversized `pucho` line in REPL → Galti, then the next short line is accepted.

## Phase 3.1 and Phase 3.2 verification addenda

- `cargo test --test phase3_lexer --locked --quiet`: 11 passed, 0 failed after Phase 3.2 (10 passed after Phase 3.1).
- `cargo run --quiet -- run selfhost/lexer/demo.boli`: `Likho||1|1|0|5|1|1|1|6`, `Number|5|1|7|6|7|1|7|1|8`, `Newline||1|8|7|8|1|8|1|9`, `Eof||2|1|8|8|2|1|2|1`.
- Full gate re-run after Phase 3.2 edits: `cargo fmt --all -- --check`, `cargo check --all-targets --locked`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --all-targets --locked --quiet`, `cargo build --release --locked`, `cargo run --release --locked --quiet -- version`.
- Release binary still reports `BOLI version 0.6.5`.
- YAML parse: `.github/workflows/ci.yml` and `.github/workflows/release.yml` OK.
- `git diff --check`: clean.

## Environment notes (honest)

- `cargo` in `~/.cargo/bin` is a rustup shim with no default toolchain; the system Termux `cargo`/`rustc` 1.100.0 is used.
- This environment is Android/Termux (aarch64) — Termux compatibility is genuinely verified. Windows/macOS/iOS are **not** tested; no claims are made.
- The VM uses bytecode step semantics; step counts need not equal the tree-walker's statement count for an identical source file. Both enforce their configured `BOLI_MAX_STEPS`.

## Conclusion

All configured gates are green, the current gate has 215 passed / 2 ignored, and the release binary reports v0.6.5. No regressions detected.
