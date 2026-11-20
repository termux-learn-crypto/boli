# BOLI Lexer Token Spec

## Purpose

`docs/LEXER_TOKEN_SPEC.md` freezes the wire format produced by the experimental self-hosted BOLI lexer in `selfhost/lexer/`. The format is map-based because BOLI has no structs or enums. Rust remains the reference implementation in `src/lexer.rs`.

Status: **Phase 3.2 hardened; additive and experimental**. The default CLI, parser, and interpreter still use the Rust lexer. The authoritative parser-facing contract is `docs/TOKEN_CONTRACT.md`.

## Entry point

```boli
laao "errors.boli"
laao "token.boli"
laao "reader.boli"
laao "unicode.boli"
laao "lexer.boli"

res rakho lexer_lex source max_source max_string max_tokens
```

Inputs:

- `source`: text, supplied in memory. The BOLI lexer never reads files.
- `max_source`: number, source UTF-8 byte cap.
- `max_string`: number, text-literal/word UTF-8 byte cap.
- `max_tokens`: number, maximum token count including `Eof`.

Import order is mandatory: `unicode.boli` must be loaded before `lexer.boli`.

Result map:

```text
{
  "ok": sahi | galat,
  "tokens": list | khaali,
  "state": map | khaali,
  "error": map | khaali
}
```

`state` is reserved for future incremental use and is currently `khaali` in every result; internal helpers do pass a cursor map, but `lexer_lex` does not expose it. `error` is `khaali` on success.

## Token map

Each token is:

```text
{
  "kind": "Likho" | "Number" | "Text" | "Ident" | "Newline" | "Eof" | ...,
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

Rules:

- `kind` is always text and uses the Rust `Tok` variant name exactly.
- Keyword/operator kinds carry empty `value`.
- `Number` carries Rust-compatible `Value::Number(n).dikhane_laayak()` text.
- `Text` carries the decoded literal text, escaped only for canonical dumps.
- `Ident` carries the raw identifier text.
- `Newline` and `Eof` carry empty `value`.
- `line` and `col` are compatibility aliases for `start_line` and `start_col`.
- Byte offsets are zero-based and end-exclusive; lines and columns are one-based character positions.

## Canonical dump

```text
KIND|escaped-value|line|col|start_byte|end_byte|start_line|start_col|end_line|end_col
```

`escaped-value` uses `escape_text`-style escaping for `\`, `"`, newline, CR, and tab. For `likho 5\n`:

```text
Likho||1|1|0|5|1|1|1|6
Number|5|1|7|6|7|1|7|1|8
Newline||1|8|7|8|1|8|1|9
Eof||2|1|8|8|2|1|2|1
```

## Kinds

Keyword/operator kinds (empty value):

```text
Likho Rakho Pucho Jodo Ghatao Guna Bhaag Agar Nahi To Khatam
Se Aur Ya Ko Tak Jab Bada Chhota Barabar Hai Kaam Wapas Do
Sahi Galat Khaali Hissa Map Laao Ruk Agla
```

Payload kinds:

- `Number`: canonical number text.
- `Text`: decoded string literal contents.
- `Ident`: identifier text.
- `Newline`: empty.
- `Eof`: empty.

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

Supported kinds:

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

Phase 3.2 gives Rust `BoliError` matching `kind` and boxed-span fields, so differential tests compare canonical token streams and canonical errors including kind and span. Unknown escapes remain literal, matching Rust; there is no `invalid_escape` kind.

## Lexing rules

- Whitespace: space, tab, and CR are skipped; LF emits `Newline`.
- `#` comments run to end of line; the newline itself is still emitted.
- Text literals use `"..."`. Supported escapes are `\"`, `\\`, and `\n`.
- Numbers are decimal digit runs with optional leading `-` only when the next character is a digit.
- Word starts accept `_`, Unicode alphabetic scalars per `lexer_is_unicode_alphabetic`, and curated combining marks.
- Word continuation additionally accepts Unicode numeric scalars per `lexer_is_unicode_numeric`.
- Every other character produces `unexpected_character`.
- `Eof` is always the final token.

## Unicode table

`selfhost/lexer/unicode.boli` is generated deterministically from Rust `char::is_alphabetic` and `char::is_numeric`, with 775 alphabetic ranges and 148 numeric ranges. It is plain BOLI data and branching; the BOLI lexer does not call Rust, FFI, a new builtin, or a runtime service. Curated combining-mark ranges remain in `lexer.boli`.

## Limits

Limits are enforced before unbounded accumulation:

- source size checked before scanning;
- token count checked on every append;
- text and word length checked while scanning;
- finite-number pre-check avoids passing huge digit strings to `sankhya`.

`lexer_byte_length` walks Unicode characters and counts UTF-8 bytes by scalar-value ranges, so `max_source`, `max_string`, and word caps match Rust's byte-based limits. The finite-number pre-check compares the integer part against Rust's first non-finite 309-digit decimal boundary before calling `sankhya`.

## Non-goals

- No BOLI file I/O.
- No drop-in replacement for `src/lexer.rs`.
- No parser consumption yet.
- No AST, compiler, bytecode, or default CLI wiring.
- No Phase 4 implementation.
