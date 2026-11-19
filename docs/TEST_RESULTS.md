# BOLI v0.6.2 — Test Results (actual commands, actual output)

**Host:** Termux on Android (aarch64), rustc/cargo 1.100.0 via the system Termux package. **Date:** 2026-11-19. **Version:** `BOLI version 0.6.2`. **Note:** `~/.cargo/bin/cargo` is a broken rustup shim; commands below use `/data/data/com.termux/files/usr/bin/cargo` (exported via `PATH`).

This is the current status page. Older reports remain as historical snapshots: [COMPLETE_AUDIT.md](COMPLETE_AUDIT.md), [SECURITY_REPORT.md](SECURITY_REPORT.md), [UPGRADE_ROADMAP.md](UPGRADE_ROADMAP.md).

## Commands and results

### 1. `cargo fmt --check`
```
(clean — zero diffs)
```

### 2. `cargo check --all-targets`
```
    Checking boli v0.6.2 (/data/data/com.termux/files/home/boli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.70s
```

### 3. `cargo clippy --all-targets --all-features -- -D warnings`
```
    Checking boli v0.6.2 (/data/data/com.termux/files/home/boli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.11s
```
Zero warnings.

### 4. `cargo test`
```
running 2 tests
test limits::tests::oversized_line_across_chunks_is_drained ... ok
test limits::tests::oversized_line_with_newline_in_same_chunk_is_drained ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 108 tests
test result: ok. 108 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 28 tests
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 29 tests
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**167 passed / 0 failed** (2 unit + 108 `tests/boli.rs` library + 28 `tests/cli.rs` end-to-end + 29 `tests/vm.rs` VM/parity).

Regenerate the count with:
```bash
cargo test -- --list | grep ': test$' | wc -l
```

### 5. `cargo build --release`
```
   Compiling boli v0.6.2 (/data/data/com.termux/files/home/boli)
    Finished `release` profile [optimized] target(s) in 10.19s
```

### 6. `./target/release/boli version`
```
BOLI version 0.6.2
```

## What v0.6.2 added to verification

- `src/limits.rs` unit: over-limit `pucho` lines are drained both when the newline is in the same buffer and when it arrives in a later chunk; the following line is readable.
- `cli_import_root_confines_laao`: `BOLI_IMPORT_ROOT` permits an in-root import and rejects `../` and absolute escapes for both the interpreter and `BOLI_VM=1`.
- `vm_error_resets_stack_to_baseline`, `vm_steps_reset_between_programs`, `vm_failing_range_program_then_reuse_same_vm`: reused VMs start clean and step counters restart.
- `vm_list_and_map_assignment_compiles_rhs_once_and_clears_stack`: list/map index assignment evaluates the RHS once and leaves no stack residue.
- `vm_matches_interpreter_on_core_corpus`, `vm_and_interpreter_agree_on_runtime_errors`: interpreter and VM produce the same output and error text for a shared corpus.

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

## Environment notes (honest)

- `cargo` in `~/.cargo/bin` is a rustup shim with no default toolchain; the system Termux `cargo`/`rustc` 1.100.0 is used.
- This environment is Android/Termux (aarch64) — Termux compatibility is genuinely verified. Windows/macOS/iOS are **not** tested; no claims are made.
- The VM uses bytecode step semantics; step counts need not equal the tree-walker's statement count for an identical source file. Both enforce their configured `BOLI_MAX_STEPS`.

## Conclusion

All configured gates are green, 167/167 tests pass, and the release binary reports v0.6.2. No regressions detected.
