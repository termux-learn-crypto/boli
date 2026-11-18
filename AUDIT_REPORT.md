# BOLI Audit Report (v0.1.0 → v0.2.0)

**Auditor:** BOLI maintainer audit (staff engineer review)
**Scope:** Full repository — `Cargo.toml`, `src/*.rs`, `examples/`, `install.sh`, `README.md`
**Date:** 2026-11-18
**Verdict:** Working interpreter with good bones. 3 bugs confirmed and must be fixed, plus design trade-offs to document. No security issues beyond "trusted input" assumptions. v0.2.0 work listed below.

Severity scale: **Critical** (wrong results / crash) · **High** (broken correctness or resource safety) · **Medium** (spec surprises, robustness) · **Low** (style, docs).

---

## Confirmed Bugs

### BUG-001 — Range loop leaks a scope when an error occurs inside the loop
- **File:** `src/interpreter.rs:199-211` (`Stmt::RangeLoop`)
- **Severity:** Medium
- **Reproduction:**
  ```boli
  naam rakho "x"
  1 se 3 tak
      likho xyz        # xyz undefined
  khatam
  ```
  `self.run_block(body)?` returns early with `Err`, but `pop_scope()` before the `?` is only on the `Wapas` path — the error path keeps the pushed `ginti` scope on the stack. One extra empty scope per erroring range loop. (Subsequent direct loop pushes are fine, so global corruption is limited, but scope depth grows and `ginti` semantics can leak into later code.)
- **Fix:** Capture result, always `pop_scope()`, then handle.

### BUG-002 — REPL block depth can go negative and never recovers
- **File:** `src/main.rs:155-158`
- **Severity:** Low
- **Reproduction:** REPL me `khatam` type karo bina block ke. `depth` −1 ho jata hai; phir `nikalo` bhi kaam nahi karta (`depth == 0` asal value se match nahi karta).
- **Fix:** Clamp depth at 0, print a Hinglish message on extra `khatam`, strip quoted strings before depth calculation.

### BUG-003 — `aur` and `ya` have the same precedence (left-associative)
- **File:** `src/parser.rs:407-428` (`parse_cond`)
- **Severity:** High
- **Reproduction:** `agar ginti 1 se bada hai ya ginti 3 barabar hai aur ginti 10 se chhota hai hai to` — parsed as `(A ya B) aur C` instead of the expected `A ya (B aur C)`. Standard boolean precedence: **`aur` binds tighter than `ya`**.
- **Fix:** Two-level parsing: `parse_cond → parse_and ((ya) parse_and)*`; `parse_and → comparison ((aur) comparison)*`. Condition evaluator unchanged.

### BUG-004 — Range loop uses float accumulation (`i += 1.0`) instead of a count
- **File:** `src/interpreter.rs:200-209`
- **Severity:** Medium
- **Impact:** Incremental float drift and termination edge cases (fractional bounds, huge bounds, `1e17 se 1e18 tak`). Not fatal with step `1.0` but not robust. Also no cap on iterations.
- **Fix:** Compute iteration count `floor(to − from) + 1`, loop `count` times with `from + i` as f64, and cap via global steps limit.

### BUG-005 — Zero-argument kaam cannot be called in expression position
- **File:** `src/interpreter.rs:298-300` (`Expr::Var` arm)
- **Severity:** Medium
- **Reproduction:**
  ```boli
  kaam jawab_do
      wapas do 42
  khatam
  likho jawab_do      # Galti: naam 'jawab_do' nahi mila
  ```
- **Fix:** `Expr::Var` falls back to zero-argument kaam when the variable does not exist.

---

## Verified Correct (reviewed, no change)

| Area | Notes |
|------|-------|
| Range loop scope pop on `wapas do` | popped before returning ✓ |
| Wapas outside kaam | rejected with Hinglish error ✓ |
| Division by zero | `b == 0.0` error ✓ |
| Comparison operators incl. `bada ya barabar` | parse and eval ✓ |
| String comparison `barabar` | handled by `values_equal` ✓ |
| Function arity mismatch | Hinglish error before executing ✓ (args *are* evaluated first — acceptable, documented) |
| Scope push/pop on kaam call | balanced ✓ |
| `kaam` as contextual identifier | documented behavior ✓ |
| Kaam duplicate definition | rejected ✓ |
| `likho` on closed pipe | silent exit 0 (fixed v0.1.1) ✓ |

---

## Risks & Design Trade-offs (accepted, documented — not bugs)

| # | Trade-off | Decision |
|---|-----------|----------|
| T-001 | `set()` updates nearest enclosing scope (Python-like). A function assigning to a name that exists globally mutates the global instead of creating a local. | Documented in spec; tests lock semantics |
| T-002 | Function call arguments are evaluated before arity checking | Documented |
| T-003 | Statement `kaam ...` position prefers function *definition* over calling a function named `kaam` | Documented |
| T-004 | Zero-argument kaam in expression: variable wins if both exist | Documented |
| T-005 | Condition operands never do call-lookahead (`ginti2 5 se chhota hai` = comparison, not call with arg 5) | Documented |
| T-006 | `parse_comparison` rejects mixed text/number comparisons | Existing behavior kept |
| T-007 | REPL depth heuristic is line-based; `khatam` inside string text still counts (now partially mitigated by string-stripping in REPL) | Documented |

---

## Phase 4 Feature Work (new, not bugs)

1. Negative numbers (unary `-` only immediately before a digit)
2. Better string escapes (`\n`, documented `\"`, `\\`)
3. **Lists**: `cheezein rakho 1 2 3` (multi-atom rakho), `hissa` keyword for indexing (1-based), `cheezein hissa 2 rakho 99` for assignment, list prints as `1, 2, 3`, `jodo` concatenates lists
4. **`ruk`** (break) and **`agla`** (continue), aliases `break` / `continue`

---

## Phase 5 Runtime Safety (missing → adding)

- Maximum execution steps (default 100,000, env `BOLI_MAX_STEPS`)
- Maximum kaam call depth (default 64, env `BOLI_MAX_CALL_DEPTH`)
- Maximum string size (default 1 MiB, env `BOLI_MAX_STRING_BYTES`)
- ~~Maximum source file size (default 10 MiB, env `BOLI_MAX_FILE_BYTES`)~~ — **was documented but never implemented; implemented in v0.3.0 as `BOLI_MAX_SOURCE_BYTES` in `run_file` (metadata check before read).**
- All violations are Hinglish Galti errors, never panics/SIGSEGV

---

## Phase 6 Documentation to Add

`LANGUAGE_SPEC.md`, `ARCHITECTURE.md`, `CHANGELOG.md`, `TESTING.md`, `CONTRIBUTING.md`, `ROADMAP.md`, README update (testing section, new features), Cargo.toml version bump to `0.2.0`.

---

## Phase 7 Verification Plan

`cargo fmt --check` (if rustfmt present), `cargo check`, `cargo test`, `cargo build --release`, `cargo clippy` (if present), all examples re-run, REPL smoke tests.

---

## Remaining Risks After v0.2.0

- Number is f64-only; integer overflow silently rounds (documented)
- No GC pressure concerns; `Value` clone-heavy but fine for this scale
- ~~Stack depth for expression nesting is not depth-checked~~ — **fixed in v0.3.0** (`BOLI_MAX_EVAL_DEPTH`, default 256; parser nesting `BOLI_MAX_PARSE_DEPTH`, default 64; condition chain `BOLI_MAX_CONDITION_DEPTH`, default 128 — a 200k-term `ya` chain no longer SIGSEGVs)
- Unicode width: identifiers accepted via `is_alphanumeric`, but normalization not performed (documented)

---

# v0.2.0 Re-verification & Resolution (2026-11-18)

All five Copilot findings verified against base commit `a5fca49` in an isolated `git worktree` (since removed). All fixed; 67 automated tests pass; all four example programs re-run clean.

## Verified on base (reproduced)

| ID | Reproduction on base | Outcome after fix |
|----|----------------------|-------------------|
| BUG 1 (P1) | `x rakho 7\ny rakho 9\np x y` → `Galti: line 7 me kaam 'x' nahi mila` (nested-call parse) | flat args, tests pass |
| BUG 2 (P2) | REPL after loop error: `likho ginti` printed leaked `1` | `naam 'ginti' nahi mila` |
| BUG 3 (P2) | `9007199254740992 se 9007199254740994 tak` looped forever printing the bound | `9007199254740992 9007199254740993 9007199254740994` |
| BUG 4 (P2) | `a ya b aur c` grouped `(a ya b) aur c` (left-assoc, same precedence) | `a ya (b aur c)` |
| BUG 5 (P2) | `pucho` accepted `nan`/`inf`/`Infinity` as numbers; literals up to 1e400+ silently overflowed to inf; `a*b` overflow produced `inf` | all finite: input stays text, literal rejected, arithmetic errors |

## Git diff summary vs base (a5fca49)

- `src/parser.rs` — flat argument parsing; `aur`/`ya` two-level binding; `hissa` contextual identifier; rakho-value list collection; `ruk`/`agla`; index parsing
- `src/interpreter.rs` — loop scope cleanup (always pop); current-scope-only `rakho`; iteration-count range loop; function depth limit; steps limit; string concat size limit; list values & `Index`; zero-arg kaam expression call; broken-pipe exit
- `src/lexer.rs` — negative number literals; Unicode marks in identifiers; finite literal validation; `\n` escape
- `src/ast.rs` — `List`, `Index` expressions; `Ruk`/`Agla`/`ListAssign` statements; `line_for_error`
- `src/main.rs` — REPL quote-aware depth counting and stray-`khatam` recovery
- `src/lib.rs` (new) — library crate; `src/main.rs` now binary-only
- `tests/boli.rs` (54 tests), `tests/cli.rs` (13 tests)
- `CHANGELOG.md`, `LANGUAGE_SPEC.md`, `TESTING.md`, `AUDIT_REPORT.md`

## Known limitations (not bugs)

- f64-only numbers (integers > 2^53 lose precision; finite only)
- Mixed text/number comparisons rejected (by design)
- `pucho` cannot distinguish "nan the string" from "not a number" — by design both become Text
- REPL depth heuristic is line-based (single-line `khatam` inside a multi-line block inside a string is counted; quoted text is stripped before counting)
