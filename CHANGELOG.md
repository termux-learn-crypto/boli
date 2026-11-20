# Changelog

All notable changes to BOLI. BOLI follows [Semantic Versioning](https://semver.org/): breaking changes bump the minor digit (pre-1.0).

## [Unreleased]

### Experimental self-hosted lexer
- Added `selfhost/lexer/` and `tests/phase3_lexer.rs` as an additive, non-default BOLI lexer experiment.
- Phase 3.1 fixed negative-number lexing, finite-number prechecks, UTF-8 byte caps, and documented the remaining Unicode-identifier and public-state limitations.
- Current gate: 214 passed / 2 ignored; Rust remains authoritative.

## [0.6.5] — 2026-11-19

### Production-readiness packaging
- **Pinned toolchain & CI:** Rust 1.100.0 pinned via `rust-toolchain.toml`; CI matrix covers ubuntu, macOS and Windows.
- **Tag-gated releases:** release artifacts are built only on tags and published with SHA-256 checksums.
- **crates.io metadata:** `Cargo.toml` now carries `readme`, `keywords` and `categories` (edition 2024 and zero dependencies unchanged).
- **Smoke test:** one CLI production smoke test added.

### Internal
- 191 tests (4 unit + 114 library + 34 CLI + 39 VM). Gates unchanged: fmt/check/test/clippy -D warnings/release build.

## [0.6.4] — 2026-11-19

### Full-audit fix pass (15 findings)
- **Nested values are bounded:** `BOLI_MAX_VALUE_DEPTH` (default 64) rejects deep list/map values; display and byte accounting are iterative, so 5,000-deep input is a Hinglish Galti in both engines instead of a debug/release SIGSEGV.
- **Step budget is per program:** imports no longer reset `steps` in the interpreter; an imported module shares the caller's step budget, matching the VM.
- **Limit arithmetic is overflow-safe:** `u64::MAX` source/read limits use `saturating_add(1)` in CLI and file readers, so debug builds do not panic and release builds do not silently read nothing.
- **VM runtime semantics match the interpreter:** range loops use normal scope accounting, `kaam` definitions and `laao` effects happen only when execution reaches them, top-level `wapas do` errors, condition lines/hints/errors match, and uncalled `ruk`/`agla` bodies compile.
- **REPL and parser hardening:** escaped quotes and `#` inside strings no longer confuse the multiline scanner; line caps count content bytes; parse depth is shared across `aur`/`ya` and counts `hissa`; public `Parser::new` cannot panic; stuck `5-3` is a lex error; map-literal errors carry columns.
- **Installer/docs hygiene:** `install.sh` handles an unset `HOME`; the broken-pipe test now actually forces EPIPE; the security report no longer claims that no violation can ever be a panic.

### Internal
- 190 tests (4 unit + 114 library + 33 CLI + 39 VM). Gates: fmt/check/test/clippy -D warnings/release build.

## [0.6.3] — 2026-11-19

### Deep-audit hardening (third pass)
- **Bounded did-you-mean hints:** `nearest_name()` now uses `HashSet` dedup, identifier-length and candidate-count caps, length-difference pruning and a two-row bounded Levenshtein with early cutoff. Oversized names no longer allocate an unbounded matrix.
- **REPL multiline buffer cap:** `BOLI_MAX_REPL_BUFFER_BYTES` (default 1 MiB) bounds the whole incomplete block, not just each line. Overflow clears the partial block so the REPL stays usable.
- **Entry file is a loaded module:** `boli run x.hl` registers the canonical entry path in both the interpreter and VM module registries before execution; `laao "x.hl"` from the entry file is skipped instead of running top-level statements twice.
- **Coverage:** new unit tests for bounded distance/oversized identifiers, a CLI REPL buffer regression, and self-import tests for both engines.

### Internal
- 172 tests (4 unit + 108 library + 30 CLI + 30 VM). Gates: fmt/check/test/clippy -D warnings/release build.

## [0.6.2] — 2026-11-19

### Audit hardening (second pass)
- **Oversized `pucho` input is drained:** after an over-limit line is rejected, input through the next newline is consumed so the REPL/CLI cannot re-read the same bad line forever.
- **VM state is reset between programs and after errors:** `steps`, scope stack, scopes and value stack are returned to their baseline; a failed program no longer poisons a reused `Machine`.
- **Optional import confinement:** `BOLI_IMPORT_ROOT` confines `laao` to one canonicalized directory in both the interpreter and the VM; paths outside the root fail with a Hinglish Galti.
- **Differential coverage:** interpreter and VM are now compared on a shared corpus and on matching runtime errors; list/map index assignment regressions assert single-evaluation and clean stacks.
- **Docs refreshed:** security scope, limits matrix, roadmap, status page, test counts and historical-report labels brought up to date.

### Internal
- 167 tests (2 unit + 108 library + 28 CLI + 29 VM). Gates: fmt/check/test/clippy -D warnings/release build.

## [0.6.1] — 2026-11-18

### Security / audit hardening
- **Module import reads are growth-safe:** `laao` now uses the same bounded `take(max+1)` read as the main file (no metadata TOCTOU).
- **Import depth cap:** `BOLI_MAX_IMPORT_DEPTH` (default 32) in both engines — long acyclic chains fail with a Hinglish Galti instead of exhausting the stack.
- **Nested relative imports resolve against the importing module's directory** (interpreter and VM `laao`).
- **VM bytecode operands are 64-bit:** constants/names/list-sizes/call counts no longer wrap at 65,535.
- **Unwind-on-error:** interpreter and VM parameter binding both unwind scope/trace/call-depth on budget failure; the VM restores entry scope depth on any error (range loops included).
- **Built-ins honor the configured string cap** (`upar`, `neeche`, `text`, `chaabiyan` key bytes, list cap for keys); `sankhya` already capped.
- **`pucho` line cap applies even when the newline arrives in the same input chunk.** (Was: silently accepted.)

### Internal
- 159 tests (108 library + 27 CLI + 24 VM). Gates: fmt/check/test/clippy -D warnings/release build. Docs refreshed (README/SPEC/SECURITY_REPORT/TESTING/COMPLETE_AUDIT/ARCHITECTURE).

## [0.6.0] — 2026-11-18

### Added
- **Maps** (word-based syntax, no symbols): `mapa rakho map "k" v ...`; `mapa hissa "k"`/`mapa hissa "k" rakho v`; text keys; nesting; `barabar` equality; `chaabiyan` builtin for sorted keys; map bytes count toward the live-scope budget.
- **Modules** (`laao "file.hl"`): executes an imported `.hl` file once in the current environment; circular-safe; relative paths resolve against the importing file's directory; bounded by `BOLI_MAX_SOURCE_BYTES`; import errors reported as `file 'x.hl' me error: ...`.
- **Bytecode VM (experimental):** `boli --vm run x.hl` or `BOLI_VM=1 boli run x.hl`. Same language semantics: expressions, statements, `kaam` (recursion, zero-arg fallback, builtins), maps, lists, loops, `laao` (compile-time inline), the step/depth/memory limits, did-you-mean hints, runtime tracebacks. Tree-walker stays the default; VM is parity-tested on the example corpus (21 VM tests).
- **Did-you-mean hints:** unknown `naam`/`kaam` errors suggest the closest known identifier (edit distance).
- **Parser fix:** `hissa` indexing is left-associative — `a hissa 1 hissa 2` means `(a[1])[2]` (was `a[1[2]]`).

### Internal
- 147 tests (103 library + 23 CLI + 21 VM). `cargo fmt --check` / `cargo check` / `cargo clippy --all-targets --all-features -- -D warnings` / `cargo build --release` all green.
- Interpreter/VM share `value_size`, `eval_arith`, `eval::compare`, `index_get_pub`/`index_set_pub`, and the live-scope budget helpers in `src/scope.rs`.

## [0.5.0] — 2026-11-18

### Added
- **Runtime tracebacks:** runtime errors inside nested `kaam`s now print the call chain, innermost first: `kaam '<name>' ke andar (line N)`. `BoliError.trace` carries the frames; top-level errors are unchanged.
- **Interpreter module split:** `interpreter.rs` → `interpreter/{mod,value,eval}.rs` (no behavior change, public API unchanged).

### Internal
- 112 tests (93 library + 19 CLI). Gates green: fmt/check/test/clippy -D warnings/release build.

## [0.4.0] — 2026-11-18

### Added
- **Built-in kaams** (pure Rust, zero dependencies): `lambayi <text|list|khaali>` (Unicode chars / items), `upar`, `neeche` (full Unicode case mapping), `text <anything>` (value → display text), `sankhya <text>` (strict parse; invalid text / inf → Galti). User-defined kaam in naam se shadow ho sakta hai (user ki priority). `boli help` lists them.
- **Column diagnostics:** lexer/parser errors carry 1-based char columns (`BoliError.col`); CLI and REPL print the offending source line with a `^` caret (EOF-clamped). Runtime errors remain line-only by design.
- **Repository organization:** audit reports and logs moved to `docs/` with links fixed.
- `builtins.rs` extracted as its own module.

### Internal
- 108 tests (89 library + 19 CLI); gates green: fmt, check, test, clippy -D warnings, release build.
- API additions only: `BoliError { line, col }`, `line_col`, `builtins` module, `Token.col`.

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
