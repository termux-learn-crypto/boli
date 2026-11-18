# Changelog

All notable changes to BOLI. BOLI follows [Semantic Versioning](https://semver.org/): breaking changes bump the minor digit (pre-1.0).

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
- `AUDIT_REPORT.md`, `LANGUAGE_SPEC.md`, `ARCHITECTURE.md`, `TESTING.md`.

## [0.1.0] — 2026-11-18

Initial public release: lexer, parser, AST, tree-walking interpreter, CLI + REPL, examples (namaste, calculator, calculator2, loop), README guide, installer script (`install.sh`), MIT license.
