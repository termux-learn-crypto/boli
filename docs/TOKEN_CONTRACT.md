# BOLI Token Contract

## Purpose

`docs/TOKEN_CONTRACT.md` is the authoritative parser-facing contract for lexer output. It covers both the Rust reference lexer in `src/lexer.rs` and the experimental self-hosted lexer in `selfhost/lexer/`.

Status: **frozen for Phase 3.2; additive changes only**. The default CLI still uses the Rust lexer. No Phase 4 parser implementation is included.

## Span contract

A span is:

```text
{
  "start": zero-based UTF-8 byte offset, inclusive,
  "end": zero-based UTF-8 byte offset, exclusive,
  "start_line": one-based Unicode character line,
  "start_col": one-based Unicode character column,
  "end_line": one-based Unicode character line,
  "end_col": one-based Unicode character column
}
```

Rules:

- Byte offsets are zero-based and end-exclusive, matching Rust string slicing conventions.
- Lines and columns are one-based character positions, matching the pre-existing `Token.line` and `Token.col` fields.
- A newline token spans exactly the LF byte: `[start, start + 1)`.
- An EOF token is zero-length at the final cursor position.
- An unexpected non-ASCII character spans all of its UTF-8 bytes; `end_col` is still one character after `start_col`.
- The Rust struct is `boli::ast::Span`; the BOLI map uses the six field names above.
- `Token.line` and `Token.col` remain as compatibility aliases for `span.start_line` and `span.start_col`.

## Token map

```text
{
  "kind": text,
  "value": text,
  "line": number,
  "col": number,
  "start_byte": number,
  "end_byte": number,
  "start_line": number,
  "start_col": number,
  "end_line": number,
  "end_col": number
}
```

`kind` uses the Rust `Tok` variant name exactly. Payload rules are unchanged:

- `Number`: canonical `Value::Number(n).dikhane_laayak()` text.
- `Text`: decoded literal text, escaped only in canonical dumps.
- `Ident`: raw identifier text.
- Unit keyword/operator tokens: empty `value`.
- `Newline` and `Eof`: empty `value`.

## Canonical token dump

```text
KIND|escaped-value|line|col|start_byte|end_byte|start_line|start_col|end_line|end_col
```

Example for `likho 5`:

```text
Likho||1|1|0|5|1|1|1|6
Number|5|1|7|6|7|1|7|1|8
Newline||1|8|7|8|1|8|1|9
Eof||2|1|8|8|2|1|2|1
```

## Error map

```text
{
  "kind": text,
  "message": text,
  "line": number,
  "col": number,
  "start_byte": number,
  "end_byte": number,
  "start_line": number,
  "start_col": number,
  "end_line": number,
  "end_col": number
}
```

Rust now mirrors this with `BoliError.kind: Option<&'static str>` and `BoliError.span: Option<Box<Span>>`. The span is boxed to keep `BoliError` small enough for the existing `clippy::result_large_err` gate. Constructors without a kind or span set `None` for backward compatibility; the canonical dump dereferences the boxed span.

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

Canonical error dump:

```text
KIND|message|line|col|start_byte|end_byte|start_line|start_col|end_line|end_col
```

## Unicode identifier policy

Identifier start:

- `_`
- any scalar accepted by Rust `char::is_alphabetic`
- combining marks in the curated mark ranges used by both lexers

Identifier continuation:

- anything accepted as a start character
- any scalar accepted by Rust `char::is_numeric`

The BOLI lexer obtains full alphabetic/numeric parity from `selfhost/lexer/unicode.boli`, a deterministic generated range table with 775 alphabetic ranges and 148 numeric ranges. No Rust builtin, FFI, dependency, or runtime call was added to the BOLI lexer.

Import order is mandatory:

```boli
laao "errors.boli"
laao "token.boli"
laao "reader.boli"
laao "unicode.boli"
laao "lexer.boli"
```

## Result map

```text
{
  "ok": sahi | galat,
  "tokens": list | khaali,
  "state": map | khaali,
  "error": map | khaali
}
```

`state` remains reserved and is always `khaali` in public results.

## Compatibility and non-goals

- Existing `line`, `col`, `kind`, `value`, and BOLI result-map fields are preserved.
- BOLI result maps are additive; consumers that ignore span fields keep working.
- Rust `Token { span }` and `BoliError { kind, span }` are source-breaking struct-literal changes; use `Token::new`, `BoliError::line`, or `BoliError::line_col`. This ships as v0.7.0.
- `SYNTHETIC_EOF` in the Rust parser uses a zero-span value; parser logic is unchanged.
- No AST format, parser rules, bytecode format, VM opcode, or CLI behavior changed in Phase 3.2.
