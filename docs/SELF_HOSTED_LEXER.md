# Self-Hosted Lexer (Experimental)

## Status

`selfhost/lexer/` contains a constrained, in-memory lexer written in BOLI. It is an additive experiment and documentation target for self-hosting design. It is **not** wired into the default CLI, parser, interpreter, VM, or build. Rust remains authoritative.

Goal: reproduce Rust lexer tokens and diagnostics on a golden differential corpus, while explicitly recording the capabilities BOLI still lacks.

## Files

```text
selfhost/lexer/
  errors.boli     result/error map constructors
  token.boli      token maps and canonical token formatter
  reader.boli     cursor/state helpers and list push
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
laao "lexer.boli"

res rakho lexer_lex source max_source max_string max_tokens
```

The source is always an in-memory text value. No file path or file handle is accepted.

`run.boli` currently calls `lexer_lex` at import time, so the differential harness imports the four implementation modules directly instead of importing `run.boli`.

## Running locally

From the repository root:

```bash
export PATH=/data/data/com.termux/files/usr/bin:$PATH

cargo run --quiet -- run "selfhost/lexer/demo.boli"
cargo test --test phase3_lexer --locked -- --nocapture
```

The demo prints canonical tokens:

```text
Likho||1|1
Number|5|1|7
Newline||1|8
Eof||2|1
```

## Design notes

### Explicit state

BOLI assignment writes only to the innermost scope, and functions cannot capture outer mutable state. Therefore the cursor is a map `{line, col, pos}` that is returned from every helper and threaded explicitly through `lexer_lex`. The public result keeps a `state` field for future incremental use, but `lexer_ok` and `lexer_fail` currently set it to `khaali`. This is a deliberate additive compatibility choice, not a claim of exposed incremental state.

### No nested calls

BOLI does not support nested builtin/user function calls as arguments. The implementation uses named temporaries for results such as current character, peeked character, pushed state maps, and validation maps.

### Sequential conditions

`agar ... to` / `nahi to` / `khatam` behaves as an if/else chain only when written that way. The text scanner uses a `handled` flag so EOF, quote, newline, escape, and ordinary-character branches are mutually exclusive. This prevents falling through from a handled branch into the generic append branch.

### Keyword tables

Keyword recognition uses BOLI maps named `*_info`, for example `likho_info`. Each maps a keyword to `hissa "kind"`, `hissa "value"`, and `hissa "len"`. All keyword values are empty text so canonical output matches Rust unit tokens.

### Text escape pipeline

The lexer consumes source escapes `\"`, `\\`, and `\n`. It escapes token text only when producing canonical dumps. Tab and CR skip rules use `unescape_text "\\t"` and `unescape_text "\\r"`, because BOLI string literals themselves decode only `\"`, `\\`, and `\n`.

### Infinite-number pre-check

`lexer_is_finite_text` strips sign and leading zeros, then compares the integer part against Rust's first non-finite 309-digit decimal boundary before calling `sankhya`. This avoids the interpreter's non-finite parse error while matching Rust's `line:col:message`. Negative numbers are recognized only when `-` is immediately followed by a digit.

## Differential harness

`tests/phase3_lexer.rs`:

- runs the Rust lexer directly;
- builds a BOLI source string with embedded test input;
- invokes the BOLI lexer through the CLI runner;
- canonicalizes both outputs to `KIND|escaped-value|line|col` text;
- compares errors as `line:col:message`;
- covers keywords, numbers, identifiers, strings, escapes, comments, Unicode text, whitespace, tabs, CR, limits, error cases, negative-number boundaries, finite-number boundaries, UTF-8 byte caps, BOLI error kinds, and one documented Unicode identifier gap.

`boli_literal` escapes `\`, `"`, and newline, and leaves CR/tab raw in generated BOLI literals. Earlier plans to inject `unescape_text` into generated source were superseded by this direct approach.

## Known limitations

- `lexer_byte_length` now matches Rust's UTF-8 byte caps, but it walks every character and inherits BOLI's evaluation-step cost on very large sources.
- Unicode identifier recognition approximates Rust with ASCII plus the curated mark/Indic ranges from `src/lexer.rs`; scripts outside that set, such as Greek, are still rejected. This is verified by `phase3_documented_unicode_identifier_gap`.
- Errors are maps; Rust `BoliError` has no `kind` field, so `kind` is documentation-only for now.
- Public `state` is always `khaali`, even though internal helpers thread a cursor map.
- No byte offsets/spans are produced.
- No file I/O is used or permitted in this phase.
- No Phase 4 parser consumption or BOLI bytecode integration exists yet.
- This is not a self-hosting milestone by itself. See `docs/PHASE3_REPORT.md` and `docs/SELF_HOSTING_ROADMAP.md`.

## Safety

The lexer accepts explicit limits from the caller. Tests use widened limits only to avoid unrelated caps while testing lexing behavior. Production-style callers should pass the same defaults used by `src/limits.rs`.

## Rollback

Because this is additive, rollback is removal of `selfhost/lexer/`, `tests/phase3_lexer.rs`, and the Phase 3 docs. No existing Rust source, CLI behavior, parser path, or default test needs to be reverted.
