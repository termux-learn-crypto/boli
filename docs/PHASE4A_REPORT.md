# BOLI Phase 4A Report — Parser Grammar, AST, Architecture, and Error Design

## 1. Scope and status

Phase 4A was a design-only phase. It inspected the current BOLI implementation and produced parser specifications for a future self-hosted parser.

Status: **Phase 4A complete; Phase 4B implementation readiness is READY WITH BLOCKERS.**

No parser implementation, AST implementation, lexer change, VM change, CLI change, Rust parser rewrite, new syntax, file deletion, or test fabrication occurred in Phase 4A. The working tree still contains the uncommitted Phase 3.2 changes from the previous phase.

Starting HEAD: `41e8905 BOLI Phase 3.1: audit and harden self-hosted lexer`. No commit or push was requested or performed.

## 2. Method

The design inspected:

- `src/parser.rs` — implemented grammar, lookahead, depth guards, error messages;
- `src/ast.rs` — AST variants, errors, spans, line helpers;
- `src/lexer.rs` — token inventory and token contract source;
- `src/interpreter/eval.rs`, `src/interpreter/mod.rs`, `src/vm.rs` — AST consumers;
- `src/builtins.rs`, `src/scope.rs`, `src/limits.rs`, `src/main.rs` — runtime constraints;
- `selfhost/lexer/` — current BOLI-side token boundary;
- `tests/boli.rs`, `tests/cli.rs`, `tests/vm.rs`, `tests/phase3_lexer.rs`, `tests/vm_corpus.rs`, `tests/vm_fuzz.rs`, `tests/vm_bench.rs`;
- `LANGUAGE_SPEC.md`, Phase 3.2 report, token contract, parser interface spec, self-hosting roadmap, and other current docs.

Every grammar rule in the new spec was checked against `src/parser.rs`, not inferred from conventional language syntax.

## 3. Audit findings

### 3.1 Grammar truth

- The parser is recursive descent with one- and two-token lookahead; it has no backtracking.
- Statements are newline-terminated; extra tokens on the same line are errors.
- Arithmetic is nested prefix Hinglish, not symbol infix precedence.
- `hissa` is a left-associative postfix index operator and also a contextual identifier.
- `kaam` is a keyword, variable name, call name, and definition marker depending on local lookahead.
- Function-call arguments are flat atoms; `f a b` means `f(a, b)`, not nested calls.
- Condition precedence is `ya` lower than `aur`; each comparison reads two operands.
- A comparison always ends with `hai`; `se` before the comparator is optional.
- Range loops start only with a `Number` token and do not store a loop-variable name in the AST.
- `map` pair syntax is parser-level and only appears after `name rakho`.

### 3.2 AST truth

- `Program` is a flat `Vec<Stmt>`.
- There are 9 expression variants, 3 condition variants, and 15 statement variants.
- Every AST node stores a line number only, not a full source span.
- Tokens and errors have six-field spans, but parse errors currently use `BoliError::line_col`, leaving `kind` and `span` as `None`.
- Interpreter and VM both exhaustively match all current AST variants; any future AST change requires updating both.

### 3.3 Runtime and self-hosting constraints

- BOLI assignment writes only the innermost scope; parser state must be threaded explicitly.
- BOLI has no enums or pattern matching; tagged maps are the intended AST representation.
- BOLI has no exceptions; result maps are required.
- Default limits are restrictive for a full parser: 100,000 steps, 64 call depth, 64 parse depth, 128 condition depth, 1,000,000 tokens, and 64 nested value depth.
- VM stabilization remains a separate prerequisite for compiler migration; this parser design does not override that rule.

## 4. Design produced

New documents:

- `docs/PARSER_GRAMMAR.md` — observed grammar, EBNF, lookahead rules, limits, and grammar-versus-doc conflicts.
- `docs/PARSER_AST_SPEC.md` — current AST inventory, tagged-map design, span policy, and runtime compatibility matrix.
- `docs/PARSER_ARCHITECTURE.md` — state-threaded recursive-descent design, module layout, token-stream boundary, and determinism rules.
- `docs/PARSER_ERROR_RECOVERY.md` — current fail-fast errors, proposed parser error kinds, structured diagnostics, and conservative recovery options.
- `docs/PHASE4A_REPORT.md` — this report.

Updated document:

- `docs/SELF_HOSTING_ROADMAP.md` — Phase 4A status, design decisions, and Phase 4B blocker note.

## 5. Compatibility decisions

1. Grammar source of truth remains `src/parser.rs`.
2. No new syntax is introduced.
3. Contextual `kaam` and `hissa` behavior is preserved.
4. Flat call arguments are preserved.
5. Arithmetic nesting replaces any invented precedence model.
6. Default parser diagnostics remain fail-fast and single-error.
7. Rust AST remains line-only; strict and legacy golden AST modes are required.
8. Public lexer result maps remain the only token boundary for a future BOLI parser.
9. Rust sources stay as reference/bootstrap; nothing is deleted.
10. VM and CLI behavior remain unchanged.

## 6. Blockers for Phase 4B

Phase 4B is **READY WITH BLOCKERS**, not fully `READY`, because:

- no BOLI AST representation exists yet;
- no parser differential or golden AST corpus exists yet;
- Rust parse errors do not yet emit typed kinds or spans;
- BOLI step/call/value-depth limits may need explicit build-time overrides;
- VM stabilization is still required before compiler migration;
- the tagged-map AST shape must be validated in a small spike before a full parser port.

These blockers do not invalidate the Phase 4A design. They define what must be resolved before implementation is called ready.

## 7. Test and validation plan for later phases

1. Token-stream tests: `peek`, `peek_at`, `next`, synthetic EOF, line-end handling.
2. AST construction tests for every expression, condition, and statement variant.
3. Differential tests against Rust parser using legacy line-only dumps.
4. Strict span golden tests once BOLI AST spans exist.
5. Contextual keyword tests for `kaam` and `hissa`.
6. Flat call argument tests.
7. Depth and condition-limit tests at boundary values.
8. Error message, line, col, and span parity tests.
9. Lexer-error passthrough tests.
10. Full corpus tests over `examples/*.hl`.

## 8. Final validation

The full gate was re-run after the Phase 4A documentation changes with:

```bash
export PATH=/data/data/com.termux/files/usr/bin:$PATH
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked --quiet
cargo build --release --locked
cargo run --release --locked --quiet -- version
cargo run --quiet -- run selfhost/lexer/demo.boli
git diff --check
```

Results:

- `cargo fmt --all -- --check`: clean.
- `cargo check --all-targets --locked`: passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed with zero warnings.
- `cargo test --all-targets --locked --quiet`: **216 passed, 0 failed, 2 ignored**.
- `cargo build --release --locked`: passed.
- Release binary: `BOLI version 0.6.5`.
- Self-hosted lexer demo:
  - `Likho||1|1|0|5|1|1|1|6`
  - `Number|5|1|7|6|7|1|7|1|8`
  - `Newline||1|8|7|8|1|8|1|9`
  - `Eof||2|1|8|8|2|1|2|1`
- YAML parse of `.github/workflows/ci.yml` and `.github/workflows/release.yml`: OK.
- `git diff --check`: clean.

No test result was fabricated and no failing result was hidden.

## 9. Git state

Phase 4A added only documentation files and modified `docs/SELF_HOSTING_ROADMAP.md`. The working tree still contains the uncommitted Phase 3.2 source and documentation changes. No commit or push was performed.

## 10. Decision

Phase 4A design work is complete.

Phase 4B implementation readiness: **READY WITH BLOCKERS**.

Full self-hosting readiness remains unchanged from the roadmap: **2/5** until VM stabilization, runtime primitives, sandboxed file/byte I/O, bytecode format, BOLI parser, BOLI AST helpers, BOLI compiler, self-compile, reproducibility, and Rust-free release are complete.
