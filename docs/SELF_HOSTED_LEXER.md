# Self-Hosted Lexer (Experimental)

## Status

`selfhost/lexer/` contains a constrained, in-memory lexer written in BOLI. It is an additive experiment and documentation target for self-hosting design. It is **not** wired into the default CLI, parser, interpreter, VM, or build. Rust remains authoritative.

Phase 3.2 added full Unicode identifier parity, source spans, structured Rust error kinds, and a deterministic generated-case test. Parser readiness is **READY WITH LIMITATIONS**; see `docs/PARSER_INTERFACE_SPEC.md`.

## Files

```text
selfhost/lexer/
  errors.boli     result/error map constructors with spans
  token.boli      token maps, span fields, canonical token formatter
  reader.boli     byte-aware cursor/state helpers and token push
  unicode.boli    generated Unicode alphabetic/numeric range tables
  lexer.boli      keyword tables, helpers, and lexer_lex
  run.boli        module-level demo runner
  demo.boli       manual sample invocation
  tests/          reserved for BOLI-side fixtures
```

`tests/phase3_lexer.rs` is the Rust-side differential harness.

## API

```boli
laao "errors.boli"
laao "token.boli"
laao "reader.boli"
laao "unicode.boli"
laao "lexer.boli"

res rakho lexer_lex source max_source max_string max_tokens
```

The source is always an in-memory text value. No file path or file handle is accepted.

`run.boli` currently calls `lexer_lex` at import time, so the differential harness imports the implementation modules directly instead of importing `run.boli`.

## Running locally

From the repository root:

```bash
export PATH=/data/data/com.termux/files/usr/bin:$PATH

cargo run --quiet -- run "selfhost/lexer/demo.boli"
cargo test --test phase3_lexer --locked --quiet
```

The demo prints canonical tokens with spans:

```text
Likho||1|1|0|5|1|1|1|6
Number|5|1|7|6|7|1|7|1|8
Newline||1|8|7|8|1|8|1|9
Eof||2|1|8|8|2|1|2|1
```

## Design notes

### Explicit state

BOLI assignment writes only to the innermost scope, and functions cannot capture outer mutable state. The cursor is a map that is returned from every helper and threaded explicitly through `lexer_lex`. The public result keeps a `state` field for future incremental use, but `lexer_ok` and `lexer_fail` set it to `khaali`.

### Byte-aware cursor

`reader.boli` tracks `byte`, `line`, and `col`. `lexer_mark` captures the start of a token or error; `lexer_push` writes start/end byte offsets and start/end line/column onto every token.

### No nested calls

BOLI does not support nested builtin/user function calls as arguments. The implementation uses named temporaries for results such as current character, peeked character, pushed state maps, and validation maps.

### Sequential conditions

`agar ... to` / `nahi to` / `khatam` behaves as an if/else chain only when written that way. The text scanner uses a `handled` flag so EOF, quote, newline, escape, and ordinary-character branches are mutually exclusive.

### Keyword tables

Keyword recognition uses BOLI maps named `*_info`, for example `likho_info`. Each maps a keyword to `hissa "kind"`, `hissa "value"`, and `hissa "len"`. All keyword values are empty text so canonical output matches Rust unit tokens.

### Text escape pipeline

The lexer consumes source escapes `\"`, `\\`, and `\n`. It escapes token text only when producing canonical dumps. Tab and CR skip rules use `unescape_text "\\t"` and `unescape_text "\\r"`, because BOLI string literals themselves decode only `\"`, `\\`, and `\n`.

### Infinite-number pre-check

`lexer_is_finite_text` strips sign and leading zeros, then compares the integer part against Rust's first non-finite 309-digit decimal boundary before calling `sankhya`. This avoids the interpreter's non-finite parse error while matching Rust's `line:col:message`.

### Unicode parity

`unicode.boli` is a deterministic generated BOLI table with 775 alphabetic ranges and 148 numeric ranges, generated from Rust `char::is_alphabetic` and `char::is_numeric`. It adds no Rust dependency, FFI, new builtin, or runtime call to the BOLI lexer. Combining marks remain curated directly in `lexer.boli`.

## Differential harness

`tests/phase3_lexer.rs`:

- runs the Rust lexer directly;
- builds a BOLI source string with embedded test input;
- invokes the BOLI lexer through the interpreter;
- canonicalizes both token streams to `KIND|escaped-value|line|col|start_byte|end_byte|start_line|start_col|end_line|end_col`;
- canonicalizes errors to `kind|message|line|col|start_byte|end_byte|start_line|start_col|end_line|end_col`;
- covers keywords, numbers, identifiers, strings, escapes, comments, Unicode text, whitespace, tabs, CR, limits, error cases, negative-number boundaries, finite-number boundaries, UTF-8 byte caps, BOLI error kinds, Unicode identifier policy, and fixed-seed deterministic generated cases;
- uses 11 tests, including `phase3_unicode_identifier_policy` and `phase3_deterministic_generated_lexer_cases`.

`boli_literal` escapes `\`, `"`, and newline, and leaves CR/tab raw in generated BOLI literals.

## Known limitations

- `lexer_byte_length` walks every character and inherits BOLI's evaluation-step cost on very large sources.
- Public `state` is always `khaali`, even though internal helpers thread a cursor map.
- No BOLI parser consumes the token stream, and no BOLI AST exists.
- No file I/O is used or permitted in this phase.
- No Phase 4 bytecode or compiler integration exists yet.
- This is not a self-hosting milestone by itself. See `docs/PHASE3_2_REPORT.md` and `docs/SELF_HOSTING_ROADMAP.md`.

## Safety

The lexer accepts explicit limits from the caller. Tests use widened limits only to avoid unrelated caps while testing lexing behavior. Production-style callers should pass the same defaults used by `src/limits.rs`.

## Rollback

Because this is additive but now touches shared Rust types for spans/kinds, rollback requires reverting the Phase 3.2 source changes in `src/ast.rs`, `src/lexer.rs`, `src/parser.rs`, `src/vm.rs`, `src/interpreter/mod.rs`, and `tests/boli.rs` in addition to removing `selfhost/lexer/`, `tests/phase3_lexer.rs`, and the Phase 3 docs. No CLI behavior changed.
