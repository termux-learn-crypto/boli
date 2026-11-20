# BOLI Parser Interface Spec

## Status

This document is a Phase 3.2 interface specification only. No Phase 4 parser, AST, compiler, or bytecode implementation was added.

Parser readiness for the token contract: **READY WITH LIMITATIONS**.

## Rust-facing input

The parser consumes `Vec<Token>` produced by `src/lexer.rs` or a compatible token producer.

```rust
pub struct Token {
    pub tok: Tok,
    pub line: usize,
    pub col: usize,
    pub span: Span,
}
```

Parser-facing guarantees:

1. The final token is `Eof`.
2. `Newline` tokens separate statements and are emitted for every LF.
3. Comments are removed before tokenization; their trailing newline remains.
4. `Token.line` and `Token.col` equal `span.start_line` and `span.start_col`.
5. Byte offsets are zero-based and end-exclusive; see `docs/TOKEN_CONTRACT.md`.
6. Synthetic EOF uses `Span::default()`-equivalent coordinates.
7. Unit keywords carry empty text; payload tokens are `Number`, `Text`, and `Ident`.

## Current Rust parser behavior

`src/parser.rs` was not changed except for the minimal `SYNTHETIC_EOF` token literal needed by the new `span` field; that field is a source-breaking Rust API change.

Existing parser behavior remains authoritative:

- recursive descent with two-token lookahead;
- no backtracking;
- no multi-error recovery;
- first parse error is returned immediately;
- `kaam` and `hissa` retain contextual handling;
- punctuation such as `; { } ( ) = : .` is not part of the language;
- AST nodes still store line only, not full spans.

## Error interface available to a future parser

Lexer errors now carry a typed kind and optional span:

```rust
pub struct BoliError {
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub message: String,
    pub trace: Vec<(String, usize)>,
    pub kind: Option<&'static str>,
    pub span: Option<Box<Span>>,
}
```

A future parser may use `kind` and `span` for structured diagnostics without changing the lexer. The span is boxed to keep `BoliError` below the `clippy::result_large_err` threshold; read it with `error.span.as_deref()`. Existing error constructors set both fields to `None`, so runtime output remains compatible, but existing struct literals must migrate to those constructors.

## BOLI-side token map

A BOLI parser should consume the maps described in `docs/TOKEN_CONTRACT.md`:

```text
kind value line col start_byte end_byte start_line start_col end_line end_col
```

It must not depend on `reader.boli`, `errors.boli`, `unicode.boli`, or any internal lexer helper. Only `lexer_lex` result maps are a supported boundary.

Required import order for the experimental lexer:

```boli
laao "errors.boli"
laao "token.boli"
laao "reader.boli"
laao "unicode.boli"
laao "lexer.boli"
```

## Parser design constraints

- BOLI assignment writes only to the innermost scope; parser state must be threaded explicitly through functions.
- BOLI has no enums or pattern matching; tagged maps are the intended AST representation.
- BOLI has no exceptions; a future parser should return `{ok, ast, error, state}` maps.
- Recursive descent is feasible, but the existing 100k-step and 64-call-depth defaults may need explicit build-time limits for a full BOLI parser.
- Span propagation to BOLI AST nodes is required for production-quality diagnostics but is outside Phase 3.2.

## Limitations preventing `READY`

- No BOLI parser exists.
- No BOLI AST representation exists.
- Rust AST still stores line only; full source spans are not propagated into AST nodes.
- The Rust parser does not yet emit typed parse-error kinds or spans.
- No parser differential corpus or golden AST dump exists.

## Non-goals

- No changes to `src/parser.rs` beyond the synthetic EOF literal.
- No AST changes.
- No compiler or bytecode work.
- No default CLI wiring to the self-hosted lexer.

## Decision

The token boundary is now stable enough to design a parser against, so readiness is **READY WITH LIMITATIONS**. It is not `READY` until a BOLI parser/AST exists and `NOT READY` no longer applies because spans and typed errors are now defined and verified.
