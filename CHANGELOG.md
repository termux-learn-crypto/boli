# Changelog

All notable changes to BOLI. BOLI follows [Semantic Versioning](https://semver.org/): breaking changes bump the minor digit (pre-1.0).

## [0.3.1] — 2026-11-18

### Fixed (critical)
- **`BOLI_MAX_EVAL_DEPTH` was ignored** — the hardcoded `256` in `eval_expr` made the env var a no-op; error messages lied too. Now the configured limit (default 256) controls execution; `with_eval_depth_limit` and the env var both work (tests + CLI env test).
- **Lexer memory:** `source.chars().collect()` duplicated the source as a char vector and literals were allocated before validation. Lexer now walks a two-char sliding cursor; source/literal/identifier bytes are capped at lex time (`BOLI_MAX_SOURCE_BYTES` / `BOLI_MAX_STRING_BYTES`); total tokens capped (`BOLI_MAX_TOKENS`, 1,000,000); violations are Hinglish Galti errors; Devanagari/Unicode identifiers unchanged.
- **Source file reading:** metadata check alone was TOCTOU-unsafe and `read_to_string` allocated unboundedly. Now `read_capped()` honors `BOLI_MAX_SOURCE_BYTES` with a hard `take(max+1)` read; growth during read stops at the cap; oversized/invalid-UTF-8 files refuse with Galti (exit 1).
- **REPL input:** unbounded `read_line` replaced by chunked `read_repl_line` capped at `BOLI_MAX_REPL_LINE_BYTES` (64 KiB); oversized lines are drained and rejected without killing the REPL; multiline blocks, quoted text, `#` comments and `nikalo` all preserved.
- **Runtime memory budget:** new `BOLI_MAX_TOTAL_BYTES` (default 64 MiB) — live scope value bytes are counted on every set/pop (text bytes, list cells, scalars); replacement is not double-counted; scope exit frees its bytes; exceeding the budget is a deterministic Galti error (verified: string + list storage, scope-pop release, replacement semantics).

### Docs
- README/LANGUAGE_SPEC/SECURITY_REPORT/TESTING: limits tables expanded (11 limits); v0.3.1 notes; test counts corrected (95).
- docs/COMPLETE_AUDIT.md: v0.3.1 appendix with the 5 fixes and verification.

### Internal
- 95 tests (77 library + 18 CLI), `cargo fmt --check` / `cargo check` / `cargo clippy --all-targets --all-features -- -D warnings` / `cargo build --release` all green.
- Backward compatible: existing programs unchanged; only crash/unbounded paths became Galti errors.

## [0.3.0] — 2026-11-18

### Fixed
- **Init string literal size hole:** literals such as `likho "a very long literal"` bypassed `BOLI_MAX_STRING_BYTES` (only concatenation was checked). Literals are now validated at evaluation.
- **Condition chain stack overflow:** a condition with a huge `ya`/`aur` chain (e.g. 200,000 terms) built an unbounded left-nested `Cond` and crashed with SIGSEGV. Both parse-time and eval-time chains are now capped (`BOLI_MAX_CONDITION_DEPTH`, default 128) with a Hinglish Galti error.
- **Unbounded `pucho` input:** `pucho` read a full stdin line into memory without limit. Now read in bounded chunks capped by `BOLI_MAX_STRING_BYTES`; invalid UTF-8 decodes lossy (U+FFFD) instead of erroring.
- **Source file size:** `boli run file.hl` reads at most `BOLI_MAX_SOURCE_BYTES` (default 10 MiB); enforced with a metadata check before reading. (AUDIT_REPORT previously claimed a limit existed; it did not.)
- **List size:** lists are now capped (`BOLI_MAX_LIST_ITEMS`, default 10,000) at creation and for `jodo` list concatenation results; overflow is a Galti error.

### Added
- `src/limits.rs`: single source of truth for all safety limits (defaults + `BOLI_MAX_*` env override).
- `BOLI_MAX_PARSE_DEPTH` (64) — statement/expression nesting cap, env-tunable.
- `BOLI_MAX_CONDITION_DEPTH` (128) — `aur`/`ya` chain cap, parse and eval.
- `BOLI_MAX_EVAL_DEPTH` (256) — expression evaluation depth cap (deep `hissa` chains).
- Fuzz-ish regression test (deterministic xorshift; lexer/parser must not panic).
- CLI tests: bogus subcommand and extra `run` args exit non-zero with a Galti error.

### Changed
- `Expr::Text` evaluation now returns the literal via `checked_text` (same policy as concatenation).
- Cargo package metadata: `description`, `license`, `repository`.

### Docs
- LANGUAGE_SPEC keyword rule corrected: keywords match case-insensitively, identifiers case-sensitively. Limits table expanded (8 limits). Test counts corrected. AUDIT_REPORT false claim annotated.

### Internal
- `cargo fmt --check` is now a clean gate (one-time reformat commit `df5c012`).
- Clippy `-D warnings` clean; test suite 82 automated tests.

## [0.2.0] — 2026-11-18

### Fixed
- **BUG 1 (P1, parser):** call arguments parse as flat atoms — `f a b` now correctly evaluates as `f(a, b)` instead of the nested `f(a(b))`. A non-variable single-argument call like `f g` still binds `g` as an atom argument (use a zero-argument kaam or a variable for nesting).
- **BUG 2 (P2, interpreter):** range-loop scope is always popped, even when the loop body raises a runtime error. Verified: leak away in tests and in REPL (stray `ginti` no longer visible after failed loops).
- **BUG 3 (P2, interpreter):** range-loop iteration no longer depends on float `i += 1.0` progress — iteration count is computed from the bounds, so `9007199254740992 se 9007199254740994 tak` terminates exactly. Empty, negative, huge and mixed-fraction ranges are all well-formed; huge ranges terminate via the steps limit (§ safety).
- **BUG 4 (P2, parser):** `aur` (AND) now binds tighter than `ya` (OR) — `A ya B aur C` is `A ya (B aur C)`. See LANGUAGE_SPEC §4 and README.
- **BUG 5 (P2, interpreter/lexer):** NaN/±∞ can no longer enter the program: `pucho` parses only finite numbers (`nan`/`inf` stay as text), numeric literals are validated with `is_finite()`, and arithmetic results are checked with `is_finite()` with a Hinglish runtime error on overflow.

### Added
- Runtime safety limits, all configurable via environment variables: `BOLI_MAX_STEPS` (100,000), `BOLI_MAX_CALL_DEPTH` (64), `BOLI_MAX_STRING_BYTES` (1 MiB).
- `ruk`/`agla` (break/continue; `break`/`continue` as aliases).
- 1-based list indexing via `hissa`; list literal via multi-value `rakho`; list concatenation with `jodo`; list equality via `barabar`; list print as `1, 2, 3`.
- Zero-argument kaam callable in expressions (`likho jawab_do`).
- Negative numbers (leading `-` before a digit).
- `\n` escape in text literals; Unicode marks in identifiers.
- Interactive REPL: quote-aware depth tracking, proper message and recovery on stray `khatam`.
- Exit 0 on closed stdout (no Broken-Pipe panic; `boli ... | head` is safe).

### Changed
- **Semantics change:** `rakho` writes to the current (innermost) scope only; it no longer silently mutates an outer or global variable from inside a kaam (tested: `global_not_overwritten_by_kaam_body`).
- `ruk`/`agla` break/continue implemented (including `break`/`continue` aliases); REPL depth counting understands quoted text; silent exit 0 when stdout closes early (SIGPIPE safe).

### Internal
- `src/lib.rs` (library) + `src/main.rs` (binary) split so integration tests can use the parser/interpreter.
- Test suite: 67 automated tests (`cargo test`): lexer, parser, AST, interpreter (functions/scopes/lists/loops/limits), CLI end-to-end (REPL, binary broken-pipe, missing file, pucho input).
- `docs/AUDIT_REPORT.md`, `LANGUAGE_SPEC.md`, `ARCHITECTURE.md`, `TESTING.md`.

## [0.1.0] — 2026-11-18

Initial public release: lexer, parser, AST, tree-walking interpreter, CLI + REPL, examples (namaste, calculator, calculator2, loop), README guide, installer script (`install.sh`), MIT license.
