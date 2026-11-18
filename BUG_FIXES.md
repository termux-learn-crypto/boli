# BOLI v0.3.0 — Bug Fixes

Each entry: symptom → root cause → fix → verification. (Full audit table in COMPLETE_AUDIT.md.)

---

## FIX-1 — String literal bypasses `BOLI_MAX_STRING_BYTES` (v0.3.0, High)
**Symptom:** `BOLI_MAX_STRING_BYTES=1 boli run t.hl` with `likho "abcd1234"` printed the string.
**Root cause:** `Expr::Text(s) => Ok(Value::Text(s.clone()))` in `eval_expr_inner` skipped `checked_text`; only the `jodo` concat path validated size.
**Fix:** `Expr::Text(s, line) => checked_text(s.clone(), *line, self.max_string_bytes)` (interpreter.rs).
**Verification:** manual run (prints Galti with limit=1) + `literal_string_size_limit` unit test.

## FIX-2 — 200k-term `ya` condition → SIGSEGV (v0.3.0, Critical)
**Symptom:** `agar a a barabar hai ya a a barabar hai ya … (200,000×) to` segfaulted the interpreter.
**Root cause:** `parse_cond`/`parse_aur` accumulate `Cond::Ya/Aur` left-nested `Box` nodes with no chain bound; `eval_cond` recursed without a depth counter; AST drop recursed the same chain.
**Fix:** chain counter in `parse_cond`/`parse_aur` (error beyond `BOLI_MAX_CONDITION_DEPTH`, default 128), independent eval-side counter `cond_depth` in `eval_cond` (same limit). Depth error uses the condition's own line number.
**Verification:** 200k-term reproducer now exits with `Galti: line 2 me condition bahut lambi ho gayi (max 128 'ya')` in <1s; `condition_ya_chain_parse_limit`/`condition_ya_chain_eval_limit` tests. Fix at parse time also bounds AST drop glue.

## FIX-3 — Source file read unbounded (v0.3.0, High)
**Symptom:** `boli run big.hl` called `fs::read_to_string` without a size check; AUDIT_REPORT documented a (nonexistent) 10 MiB `BOLI_MAX_FILE_BYTES` limit.
**Root cause:** missing check in `run_file`; documentation/implementation mismatch.
**Fix:** metadata-length check before reading; `BOLI_MAX_SOURCE_BYTES` env, default 10 MiB (10 × 1024 × 1024). Oversized file → Galti + exit 1 before reading.
**Verification:** 11 MiB file → `Galti: file '…' bahut badi hai (11534336 bytes, max 10485760)`; normal files unaffected.

## FIX-4 — Lists without size limit (v0.3.0, Medium)
**Symptom:** `cheezein rakho <10,001+ literals>` or repeated `jodo c a aur b` grew lists without bound.
**Root cause:** `Expr::List` evaluated atoms with no count check; `eval_arith` Jodo list+list extended without a cap.
**Fix:** `BOLI_MAX_LIST_ITEMS` (default 10,000) — checked at list creation and on the concat result (`max_list_items` passed into `eval_arith`).
**Verification:** 20,000-item literal → Galti; concat of two 3-item lists under limit of 5 → Galti; 3-item list under limit → prints. `list_creation_limit`, `list_concat_limit`, `list_under_limit_works`.

## FIX-5 — `pucho` unbounded stdin buffer (v0.3.0, Medium)
**Symptom:** a single >1 MiB stdin line would be buffered fully regardless of `BOLI_MAX_STRING_BYTES`; invalid UTF-8 produced "input nahi mil raha".
**Root cause:** `stdin().read_line()` with no cap; UTF-8 errors mapped to a generic message.
**Fix:** chunked `BufRead::fill_buf` loop; buffer capped at `max_string_bytes`; oversized input → Galti (program errors out, offending line stays unread); invalid UTF-8 decoded lossy (U+FFFD).
**Verification:** oversized-input unit path exercised via code review; existing pucho CLI tests (negative infinity, NaN, huge-finite, normal) still pass unchanged.

## FIX-6 — REPL `jab tak` double-count depth (this cycle, Medium)
**Symptom:** an incomplete line `jab tak` pushed depth twice (first word `jab` + last word `tak`), trapping the REPL into extra `khatam`s.
**Root cause:** trailing-`tak` heuristic ran in addition to the leading-`jab` rule.
**Fix:** trailing-`tak` rule skipped when the line starts with `jab` (`main.rs`).
**Verification:** REPL smoke test (`likho … nikalo`) passes; CLI multiblock test passes.

## FIX-7 — Keyword case handling misdocumented (this cycle, Medium)
**Symptom:** LANGUAGE_SPEC §1 said keywords must be lowercase while the lexer lowercases words before matching (`LIKHO` is `likho`), and called literals case-sensitive — self-contradictory.
**Root cause:** spec drifted from the lexer (linter of record: `keyword(&word.to_lowercase())`).
**Fix:** SPEC §1 rewritten: keywords case-insensitive; identifiers case-sensitive; literals untouched.
**Verification:** `LIKHO "…"` runs; identifier case split unchanged.

## FIX-8 — Docs/test-count drift (this cycle, Low)
**Symptom:** README/CHANGELOG/AUDIT_REPORT claimed 67 tests; actual 71 (now 82). TESTING.md claimed 54 library tests.
**Root cause:** counts hard-coded in docs without CI enforcement.
**Fix:** all docs updated to 82 (67 library + 15 CLI); TESTING.md lists the release matrix (fmt/check/test/clippy -D warnings/release build).
**Verification:** `cargo test` prints 67 + 15.

## FIX-9 — Clippy/format gates not clean (df5c012, Low)
**Symptom:** `cargo fmt --check` failed with 118 diffs; clippy warned 2 collapsible_if.
**Fix:** one-time `cargo fmt` commit (no logic changes, verified by tests); collapsible ifs restructured; let-chain used where idiomatic.
**Verification:** `cargo fmt --check` clean; `cargo clippy --all-targets --all-features -- -D warnings` clean.

## FIX-10 — Zero-arg kaam vs variable error message (this cycle, Low)
**Symptom:** `likho x` with a two-parameter kaam `x` reported "naam 'x' nahi mila" instead of an arity error.
**Fix:** explicit arm: kaam exists with N params → "kaam 'x' ko N cheezein chahiye, seedha call karo"; missing kaam → original error.
**Verification:** `kaam_wrong_arity_error` and neighbors pass.

## FIX-11 — `Flow::Agla` lost its line number (this cycle, Low)
**Symptom:** top-level stray `agla` reported a hardcoded line 1.
**Fix:** `Agla(usize)` carries the statement line end-to-end like `Ruk(usize)`.
**Verification:** compile + `ruk`/`agla` tests pass; error messages now cite the real line.

## FIX-12 — Package metadata missing (this cycle, Low)
**Symptom:** Cargo.toml had no description/license/repository despite an MIT LICENSE file and public repo.
**Fix:** added `description`, `license = "MIT"`, `repository`.

## FIX-13 — Parser env configurability gaps (this cycle, Low)
**Symptom:** parse depth and condition depth were compile-time constants.
**Fix:** `BOLI_MAX_PARSE_DEPTH` (64) and `BOLI_MAX_CONDITION_DEPTH` (128) now env-tunable via `src/limits.rs`; `BOLI_MAX_EVAL_DEPTH` (256) also documented/tunable.
**Verification:** `Parser::with_limits` used in tests for deterministic small limits.

All fixes preserve backward compatibility: every previously valid program still runs identically; only over-limit behavior changed (crash → Hinglish error).
