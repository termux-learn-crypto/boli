# BOLI — Architecture Report (v0.3.0)

> Historical snapshot — kept for traceability. Current status: [TEST_RESULTS.md](TEST_RESULTS.md) and [CHANGELOG.md](../CHANGELOG.md).

## Current structure

```
source.hl ──► lexer.rs ──► parser.rs ──► ast.rs (Program) ──► interpreter.rs (Environment) ──► stdout
                Tokens         recursive-descent      Stmt/Expr/Cond      scopes + kaam table
                                     │
                                     └────────────── limits.rs (all limits, env override)
```

| Module | Lines (approx) | Responsibility |
|---|---|---|
| `src/ast.rs` | ~180 | `Program`/`Stmt`/`Expr`/`Cond`, `BoliError` (line-numbered Hinglish), `Expr::line_for_error` |
| `src/lexer.rs` | ~290 | UTF-8 tokenizer; keywords (ci match), Unicode identifiers, finite numbers, text escapes, `#` comments |
| `src/parser.rs` | ~640 | Recursive-descent; newline-terminated statements; `aur`>`ya` precedence; flat call args; depth+chain guards |
| `src/interpreter.rs` | ~640 | Tree-walking evaluator: scope stack, kaam table, env-tunable limits, output-capture hook |
| `src/limits.rs` | ~25 | **Single source of truth** for defaults + `BOLI_MAX_*` env parsing |
| `src/main.rs` | ~210 | CLI (`run`/`help`/`version`) + REPL (quote/comment-aware block-depth heuristic) |
| `src/lib.rs` | 7 | Crate root exposing modules for tests |

## Modularity evaluation (asked: split interpreter into environment/evaluator/functions/loops/limits?)

**Decision: keep `interpreter.rs` as a single file; extract `limits.rs` only.**

Reasons:
1. The evaluator's state (scope stack, kaam table, step/depth counters, capture hook) is shared by every arm — `run_stmt`, `eval_expr`, `eval_cond`, `call_kaam` mutate the same `Environment`. A file split moves the same coupling across module boundaries (every helper needs `&mut Environment` regardless) without changing ownership. That is namespace churn, not modularity.
2. File size (~640 lines) is well within reviewable bounds for a single cohesive component; splitting raises merge-conflict surface for one person.
3. `limits.rs` *was* extracted because four consumers (lexer/parser/interpreter/main) need one consistent definition — a genuine deduplication, not cosmetic.
4. Public API (`boli::parser`, `boli::interpreter::Environment`, `boli::limits`) stays stable; v0.2.0-era tests compile unchanged.
5. Re-evaluate if and when a bytecode VM lands (see UPGRADE_ROADMAP.md) — a VM is a different execution strategy and *should* be its own module.

## Error model
One error type (`BoliError { line: Option<usize>, message }`), one display form (`Galti: line N me …`). Every layer (lex/parse/eval) produces it; nothing else escapes to the user except `std::process::exit(0)` on closed stdout (deliberate SIGPIPE behavior).

## Safety model
All recursion axes and resource growth are capped by env-tunable limits with Hinglish-error semantics (see SECURITY_REPORT.md for the matrix). `push_scope`/`pop_scope` are strictly paired on every exit path (tested). Kaam calls are table-based (no closures); recursion depth 64.

## Performance characteristics
Tree-walking interpreter, O(n) per statement, `HashMap` scope lookup, `Value` clone-on-read. Numbers f64. Fine for interactive/teaching workloads and programs ≤ a few thousand statements. Not a high-performance runtime — by design.

## Backward compatibility
All v0.1.0/v0.2.0 programs run unchanged. The only behavioral deltas are: previously crashing inputs now produce Galti errors, and the v0.2.0 `rakho` scoping semantics change (documented in CHANGELOG). Public library API additions only (`Expr::line_for_error`, builder methods `with_list_limit`, `with_eval_depth_limit`, `with_condition_depth_limit`, new `limits` module).

## Build/tooling
Rust edition 2024, zero dependencies. Gates: `cargo fmt --check`, `cargo check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --release` — all green on x86_64 + aarch64 (Termux) hosts. `install.sh` installs into `$PREFIX/bin` (Termux), `/usr/local/bin`, `~/.local/bin`, or `$BOLI_INSTALL_DIR`.
