# BOLI — Complete Source-Code Audit (v0.3.0)

> Historical base report — kept for traceability. Current status: [TEST_RESULTS.md](TEST_RESULTS.md) and the addenda below.

**Date:** 2026-11-18 · **Auditor:** staff-engineer review (this session) · **Base:** v0.2.0 + fix `5d9609e` → **Result:** 6 issues verified & fixed (see BUG_FIXES.md), all green gates, v0.3.0 released.

Scope: `Cargo.toml`, `src/*.rs`, `tests/*.rs`, `examples/`, `install.sh`, all docs, and the README claims.

---

## Severity scale
Critical (crash/memory) · High (correctness) · Medium (robustness/limits) · Low (docs/style)

## Baseline (before any modification)
- `cargo check` ✓ · `cargo test` 71 pass · `cargo clippy --all-targets --all-features -D warnings` ✓ · `cargo fmt --check` ✗ (118 pre-existing diffs)

## Lexer (`src/lexer.rs`)
| Area | Finding | Severity | Verdict |
|---|---|---|---|
| Tokenization | Keywords lowercased before match → keywords are **case-insensitive** (`LIKHO` works). Spec said otherwise (confusing, self-contradictory line). | Medium (docs) | SPEC fixed |
| Unicode | Identifiers accept Unicode letters + mark ranges incl. Devanagari; no NFC normalization (é vs e+́ are distinct identifiers). | Low | Documented, kept |
| Escapes | Only `\"`, `\\`, `\n`; other backslashes pass through literally (e.g. `\t` stays `\t`). | Low | Documented |
| Numbers | Leading `-` only directly before a digit; finite-only; `"1e400"` overflow → Galti error. No scientific notation (`1e5` → "'e5' yahan nahi aa sakta"). | Low | Documented limitation |
| Control chars | `\u{0007}` etc. → Galti error. | — | Verified by test |
| String literal | No literal size check at lex time (but file-size cap 10 MiB ≤ any practical literal; eval-time check added in v0.3.0). | — | Fixed at eval |
| Comments | `#` to EOL; `#` inside `"..."` is not a comment. | — | Correct |

## Parser (`src/parser.rs`)
| Area | Finding | Severity | Verdict |
|---|---|---|---|
| Recursion | No nesting guard pre-0.2.0 → SIGSEGV (2k nested `agar`). Fixed in 0.2.0 (limit), made env-tunable `BOLI_MAX_PARSE_DEPTH` in 0.3.0, now 64 default. | Critical | Fixed & tested |
| Condition chains | `ya`/`aur` are parsed as loops with unbounded left-nesting; 200k-term chain on a 3.8 MB line → SIGSEGV (verified). | Critical | Fixed: chain cap 128 (parse + eval) |
| Precedence | `aur` binds tighter than `ya` (v0.2.0 fix, AST-verified). | — | Regression test exists |
| Flat call args | `f a b` → `f(a, b)` (v0.2.0 fix). | — | Regression test exists |
| Errors | Hinglish errors with line numbers; `khatam` without opener and stray `nahi to` produce targeted messages. | — | Verified |

## AST (`src/ast.rs`)
| Area | Finding | Severity | Verdict |
|---|---|---|---|
| Consistency | Uniform `line` carried on all nodes; `Expr::line_for_error` added in 0.3.0 for depth-limit error reporting. | Low | Added |
| Memory | `Cond` left-nested boxes are dropped recursively — safe once chain ≤ 128 (parser cap). | Low | Covered by same fix |

## Interpreter (`src/interpreter.rs`)
| Area | Finding | Severity | Verdict |
|---|---|---|---|
| String literals | `Expr::Text(s) => Value::Text(s.clone())` — **bypassed** `BOLI_MAX_STRING_BYTES` (verified: printed with limit=1). | High | Fixed (checked_text) + test |
| List size | No length limit anywhere (creation, `jodo` concat) — unbounded memory via 10k+ item literal or concat doubling. | Medium | Fixed (`BOLI_MAX_LIST_ITEMS`, 10k) + tests |
| Pucho input | `read_line` with no size cap — unbounded stdin buffer. | Medium | Fixed (chunked `fill_buf`, capped, lossy UTF-8) |
| Arithmetic | f64, finite-only enforced at literals/input/result; div-by-zero → Galti. Overflow → Galti, not ±inf. | — | Verified (tests) |
| Recursion | Call depth cap 64; evaluation depth cap 256 (deep `hissa` chains), env-tunable. | — | v0.3.0 |
| Loops | Range loop: count-based iteration (no float drift), scope popped on all paths incl. error. | — | Fixed in 0.2.0, tests exist |
| Scopes | `rakho` writes innermost scope only (v0.2.0 semantics change, documented). | — | Tests lock behavior |
| Steps | Global step budget 100k; loops/body count steps; `def` of same kaam name rejected. | — | Verified |
| Range count | `to_i64_checked` guards huge bounds; huge ranges stop via steps limit. | — | v0.2.0 |

## CLI / REPL (`src/main.rs`)
| Area | Finding | Severity | Verdict |
|---|---|---|---|
| File read | `fs::read_to_string` unbounded (AUDIT_REPORT claimed a 10 MiB limit; **it did not exist**). | High (docs + code) | Fixed: `BOLI_MAX_SOURCE_BYTES` (10 MiB), metadata check before read |
| Args | `boli bogus` / `boli run a b` → Galti + exit 1. | Low | Fixed, tested |
| REPL depth | `jab tak ... tak` double-counted; stray `khatam` latched negative depth. | Medium | Fixed (0.2.0/0.3.0), CLI test exists |
| Broken pipe | Closed stdout → exit 0 (no panic). | — | CLI test exists |

## Every issue discovered (register)

| # | Issue | Severity | Status |
|---|---|---|---|
| A1 | Recursion/stack-overflow via nesting (parser & eval) | Critical | Fixed (0.2.0/0.3.0, tests) |
| A2 | Condition chain 200k → SIGSEGV | Critical | Fixed v0.3.0 |
| A3 | String literal bypasses size limit | High | Fixed v0.3.0 |
| A4 | Source file read unbounded; audit doc lied | High | Fixed v0.3.0 |
| A5 | List size unbounded | Medium | Fixed v0.3.0 |
| A6 | `pucho` unbounded stdin buffer | Medium | Fixed v0.3.0 |
| A7 | REPL `jab tak` double-depth count; negative-depth latch | Medium | Fixed (0.2.0 + this fix) |
| A8 | Keyword case handling misdocumented; identifiers case-sensitive | Medium | SPEC fixed |
| A9 | Docs claim 67 tests; actual 71 (now 82) | Low | Docs fixed |
| A10 | Clippy 2× collapsible_if; `cargo fmt` not clean | Low | Fixed (`df5c012` reformat, zero warnings) |
| A11 | Zero-arg kaam vs variable error message misleading | Low | Fixed (clear error) |
| A12 | `Flow::Agla` carried no line; top-level stray `agla` reported line 1 | Low | Fixed |
| A13 | Cargo.toml missing description/license/repository | Low | Fixed |
| A14 | `1e5` confusing error; f64-only numbers; integer >2^53 imprecise | Info | Documented (design) |
| A15 | No NFC normalization of identifiers | Info | Documented (design) |
| A16 | Escapes: only `\"` `\\` `\n` | Info | Documented (design) |

## Not issues (verified correct, no action)
- Scope push/pop pairing on error paths (loops, kaam, range) — tests lock it.
- Finite-number enforcement at all three entry points (literals, pucho, arithmetic).
- Zero-arg kaam expression fallback with variable shadowing precedence.
- `values_equal` for `barabar` on mixed types → `galat` without erroring.
- Install script handles Termux `$PREFIX/bin`, `/usr/local/bin`, `~/.local/bin` fallbacks, and `BOLI_INSTALL_DIR` override (tested).

## Remaining risks
- Numbers are f64-only (≈15–16 significant digits, exact integers ≤ 2^53).
- No GC; `Value` clone-heavy — fine at this scale.
- REPL depth heuristic is textual (quote/comment-aware but not full parsing) — malformed multiline input can confuse it; `nikalo`/`khatam` recovery exists.
- stdin line cap (`pucho`) drops/limits at 1 MiB by erroring the program; the remainder of an oversized line stays in stdin (harmless: error exits the program).
- No Unicode normalization.


---

# Appendix — v0.3.1 addendum (2026-11-18)

Five verified fixes, each reproduced before the change and re-verified after:

| # | Issue | Severity | Root cause | Fix |
|---|---|---|---|---|
| F1 | `BOLI_MAX_EVAL_DEPTH` ignored | High | hardcoded `256` in `eval_expr` guard; error message also hardcoded | `self.max_eval_depth` everywhere; tests `eval_depth_limit_is_configurable_and_enforced` (unit, limits 10/500) + `cli_env_eval_depth_is_respected` (env, 10/500) |
| F2 | Lexer allocates `Vec<char>` of whole source; literals unvalidated at lex time | High | convenience collection; no lex-time byte caps | sliding `Cursor` over `CharIndices`; caps: source/literal/identifier bytes + `BOLI_MAX_TOKENS`; Devanagari + escapes re-verified (`lexer_degrammar_still_works_after_cursor_rewrite`) |
| F3 | Source read relies on metadata length then unbounded `read_to_string` | High | metadata TOCTOU; large/growing files | `read_capped()`: metadata fast-path + `take(max+1).read_to_end()`; >max → Galti; invalid UTF-8 → Galti (`cli_source_bytes_boundary` + manual 11 MiB test) |
| F4 | REPL `read_line` unbounded | Medium | stdin line length unconstrained | `read_repl_line` chunked reader; `BOLI_MAX_REPL_LINE_BYTES` (64 KiB); oversized line drained+rejected, REPL survives (`cli_repl_oversized_line_is_rejected_and_repl_survives`) |
| F5 | No total memory budget: N variables × M MiB strings unbounded (steps-limited only) | High | missing accounting | live scope bytes budget `BOLI_MAX_TOTAL_BYTES` (64 MiB) with replace-safe set/pop accounting; `with_total_budget_bytes` for tests; 4 unit tests (string cap, list cap, scope-pop release, replacement not double-counted) |

**Verified:** 95/95 tests green; fmt/check/clippy(-D warnings)/release build green; manually re-ran all examples + REPL.
**Remaining limitations:** see SECURITY_REPORT.md / UPGRADE_ROADMAP.md.


---

# Appendix — v0.4.0 addendum (2026-11-18)

| # | Change | Notes |
|---|---|---|
| F6 | `docs/` reorganization | audit reports/logs moved; all links fixed; root clean |
| F7 | Standard library extracted (`src/builtins.rs`) | 5 builtins: lambayi/upar/neeche/text/sankhya; user kaam wins on name clash |
| F8 | Column diagnostics | `BoliError.col`, lexer/parser errors carry 1-based columns; CLI/REPL `^` caret rendering (EOF-clamped) |
| F9 | Repo layout for stdlib growth | `builtins.rs` is where all future builtins live; no `.hl`-level builtins, no unsafe |

Verified: 108/108 tests green; `cargo fmt --check` ✓; `cargo clippy --all-targets --all-features -- -D warnings` ✓; release build ✓; REPL caret rendering manually verified.


---

# Appendix — v0.5.0 addendum (2026-11-18)

| # | Change | Notes |
|---|---|---|
| F10 | Interpreter module split | `src/interpreter.rs` (942 lines) → `interpreter/{mod,value,eval}.rs`; no behavior change; `Environment`, `Value`, API unchanged |
| F11 | Runtime tracebacks | `BoliError.trace` — kaam call chain (name, call line, innermost-first display); top-level errors unchanged; `trace` popped on every exit path |

Verified: 112/112 tests green; example error output shows full chain:

```
Galti: line 2 me naam 'naam2' nahi mila
    kaam 'galti_wala' ke andar (line 6)
    kaam 'namaste' ke andar (line 10)
    kaam 'shuru_karo' ke andar (line 13)
```


---

# Appendix — v0.6.0 addendum (2026-11-18)

| # | Change | Notes |
|---|---|---|
| F12 | maps (word-based) | `mapa` + `map` keyword; hissa indexing; BTreeMap; budget-counted |
| F13 | `laao` module system | import-time inline, circular-safe, dir-relative paths, bounded by BOLI_MAX_SOURCE_BYTES |
| F14 | bytecode VM (experimental) | `--vm`/`BOLI_VM=1`; tree-walker remains default; 21 parity tests |
| F15 | did-you-mean hints | nearest-name suggestion on naam/kaam missing errors |
| F16 | parser associativity bug | `hissa` index is left-associative (was right-nested) |
| F17 | `chaabiyan` builtin | map keys, sorted, as text list |
| F18 | REPL/help EPIPE-safe | no panics on closed stdout pipes |

Verified: 147/147 tests green; examples run identically under both backends (`--vm` and tree-walker).


---

# Appendix — v0.6.1 addendum (audit fixes, 2026-11-18)

| # | Issue (review severity) | Root cause | Fix |
|---|---|---|---|
| A1 | Module imports unbounded during reads (High) | metadata check then unbounded `read_to_string` in both engines | shared `limits::read_source_capped` (`take(max+1)`); regression: `cli_laao_import_read_is_bounded` |
| A2 | Long acyclic import chain → stack exhaustion (High) | no import-depth limit in either engine | `BOLI_MAX_IMPORT_DEPTH` (default 32); `cli_laao_deep_import_chain_errors_cleanly` |
| A3 | `pucho` over-limit line accepted when newline is in same buffer (High) | only the no-newline branch checked the cap | `read_input_line` checks `buf.len() + pos > max_bytes` on the newline branch too; `cli_pucho_oversized_line_with_same_buffer_newline_is_caught` |
| A4 | Nested relative imports resolved against top-level dir, not importing module (Medium) | `source_dir` set once, never restored | save/restore around import in both engines; `cli_laao_nested_relative_path_resolves_against_importing_file` (interp + VM) |
| A5 | VM bytecode operands truncated to 16 bits (Medium) | `as u16` casts in `const_value`/`name`/counts | operands now `usize`; `vm_constants_over_u16_boundary_work` (70,000 distinct constants) |
| A6 | Early parameter-binding errors corrupt reusable env/VM state (Medium) | `?` early return after scope/trace/depth push | closure + unconditional cleanup; `param_bind_budget_failure_does_not_leak_scope_or_depth`, `repeated_bind_failures_do_not_block_recursion`, `vm_param_bind_budget_failure_does_not_leak_scope` |
| A7 | VM range-loop scope leaks on body error (Medium) | range scope popped only on normal exit | exec wrapper pops to entry scope depth on any error; `vm_failing_range_program_then_reuse_same_vm` |
| A8 | Built-ins could exceed string cap (Medium) | `upar`/`neeche`/`text`/`chaabiyan` built text unchecked | `checked_text` caps all built-in outputs; key bytes/list cap enforced in `chaabiyan`; `builtins_respect_string_cap*`, `chaabiyan_caps_keys_bytes` |
| A9 | Docs stale (Low) | test counts/tree drifted | README/TESTING/SPEC/ARCHITECTURE refreshed |

Full verification for that release: 159/159 tests green; all gates pass; both backends (interpreter + VM) verified for every fix. Current status: [TEST_RESULTS.md](TEST_RESULTS.md).

---

# Appendix — v0.6.2 addendum (second audit pass, 2026-11-19)

Source reviewed: `/data/data/com.termux/files/home/boli-audit-report.md` (`BOLI v0.6.0 — Full Static Audit Report`). Several report items were already fixed by v0.6.1; the remainder were fixed or explicitly documented here.

| # | Report finding | Status | Resolution |
|---|---|---|---|
| H-01 | `pucho` line cap bypass | Fixed in v0.6.1, re-verified | `cli_pucho_oversized_line_with_same_buffer_newline_is_caught` still green. |
| H-02 | Imported source reads not bounded | Fixed in v0.6.1, re-verified | Shared `limits::read_source_capped`; `cli_laao_import_read_is_bounded` still green. |
| H-03 | VM list assignment compiles RHS twice | Not reproducible in current tree | `Stmt::ListAssign` compiles index, value and target once; locked in by `vm_list_and_map_assignment_compiles_rhs_once_and_clears_stack`. |
| M-04 | `drain()` only consumed the current chunk | Fixed in v0.6.2 | `read_input_line_from` + `drain_through_newline`; two unit tests cover same-chunk and cross-chunk newlines. |
| M-05 | VM stack not reset between programs / on error | Fixed in v0.6.2 | `run_program` resets steps/scopes/stack to baseline; `call_kaam` restores stack; `stack_depth()` assertions added. |
| M-06 | Optional import confinement missing | Fixed in v0.6.2 | `limits::resolve_import_path` enforces `BOLI_IMPORT_ROOT` with canonicalized prefix checks in both engines; CLI test covers allow/deny. |
| L-01 | Security report scope stale (v0.3.0) | Fixed in v0.6.2 | SECURITY_REPORT rewritten for v0.6.2 scope, threat model and limits matrix. |
| L-02 | Roadmap stale (post-v0.3.0) | Fixed in v0.6.2 | UPGRADE_ROADMAP rebased on v0.6.2; done items moved out of planning. |
| L-03 | Historical report labels missing | Fixed in v0.6.2 | Added historical notes to AUDIT_REPORT, ARCHITECTURE_REPORT, BUG_FIXES, COMPLETE_AUDIT and V0.3.0_CHANGELOG. |
| L-04 | TEST_RESULTS mixed versions | Fixed in v0.6.2 | Rewritten as the current v0.6.2 status page with actual command output. |
| L-05 | VM step semantics not disclosed | Fixed in v0.6.2 | Documented as backend-specific in LANGUAGE_SPEC, SECURITY_REPORT, TEST_RESULTS and UPGRADE_ROADMAP. |
| Coverage | No differential interpreter/VM corpus | Fixed in v0.6.2 | `vm_matches_interpreter_on_core_corpus` and `vm_and_interpreter_agree_on_runtime_errors`. |

Verification: 167/167 tests green (2 unit + 108 library + 28 CLI + 29 VM); fmt, check, clippy -D warnings and release build all pass. See [TEST_RESULTS.md](TEST_RESULTS.md).
