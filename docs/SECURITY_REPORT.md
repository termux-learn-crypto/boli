# BOLI — Security Report (v0.6.5)

**Scope:** the tree-walking interpreter, experimental bytecode VM, CLI and REPL as shipped in `boli` v0.6.5. **Threat model:** BOLI programs may come from untrusted authors (a user running someone else's `.hl`). The host process must not crash, hang forever, exhaust memory without bound, leak state into a later program, or misreport safety guarantees. **Not in scope:** confidentiality (a running program can print what it is given), full OS sandboxing (BOLI has no shell/network/write/FFI capability), and the security of the host filesystem outside files the user chose to expose through `laao`.

## Resource-limit matrix (all enforced, all documented, all env-overridable)

| Resource | Default limit | Env var | Applies to | Enforced at |
|---|---|---|---|---|
| Program steps | 100,000 | `BOLI_MAX_STEPS` | per program run | `run_stmt` + range loop |
| Kaam (call) depth | 64 | `BOLI_MAX_CALL_DEPTH` | nested calls | `call_kaam` |
| Expression eval depth | 256 | `BOLI_MAX_EVAL_DEPTH` | nested expressions | `eval_expr` |
| Parse nesting depth | 64 | `BOLI_MAX_PARSE_DEPTH` | statements/expressions | `parse_stmt`, `atom_base` |
| Condition chain | 128 | `BOLI_MAX_CONDITION_DEPTH` | `aur`/`ya` chains, parse + eval | `parse_cond`, `parse_aur`, `eval_cond` |
| String bytes | 1 MiB | `BOLI_MAX_STRING_BYTES` | literals, pucho input, concat results, input lines | eval, lexer/parse, `pucho` |
| List items | 10,000 | `BOLI_MAX_LIST_ITEMS` | creation & `jodo` concat | `Expr::List`, `eval_arith` |
| Source file bytes | 10 MiB | `BOLI_MAX_SOURCE_BYTES` | `boli run x.hl` + `laao` imports | `main.rs` / `limits.rs::read_source_capped` — metadata fast-path + bounded `take(max+1)` read (growth-safe, shared) |
| Module import depth | 32 | `BOLI_MAX_IMPORT_DEPTH` | both engines | interpreter + VM compiler recursive import guard |
| Module import root | unset (any local path) | `BOLI_IMPORT_ROOT` | interpreter + VM `laao` | canonicalized prefix check in `limits::resolve_import_path` |
| Program tokens | 1,000,000 | `BOLI_MAX_TOKENS` | lex time | lexer (`push_token`) |
| REPL input line bytes | 64 KiB | `BOLI_MAX_REPL_LINE_BYTES` | chunked read | `read_repl_line` (line drained + rejected) |
| REPL multiline buffer bytes | 1 MiB | `BOLI_MAX_REPL_BUFFER_BYTES` | incomplete block | `repl()` clears the partial block on overflow |
| `pucho` input line bytes | 1 MiB | `BOLI_MAX_STRING_BYTES` | stdin | `read_input_line_from` (drains through newline on rejection) |
| Live scope bytes (memory budget) | 64 MiB | `BOLI_MAX_TOTAL_BYTES` | set/pop scope accounting (replace-safe) | interpreter and VM |
| Nested list/map value depth | 64 | `BOLI_MAX_VALUE_DEPTH` | constructed/stored values | interpreter and VM |

Every known violation is a deterministic Hinglish `Galti: line N me …` error. Security fixes are regression-tested; a future untested path may still be a bug, so this report does not claim that no violation can ever be a panic.

## Attack-surface review

### 1. Stack exhaustion (was: crash)
Before v0.3.0, three independent axes overflowed the native stack:
1. Nesting depth (parser/eval recursion) — fixed in v0.2.0 (const) → v0.3.0 (env `BOLI_MAX_PARSE_DEPTH`/`BOLI_MAX_EVAL_DEPTH`).
2. Condition `aur`/`ya` chains (up to 200k terms reproduced SIGSEGV) — fixed v0.3.0 (chain counters at parse and eval).
3. `hissa` postfix chains 1,000s deep (parser loop builds left-heavy AST; eval recursion) — counted in parse depth 64 and capped by eval depth 256.
Fuzz test (deterministic, no dependencies) confirms lexer+parser do not panic on random inputs.

### 2. Memory exhaustion
- Whole-program token count capped (`BOLI_MAX_TOKENS`); each literal capped (`BOLI_MAX_STRING_BYTES`) at lex time; identifiers capped the same way; the lexer no longer materializes a `Vec<char>` (sliding two-char cursor) — large inputs tokenize with ~2 chars of lookahead.
- String literals: checked at lex AND at eval (`checked_text`); concat checked; `pucho` capped with bounded chunked reads.
- Lists: capped at 10,000 items (creation + concat).
- Source file: metadata check AND bounded `take(max+1)` read — a file that grows during read is still stopped at the cap (no metadata TOCTOU).
- Live scope bytes budget (`BOLI_MAX_TOTAL_BYTES`, default 64 MiB): every value entering a scope is weighed (text bytes, list cells×40B, scalars); replacing a name releases the old weight; scope pops decrement. Both engines account for range-loop `ginti` through the normal `set` path. Exceeding the budget is a deterministic Galti error — no partial-run corruption, no OOM.
- Nested value depth (`BOLI_MAX_VALUE_DEPTH`, default 64): list/map construction and storage are checked in both engines; `value_size` and display are iterative, so deep input is rejected before native-stack exhaustion.
- Steps: 100k global budget terminates loop-bombs (range `1 se 999999999 tak`, `jab tak sahi to …`).
- Numbers: f64, NaN/±∞ rejected at every entry point; arithmetic overflow → error, not silent inf.

### 3. Command/code injection
No shell, `exec`, network, write, or FFI capability exists anywhere in the language. The I/O primitives are `likho` (stdout), `pucho` (stdin) and read-only local module loading through `laao`. `BOLI_IMPORT_ROOT` can confine module reads to a canonicalized directory; when unset, any readable local path is allowed. `install.sh` copies a prebuilt binary; it does not pipe to a shell.

### 4. Broken pipe / SIGPIPE
`boli … | head` exits 0 cleanly instead of panicking (tested in CLI suite).

### 5. Integer/float edge poisoning
`pucho` parses only finite numbers; `nan`/`inf` remain text. Literals validated. `f64::INFINITY` via overflow → Galti.

### 6. File-system handling
`boli run` only *reads* the named file (no writes, no include/append primitives). Main files and `laao` imports use one bounded `take(max+1)` read, so a file that grows during the read is still rejected. Import paths are canonicalized and depth-capped; `BOLI_IMPORT_ROOT` adds an optional prefix check for both engines.

### 7. Dependency supply chain
Zero third-party dependencies (`[dependencies]` is empty). `cargo build` fetches nothing from crates.io. Dev tooling (rustfmt/clippy) is part of the toolchain only.

### 8. Panic paths
Remaining `unwrap`s are internal invariants (scope stack never empty; last token always present). User-facing errors all funnel to `BoliError`. `process::exit(0)` only on closed stdout — deliberate.

## Known limitations / honest residual risks

- `pucho` drains an over-cap line through the next newline before returning the Galti; this protects the REPL but does not make an unbounded stream cheap — the bytes are still read and discarded.
- REPL depth detection is quote/comment-aware text matching, not real parsing — pathological multiline quoting can fool it; `nikalo` at depth 0 always exits.
- Steps limit counts high-level statements/bytecode instructions, not bytes of work inside one builtin (e.g. `jodo` on a large string) — bounded separately by string/list caps and the live-scope budget. VM and interpreter step counters are independent; the VM's `BOLI_MAX_STEPS` value is not required to match the tree-walker's statement count for the same program.
- No Unicode NFC normalization (documented; affects identifier lookup only, not memory safety).
- Limits are defaults chosen for a teaching language; raise them deliberately (`BOLI_MAX_*`) for batch workloads.
- Not a sandbox for the CPU: `jab tak sahi to likho 1 khatam` burns CPU until the steps limit (~instant at 100k).
- `std::process::exit(0)` on EPIPE means a closed-pipe pipeline member is silently truncated — documented SIGPIPE semantics, intended.

## Verification performed (not claims — actual commands)

- `cargo test --all-targets --locked --quiet` — 215 passed / 0 failed / 2 ignored; historical v0.6.5 baseline was 191/191 pass (4 unit + 114 library + 34 CLI + 39 VM)
- `cargo clippy --all-targets -- -D warnings` — clean
- `BOLI_IMPORT_ROOT` confinement verified for interpreter and `BOLI_VM=1` (`cli_import_root_confines_laao`)
- Over-limit input drainage verified in `src/limits.rs` unit tests and CLI REPL regression

## VM (v0.6.0)
The experimental bytecode VM (`--vm`, `BOLI_VM=1`) enforces the same limits: step budget, call depth 64, eval/parse depth, list item cap, string budget, live-scope bytes budget, token cap. `laao` is resolved/loaded at compile time and executed at runtime (bounded file read, cap `BOLI_MAX_SOURCE_BYTES`, canonical-path cycle protection, optional `BOLI_IMPORT_ROOT`). Steps, scopes and the value stack reset before each program and unwind after an error, so a reused `Machine` is clean. Default remains the tree-walking interpreter.

## Runtime tracebacks (v0.5.0)
`BoliError.trace` exposes the kaam call stack on runtime errors (innermost-first, name + call line). This discloses internal kaam names and call line numbers to whoever reads stderr — acceptable for a local interpreter (the program source is visible anyway), but worth noting for CI log hygiene. No new memory risk: frames are small and popped on every exit path (tested).

## Audit fixes / release hygiene (v0.6.5)

- CI builds every target with `--locked` on a pinned Rust 1.100.0 toolchain, so the committed `Cargo.lock` is exactly what is compiled and released.
- Releases are tag-gated and check that the tag matches the package version before publishing artifacts.
- Release artifacts are published with SHA-256 checksums so downloads can be verified.
- Tag validation reads the package version from `cargo metadata` (via `jq`) instead of parsing `cargo pkgid` text.
- GitHub Actions are pinned to full commit SHAs, with weekly Dependabot updates for the Actions ecosystem.
- These are packaging/hygiene changes only: no new runtime limits or attack surface. External security review is still **not** claimed — findings remain self-reviewed and regression-tested.

## Audit fixes (v0.6.4)

- Full-audit fix pass closed 15 confirmed findings across interpreter, VM, lexer/parser, CLI and installer.
- Nested values are depth-capped in both engines; value display/accounting are iterative; a 5,000-deep map program is a Galti, not a SIGSEGV.
- Imports no longer reset the interpreter step budget; `u64::MAX` read limits are overflow-safe via `saturating_add(1)`.
- VM range counters use normal scope accounting; `kaam`/`laao` take effect at execution time, not compile time; top-level return, condition lines/hints, uncalled `ruk`/`agla` and nested import errors match the interpreter.
- REPL quote/comment scanning and line caps are correct; parser depth is shared for `aur`/`ya` and counts `hissa`; public parser cannot panic; `5-3` errors; map errors carry columns.
- `install.sh` handles unset `HOME`; the broken-pipe test now forces EPIPE, and the panic-free guarantee above is stated only for tested paths.

## Audit fixes (v0.6.3)

- Did-you-mean hints no longer allocate an unbounded edit-distance matrix: 64-char identifier cap, 512-candidate cap, length pruning, dedup and a two-row bounded Levenshtein with early cutoff.
- REPL multiline input has an aggregate 1 MiB cap (`BOLI_MAX_REPL_BUFFER_BYTES`); an unfinished block that exceeds it is discarded so the REPL remains responsive.
- The canonical entry path is registered in the interpreter/VM module registries before execution, so entry self-import is a no-op and top-level statements run once.
- New tests cover bounded hints, aggregate REPL overflow and self-import parity across both engines.

## Audit fixes (v0.6.2)

- Over-limit `pucho` lines are drained through the next newline; the REPL/CLI cannot re-read the rejected line (`oversized_line_*` unit tests).
- VM `steps`, scopes and value stack reset on `run_program` and unwind on error paths; `stack_depth()` is exposed for regression assertions.
- `BOLI_IMPORT_ROOT` confines `laao` to a canonicalized directory in both engines; `../` and absolute escapes are Hinglish Galti errors when set.
- Differential tests compare interpreter and VM on a shared corpus and on matching runtime errors; list/map assignment tests assert single-evaluation and clean stacks.
- Current report status: `boli-audit-report.md` findings are resolved or explicitly documented in [COMPLETE_AUDIT.md](COMPLETE_AUDIT.md).

## Audit fixes (v0.6.1)

- Module imports now use the same growth-safe bounded read as the main file (was: metadata check + unbounded `read_to_string`).
- Import nesting depth is capped (`BOLI_MAX_IMPORT_DEPTH`, 32) in both engines — no silent stack exhaustion from long acyclic chains.
- Nested relative imports resolve against the importing module's directory (was: top-level file's directory).
- VM bytecode operands are 64-bit; > 65,535 constants/names/items can no longer wrap.
- Interpreter and VM parameter binding unwind cleanly on error paths (no scope/trace/depth leaks); VM range-loop scope unwinds on every exit (error included).
- Built-ins enforce the configured string cap on their outputs; `pucho` enforces the cap even when the newline arrives in the same chunk.
- Built-ins consume the configured list cap via `max_list_items` (chaabiyan) as well.

## Built-ins (v0.4.0) attack surface
`lambayi`, `upar`, `neeche`, `text`, `sankhya` — all pure internal Rust; every allocation they make is bounded by the existing string caps (`BOLI_MAX_STRING_BYTES`), list caps (`BOLI_MAX_LIST_ITEMS`) and live-scope budget (`BOLI_MAX_TOTAL_BYTES`). No I/O, no filesystem, no network. `sankhya` rejects inf/NaN.
- `cargo clippy --all-targets --all-features -- -D warnings` — clean
- Reproduced & confirmed fixed: 200k-term condition SIGSEGV → Galti; 3,000-deep expression nesting SIGSEGV → Galti (0.2.0); 20,000-item list → Galti; literal bypass → Galti; 11 MiB file → Galti; `LIKHO` keyword equivalence; REPL multiblock.

## Recommendations
1. Keep `[dependencies]` empty; vet any addition hard.
2. If BOLI gains file/network primitives later (see UPGRADE_ROADMAP), every one needs its own `BOLI_MAX_*` limit and an entry here.
3. Keep `BOLI_IMPORT_ROOT` documented wherever `laao` is documented; any future file capability must reuse the same canonicalized-root pattern or get its own confinement variable.
4. Publish this matrix in the security section of the README/website; the current authoritative status page is [TEST_RESULTS.md](TEST_RESULTS.md).
