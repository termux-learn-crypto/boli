# BOLI — Security Report (v0.3.0)

**Scope:** the interpreter and CLI as shipped in `boli` v0.3.0. **Threat model:** BOLI programs may come from untrusted authors (a user running someone else's `.hl`). The host process must not crash, hang forever, exhaust memory without bound, or misreport safety guarantees. **Not in scope:** confidentiality (a run program can legitimately print anything it is given), sandboxing of the target machine (BOLI cannot execute shell, files, or network — by design there is no such capability).

## Resource-limit matrix (all enforced, all documented, all env-overridable)

| Resource | Limit | Env var | Default | Enforced at |
|---|---|---|---|---|
| Program steps | 100,000 | `BOLI_MAX_STEPS` | per program run | `run_stmt` + range loop |
| Kaam (call) depth | 64 | `BOLI_MAX_CALL_DEPTH` | nested calls | `call_kaam` |
| Expression eval depth | 256 | `BOLI_MAX_EVAL_DEPTH` | nested expressions | `eval_expr` |
| Parse nesting depth | 64 | `BOLI_MAX_PARSE_DEPTH` | statements/expressions | `parse_stmt`, `atom_base` |
| Condition chain | 128 | `BOLI_MAX_CONDITION_DEPTH` | `aur`/`ya` chains, parse + eval | `parse_cond`, `parse_aur`, `eval_cond` |
| String bytes | 1 MiB | `BOLI_MAX_STRING_BYTES` | literals, pucho input, concat results, input lines | eval, lexer/parse, `pucho` |
| List items | 10,000 | `BOLI_MAX_LIST_ITEMS` | creation & `jodo` concat | `Expr::List`, `eval_arith` |
| Source file bytes | 10 MiB | `BOLI_MAX_SOURCE_BYTES` | `boli run x.hl` | `main.rs` metadata check |

Every violation is a deterministic Hinglish `Galti: line N me …` error. **No violation is a panic, SIGSEGV, SIGABRT, or (in tests) an unwind.**

## Attack-surface review

### 1. Stack exhaustion (was: crash)
Before v0.3.0, three independent axes overflowed the native stack:
1. Nesting depth (parser/eval recursion) — fixed in v0.2.0 (const) → v0.3.0 (env `BOLI_MAX_PARSE_DEPTH`/`BOLI_MAX_EVAL_DEPTH`).
2. Condition `aur`/`ya` chains (up to 200k terms reproduced SIGSEGV) — fixed v0.3.0 (chain counters at parse and eval).
3. `hissa` postfix chains 1,000s deep (parser loop builds left-heavy AST; eval recursion) — capped by eval depth 256.
Fuzz test (deterministic, no dependencies) confirms lexer+parser do not panic on random inputs.

### 2. Memory exhaustion
- String literals: now checked at eval (`BOLI_MAX_STRING_BYTES`); concat was already checked; `pucho` now capped with bounded chunked reads.
- Lists: capped at 10,000 items (creation + concat).
- Source file: refused above 10 MiB before `read_to_string` allocates.
- Steps: 100k global budget terminates loop-bombs (range `1 se 999999999 tak`, `jab tak sahi to …`).
- Numbers: f64, NaN/±∞ rejected at every entry point; arithmetic overflow → error, not silent inf.

### 3. Command/code injection
None possible: no shell, no `exec`, no file/network/FFI capability exists anywhere in the language. The only I/O primitives are `likho` (stdout) and `pucho` (stdin). `install.sh` copies a prebuilt binary; it does not pipe to a shell.

### 4. Broken pipe / SIGPIPE
`boli … | head` exits 0 cleanly instead of panicking (tested in CLI suite).

### 5. Integer/float edge poisoning
`pucho` parses only finite numbers; `nan`/`inf` remain text. Literals validated. `f64::INFINITY` via overflow → Galti.

### 6. File-system handling
`boli run` only *reads* the named file (no writes, no include/append primitives). Refusal before read on oversize.

### 7. Dependency supply chain
Zero third-party dependencies (`[dependencies]` is empty). `cargo build` fetches nothing from crates.io. Dev tooling (rustfmt/clippy) is part of the toolchain only.

### 8. Panic paths
Remaining `unwrap`s are internal invariants (scope stack never empty; last token always present). User-facing errors all funnel to `BoliError`. `process::exit(0)` only on closed stdout — deliberate.

## Known limitations / honest residual risks

- `pucho`'s remaining unread bytes of an over-cap line are not drained (irrelevant once the program errors out; a REPL-style wrapper must drain before the next `pucho`).
- REPL depth detection is quote/comment-aware text matching, not real parsing — pathological multiline quoting can fool it; `nikalo` at depth 0 always exits.
- Steps limit counts statements, not bytes of work inside `jodo` on huge strings — bounded anyway by string cap.
- No Unicode NFC normalization (documented; affects identifier lookup only, not memory safety).
- Limits are defaults chosen for a teaching language; raise them deliberately (`BOLI_MAX_*`) for batch workloads.
- Not a sandbox for the CPU: `jab tak sahi to likho 1 khatam` burns CPU until the steps limit (~instant at 100k).
- `std::process::exit(0)` on EPIPE means a closed-pipe pipeline member is silently truncated — documented SIGPIPE semantics, intended.

## Verification performed (not claims — actual commands)

- `cargo test` — 82/82 pass (67 library + 15 CLI)
- `cargo clippy --all-targets --all-features -- -D warnings` — clean
- Reproduced & confirmed fixed: 200k-term condition SIGSEGV → Galti; 3,000-deep expression nesting SIGSEGV → Galti (0.2.0); 20,000-item list → Galti; literal bypass → Galti; 11 MiB file → Galti; `LIKHO` keyword equivalence; REPL multiblock.

## Recommendations
1. Keep `[dependencies]` empty; vet any addition hard.
2. If BOLI gains file/network primitives later (see UPGRADE_ROADMAP), every one needs its own `BOLI_MAX_*` limit and an entry here.
3. Publish this matrix in the security section of the README/website once v0.4 features land.
