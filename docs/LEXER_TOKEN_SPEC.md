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
- `max_source`: number, source-character cap.
- `max_string`: number, text-literal/word character cap.
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

`state` is the final cursor map: `{line, col, pos}`. It is returned on success and failure so callers can continue incremental work or diagnose position.

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
- `too_many_tokens`
- `text_too_large`
- `word_too_large`
- `unterminated_text`
- `invalid_escape`
- `number_too_large`
- `unexpected_character`
- `internal`

Diagnostic text matches `src/lexer.rs` exactly where applicable. Differential tests compare `line:col:message`; `kind` is retained for future typed-error work but is not part of current Rust parity because `BoliError` has no `kind` field yet.

## Lexing rules

- Whitespace: space, tab, and CR are skipped; LF emits `Newline`.
- `#` comments run to end of line; the newline itself is still emitted.
- Text literals use `"..."`. Supported escapes are `\"`, `\\`, and `\n`.
- Numbers are decimal digit runs with optional leading `-`, using Rust-compatible finite-number formatting.
- Word atoms are keywords or identifiers. ASCII letters/digits, `_`, and a curated Indic range are accepted.
- Every other character produces `unexpected_character`.
- `Eof` is always the final token.

## Limits

Limits are enforced before unbounded accumulation:

- source size checked before scanning;
- token count checked on every append;
- text and word length checked while scanning;
- finite-number pre-check avoids passing huge digit strings to `sankhya`.

BOLI has no byte-length primitive, so `lambayi` counts Unicode characters while Rust caps UTF-8 bytes. This is a documented approximation, not exact parity for multibyte source. See `docs/PHASE3_REPORT.md`.

## Non-goals

- No BOLI file I/O.
- No drop-in replacement for `src/lexer.rs`.
- No span/byte offsets yet; only `line` and `col`.
- No parser consumption yet.
- No claim that Phase 3/self-hosting is complete until parser and compiler stages exist.
