# BOLI Phase 3.2 Report — Lexer Spans, Unicode, and Parser Interface

## 1. Scope and status

Phase 3.2 hardened the lexer boundary only. It added full Unicode identifier parity to the experimental BOLI lexer, source spans to Rust and BOLI tokens, structured lexer error kinds to Rust, expanded differential tests, and two parser-facing documents.

Status: **Phase 3.2 complete; parser readiness is READY WITH LIMITATIONS**. No Phase 4 parser, AST, compiler, bytecode, CLI wiring, or default-pipeline change was implemented.

Starting HEAD: `41e8905 BOLI Phase 3.1: audit and harden self-hosted lexer`. Working tree contains uncommitted Phase 3.2 changes; no commit or push was requested or performed.

## 2. Audit method

The audit inspected:

- Rust lexer, token, AST/error, parser, VM, interpreter, builtin, and limit code;
- all BOLI lexer modules and generated fixtures;
- Rust integration/differential test harnesses;
- current docs, roadmap, reports, README, TESTING, CHANGELOG, and git status.

Every claim in this report is backed by source, generated data, or a command result. No failing test was hidden and no fabricated result is reported.

## 3. Audit findings

Key Phase 3.1 baseline findings carried into Phase 3.2:

- BOLI identifiers used ASCII plus curated mark/Indic ranges, leaving a verified Greek `α` gap.
- BOLI tokens had only `line` and `col`; no byte offsets or end positions.
- BOLI error maps had `kind`; Rust `BoliError` had no typed kind or span.
- `state` in the public result was already reserved and always `khaali`; this was kept unchanged.
- General `byte_lambayi` did not exist; the lexer-local `lexer_byte_length` remained the correct minimal approach.
- Rust parser only needed the additive `Token.span` field to keep compiling; no parser logic needed changing.

## 4. Source changes

Rust:

- `src/ast.rs`: added `Span`, `BoliError.kind`, and `BoliError.span`, plus `BoliError::line_col_kind_span`.
- `src/lexer.rs`: added `LexErrorKind`, cursor byte/span tracking, span-aware token pushes, and typed span errors.
- `src/parser.rs`: added zero-span `SYNTHETIC_EOF` only.
- `src/vm.rs` and `src/interpreter/mod.rs`: initialized the new fields as `None`.
- `tests/boli.rs`: added `Span::default()` to four token literals.

Clippy initially flagged `result_large_err` because an inline `Option<Span>` grew `BoliError`. This was fixed by storing `Option<Box<Span>>`; no lint suppression was added.

BOLI:

- `selfhost/lexer/errors.boli`: span-aware error constructors.
- `selfhost/lexer/token.boli`: span-aware token maps and canonical formatter.
- `selfhost/lexer/reader.boli`: byte-aware cursor, `lexer_mark`, and `lexer_push`.
- `selfhost/lexer/lexer.boli`: Unicode-aware word rules and span-carrying token/error paths.
- `selfhost/lexer/run.boli`: imports `unicode.boli` before `lexer.boli`.
- `selfhost/lexer/unicode.boli`: new generated BOLI range table.

`Cargo.toml` was unchanged; no dependencies were added.

## 5. Unicode identifier policy

`selfhost/lexer/unicode.boli` was generated deterministically from Rust `char::is_alphabetic` and `char::is_numeric`.

- Alphabetic ranges: `775`.
- Numeric ranges: `148`.
- File size: `6476` lines, `439963` bytes.
- Word start: `_`, generated alphabetic scalar, or curated combining mark.
- Word continuation: word start plus generated numeric scalar.

The BOLI lexer makes no Rust call, uses no FFI, adds no builtin, and has no runtime dependency for Unicode classification. The generator lived outside the repository.

## 6. Span contract

A span is:

```text
start, end                zero-based UTF-8 byte offsets; end is exclusive
start_line, start_col     one-based Unicode character position
end_line, end_col         one-based Unicode character position
```

`Token.line` and `Token.col` remain compatibility aliases for `span.start_line` and `span.start_col`. Newline tokens span exactly one LF byte; EOF is zero-length at the final cursor position; an unexpected non-ASCII character spans all of its UTF-8 bytes while `end_col` advances by one character.

The frozen contract is in `docs/TOKEN_CONTRACT.md`.

## 7. Structured error contract

Supported lexer error kinds:

```text
source_too_large
token_limit
text_too_large
word_too_large
unterminated_text
number_not_finite
unexpected_character
stuck_minus
```

Rust canonical error dump:

```text
KIND|message|line|col|start_byte|end_byte|start_line|start_col|end_line|end_col
```

BOLI emits the same fields. Existing non-lexer constructors keep `kind: None` and `span: None`, so user-facing `Galti: ...` output remains compatible.

## 8. Differential test expansion

`tests/phase3_lexer.rs` grew from 10 to 11 tests. It now compares full token spans and full error kind/span fields, not only `line:col:message`.

Current tests cover:

- keywords, numbers, identifiers, strings, escapes, comments, Unicode text;
- whitespace, columns, CR/tab handling, and negative/finite boundaries;
- source, string, token, and UTF-8 byte caps;
- BOLI error kinds;
- Unicode identifier policy via `phase3_unicode_identifier_policy`;
- deterministic generated cases via `phase3_deterministic_generated_lexer_cases`.

`cargo test --test phase3_lexer --locked --quiet`: **11 passed, 0 failed**.

## 9. Deterministic generation

`deterministic_cases` uses fixed seed `0xB01_2026`, produces 24 cases of six fixed pieces each, and injects no randomness into production code. The generated corpus is repeatable and dependency-free. It exercises mixed words, numbers, signs, quotes, comments, whitespace, and Unicode pieces through both lexers.

## 10. Backward compatibility

- No CLI command, flag, prompt, or output format changed for normal programs.
- Existing `Token` fields remain.
- Public BOLI result fields remain `{ ok, tokens, state, error }`; `state` is still always `khaali`.
- Existing `BoliError::line` and `BoliError::line_col` still produce the old display text.
- Parser logic was not changed; only the synthetic EOF literal was updated.
- Resource limits and finite-number boundary behavior were preserved.

## 11. Resource limits and security

No new file I/O, network, process execution, raw byte value, FFI, global mutable state, or dependency was introduced. The generated Unicode table is static BOLI source. `lexer_byte_length` still walks characters to enforce byte caps, which is slower than Rust `len()` on very large sources but bounded and already tested. Error strings remain Hinglish and bounded by existing limits.

## 12. Documentation updates

New:

- `docs/TOKEN_CONTRACT.md`
- `docs/PARSER_INTERFACE_SPEC.md`
- `docs/PHASE3_2_REPORT.md`

Updated:

- `docs/LEXER_TOKEN_SPEC.md`
- `docs/SELF_HOSTED_LEXER.md`
- `docs/PHASE3_REPORT.md`
- `docs/PHASE3_1_AUDIT_REPORT.md`
- `docs/SELF_HOSTING_ROADMAP.md`
- `docs/TEST_RESULTS.md`
- `docs/SECURITY_REPORT.md`
- `docs/COMPLETE_AUDIT.md`
- `docs/ARCHITECTURE_REPORT.md`
- `docs/ERROR_HANDLING.md`
- `docs/RUNTIME_SPEC.md`
- `docs/PHASE2_REPORT.md`
- `docs/FILE_IO_SECURITY.md`
- `README.md`, `TESTING.md`, `CHANGELOG.md`

Older reports are retained as historical baselines; Phase 3.2 supersession notes were added where earlier claims would otherwise read as current.

## 13. Parser interface and readiness

Parser-facing guarantees are documented in `docs/PARSER_INTERFACE_SPEC.md`:

- final token is `Eof`;
- `Newline` separates statements;
- comments are removed but their newline remains;
- `Token.line`/`col` alias span start;
- payload tokens are `Number`, `Text`, and `Ident`;
- BOLI consumers should use only `lexer_lex` result maps.

Decision: **READY WITH LIMITATIONS**. The token boundary is stable enough to design a parser against. It is not `READY` because no BOLI parser/AST exists and the Rust AST still stores only line numbers.

## 14. Final validation

All commands passed after the source changes and documentation updates:

```text
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked --quiet
cargo build --release --locked
cargo run --release --locked --quiet -- version
cargo run --quiet -- run "selfhost/lexer/demo.boli"
YAML parse: .github/workflows/ci.yml, .github/workflows/release.yml
git diff --check
```

Results:

- `cargo test --all-targets --locked --quiet`: **216 passed, 0 failed, 2 ignored**.
- Release binary: `BOLI version 0.6.5`.
- Demo output:

```text
Likho||1|1|0|5|1|1|1|6
Number|5|1|7|6|7|1|7|1|8
Newline||1|8|7|8|1|8|1|9
Eof||2|1|8|8|2|1|2|1
```

## 15. Remaining limitations

- No BOLI parser, AST representation, or parser differential corpus.
- Rust AST nodes still store line only, not full spans.
- Rust parser does not emit typed parse-error kinds or spans.
- BOLI result `state` remains reserved and always `khaali`.
- No catchable structured error value in BOLI; `try`/`catch` do not exist.
- No general `byte_lambayi` builtin; only the lexer-local helper exists.
- The BOLI lexer is not wired into the default CLI or build.
- This is not self-hosting complete and is not a Phase 4 implementation.

## 16. Conclusion and commit status

Phase 3.2 achieved its lexer-boundary scope: full Unicode identifier parity, stable spans, typed Rust lexer errors, stronger deterministic differential coverage, and a documented parser interface. Parser readiness is **READY WITH LIMITATIONS**.

The working tree is intentionally dirty with Phase 3.2 changes. Review `git status` and `git diff`; commit or push only if explicitly requested.
