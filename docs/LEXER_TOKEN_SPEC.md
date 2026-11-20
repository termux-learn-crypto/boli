# BOLI Lexer Token Spec

## Purpose

`docs/LEXER_TOKEN_SPEC.md` freezes the wire format produced by the experimental self-hosted BOLI lexer in `selfhost/lexer/`. The format is map-based because BOLI has no structs or enums. Rust remains the reference implementation in `src/lexer.rs`.

Status: **experimental, additive only**. The default CLI, parser, and interpreter still use the Rust lexer.

## Entry point

```boli
res rakho lexer_lex source max_source max_string max_tokens
```

Inputs:

- `source`: text, supplied in memory. The BOLI lexer never reads files.
- `max_source`: number, source UTF-8 byte cap.
- `max_string`: number, text-literal/word UTF-8 byte cap.
- `max_tokens`: number, maximum token count including `Eof`.

Result map:

```text
{
  "ok": sahi | galat,
  "tokens": list | khaali,
  "state": map | khaali,
  "error": map | khaali
}
```

`state` is reserved for future incremental use. It is currently `khaali` in every result; internal helpers do pass a cursor map, but `lexer_lex` does not expose it. See `docs/PHASE3_1_AUDIT_REPORT.md` for the justification.

`error` is `khaali` on success.

## Token map

Each token is:

```text
{
  "kind": "Likho" | "Number" | "Text" | "Ident" | "Newline" | "Eof" | ...,
  "value": text,
  "line": number,
  "col": number
}
```

Rules:

- `kind` is always text and uses the Rust `Tok` variant name exactly.
- Keyword/operator kinds carry empty `value`, because Rust unit variants have no payload.
- `Number` carries Rust-compatible `Value::Number(n).dikhane_laayak()` text.
- `Text` carries the decoded literal text, later escaped only for canonical dumps.
- `Ident` carries the raw identifier text.
- `Newline` and `Eof` carry empty `value`.
- `line` and `col` are 1-based Unicode character positions, matching Rust `line` and `col`.

## Canonical dump

Token streams are compared in a normalized text form:

```text
KIND|escaped-value|line|col
```

`escaped-value` uses `escape_text`-style escaping for `\`, `"`, newline, CR, and tab. For example:

```text
Likho||1|1
Number|5|1|7
Newline||1|8
Eof||2|1
```

Keyword values are empty by design; this is required for Rust parity.

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
  "col": number
}
```

Current kinds:

- `source_too_large`
- `token_limit`
- `text_too_large`
- `word_too_large`
- `unterminated_text`
- `number_not_finite`
- `unexpected_character`

`stuck_minus` is produced when `-` directly follows a completed atom and is followed by a digit (for example `5-3`), matching Rust. A bare `-` or `-` followed by a non-digit produces `unexpected_character`; see `docs/PHASE3_1_AUDIT_REPORT.md`.

Diagnostic text matches `src/lexer.rs` exactly where applicable. Differential tests compare `line:col:message`; `kind` is retained for future typed-error work but is not part of current Rust parity because `BoliError` has no `kind` field yet. Unknown escapes are preserved literally as `backslash + character`, matching Rust; therefore no `invalid_escape` kind exists.

## Lexing rules

- Whitespace: space, tab, and CR are skipped; LF emits `Newline`.
- `#` comments run to end of line; the newline itself is still emitted.
- Text literals use `"..."`. Supported escapes are `\"`, `\\`, and `\n`.
- Numbers are decimal digit runs with optional leading `-` only when the next character is a digit, using Rust-compatible finite-number formatting.
- Word atoms are keywords or identifiers. ASCII letters/digits, `_`, and a curated Unicode mark/Indic range are accepted. Rust accepts the full Unicode `is_alphanumeric()` set; the remaining scripts are a documented gap.
- Every other character produces `unexpected_character`.
- `Eof` is always the final token.

## Limits

Limits are enforced before unbounded accumulation:

- source size checked before scanning;
- token count checked on every append;
- text and word length checked while scanning;
- finite-number pre-check avoids passing huge digit strings to `sankhya`.

`lexer_byte_length` walks Unicode characters and counts UTF-8 bytes by scalar-value ranges, so `max_source`, `max_string`, and word caps match Rust's byte-based limits. The finite-number pre-check compares the integer part against Rust's first non-finite 309-digit decimal boundary before calling `sankhya`. See `docs/PHASE3_1_AUDIT_REPORT.md`.

## Non-goals

- No BOLI file I/O.
- No drop-in replacement for `src/lexer.rs`.
- No span/byte offsets yet; only `line` and `col`.
- No parser consumption yet.
- No claim that Phase 3/self-hosting is complete until parser and compiler stages exist.
