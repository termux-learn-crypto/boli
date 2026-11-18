# BOLI — Security Report (v0.5.0)

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
| Source file bytes | 10 MiB | `BOLI_MAX_SOURCE_BYTES` | `boli run x.hl` | `main.rs` metadata check + bounded `take(max+1)` read (growth-safe) |
| Program tokens | 1,000,000 | `BOLI_MAX_TOKENS` | lex time | lexer (`push_token`) |
| REPL input line bytes | 64 KiB | `BOLI_MAX_REPL_LINE_BYTES` | chunked read | `read_repl_line` (line drained + rejected) |
| Live scope bytes (memory budget) | 64 MiB | `BOLI_MAX_TOTAL_BYTES` | set/pop scope accounting (replace-safe) | interpreter |

Every violation is a deterministic Hinglish `Galti: line N me …` error. **No violation is a panic, SIGSEGV, SIGABRT, or (in tests) an unwind.**

## Attack-surface review

### 1. Stack exhaustion (was: crash)
Before v0.3.0, three independent axes overflowed the native stack:
1. Nesting depth (parser/eval recursion) — fixed in v0.2.0 (const) → v0.3.0 (env `BOLI_MAX_PARSE_DEPTH`/`BOLI_MAX_EVAL_DEPTH`).
2. Condition `aur`/`ya` chains (up to 200k terms reproduced SIGSEGV) — fixed v0.3.0 (chain counters at parse and eval).
3. `hissa` postfix chains 1,000s deep (parser loop builds left-heavy AST; eval recursion) — capped by eval depth 256.
Fuzz test (deterministic, no dependencies) confirms lexer+parser do not panic on random inputs.

### 2. Memory exhaustion
- Whole-program token count capped (`BOLI_MAX_TOKENS`); each literal capped (`BOLI_MAX_STRING_BYTES`) at lex time; identifiers capped the same way; the lexer no longer materializes a `Vec<char>` (sliding two-char cursor) — large inputs tokenize with ~2 chars of lookahead.
- String literals: checked at lex AND at eval (`checked_text`); concat checked; `pucho` capped with bounded chunked reads.
- Lists: capped at 10,000 items (creation + concat).
- Source file: metadata check AND bounded `take(max+1)` read — a file that grows during read is still stopped at the cap (no metadata TOCTOU).
- Live scope bytes budget (`BOLI_MAX_TOTAL_BYTES`, default 64 MiB): every value entering a scope is weighed (text bytes, list cells×40B, scalars); replacing a name releases the old weight; scope pops decrement. Exceeding the budget is a deterministic Galti error — no partial-run corruption, no OOM.
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

- `cargo test` — 112/112 pass (93 library + 19 CLI)

## Runtime tracebacks (v0.5.0)
`BoliError.trace` exposes the kaam call stack on runtime errors (innermost-first, name + call line). This discloses internal kaam names and call line numbers to whoever reads stderr — acceptable for a local interpreter (the program source is visible anyway), but worth noting for CI log hygiene. No new memory risk: frames are small and popped on every exit path (tested).

## Built-ins (v0.4.0) attack surface
`lambayi`, `upar`, `neeche`, `text`, `sankhya` — all pure internal Rust; every allocation they make is bounded by the existing string caps (`BOLI_MAX_STRING_BYTES`), list caps (`BOLI_MAX_LIST_ITEMS`) and live-scope budget (`BOLI_MAX_TOTAL_BYTES`). No I/O, no filesystem, no network. `sankhya` rejects inf/NaN.
- `cargo clippy --all-targets --all-features -- -D warnings` — clean
- Reproduced & confirmed fixed: 200k-term condition SIGSEGV → Galti; 3,000-deep expression nesting SIGSEGV → Galti (0.2.0); 20,000-item list → Galti; literal bypass → Galti; 11 MiB file → Galti; `LIKHO` keyword equivalence; REPL multiblock.

## Recommendations
1. Keep `[dependencies]` empty; vet any addition hard.
2. If BOLI gains file/network primitives later (see UPGRADE_ROADMAP), every one needs its own `BOLI_MAX_*` limit and an entry here.
3. Publish this matrix in the security section of the README/website once v0.4 features land.
