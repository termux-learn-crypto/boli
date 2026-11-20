# BOLI Parser Errors and Recovery (Phase 4A Design)

## Status

This document describes current parser error behavior and a design for structured diagnostics. It is design-only. Phase 4A did not add error kinds, parser code, or recovery logic.

Current authority:

- `src/parser.rs` — actual parse errors.
- `src/ast.rs:17-88` — `BoliError` shape and display.
- `docs/TOKEN_CONTRACT.md` — lexer error kinds and span contract.
- `docs/PARSER_INTERFACE_SPEC.md` — parser-facing token guarantees.

## Current behavior

The Rust parser is fail-fast:

- `parse_program`, `parse_block`, and every helper use `Result<_, BoliError>`.
- The first error is returned immediately (`src/parser.rs:165-174`, `src/parser.rs:176-187`).
- There is no error list, synchronization marker, or partial AST recovery.
- On error, no statement after the failing token is parsed.
- `BoliError::line_col` is used for all current parse errors; `kind` and `span` stay `None`.
- `Display` prints `Galti: line N, col M me ...`, then optional `kaam` traceback lines (`src/ast.rs:72-85`).

Example current messages:

| Condition | Message text from `src/parser.rs` |
|---|---|
| wrong exact token | `yahan 'X' chahiye tha, par 'Y' mila` |
| bad line end | `'X' yahan nahi aa sakta, ek line me ek hi kaam hota hai` |
| bad identifier | `yahan naam chahiye tha, par 'X' mila` |
| bad `rakho` value | `rakho ke baad value chahiye thi, par 'X' mila` |
| bad statement start | `'X' yahan se statement nahi shuru ho sakta` |
| depth | `code bahut zyada gehra hai (max nesting N)` |
| expression depth | `expression bahut zyada gehra hai (max nesting N)` |
| condition depth | `condition bahut lambi ho gayi (max N 'aur'/'ya', yahan 'X')` |
| standalone `khatam` | `'khatam' bina kuch shuru kiye aaya, pehle agar/jab tak/kaam likho` |
| standalone `nahi` | `'nahi to' sirf agar ke andar use hota hai` |

## Error categories

Current parse errors fall into these categories:

1. **Structural expectation** — `expect` failed for `to`, `tak`, `hai`, `do`, `khatam`, `rakho`, `aur`, `se`, `ko`.
2. **Line termination** — extra tokens on the same line after a statement.
3. **Name expectation** — identifier position received another token.
4. **Value expectation** — `rakho`, call argument, arithmetic operand, or atom position received a non-atom.
5. **Contextual keyword misuse** — standalone `khatam`/`nahi`, or invalid `kaam` contextual form.
6. **Literal form errors** — `laao` without text, invalid map pairs, nested/empty map.
7. **Limit errors** — parse depth or condition chain length.
8. **Unexpected EOF** — missing block terminator or missing required token before EOF.

## Structured error shape for a future BOLI parser

Design-only map shape:

```text
{
  "kind":       text,
  "message":    text,
  "line":       number,
  "col":        number,
  "start_byte": number,
  "end_byte":   number,
  "start_line": number,
  "start_col":  number,
  "end_line":   number,
  "end_col":    number,
  "token_kind": text,
  "context":    text,
  "recovered":  sahi | galat
}
```

Rules:

- `line`/`col` mirror `start_line`/`start_col`.
- `span` fields follow `docs/TOKEN_CONTRACT.md`.
- `token_kind` is the `Tok` variant name of the offending token.
- `context` is a stable phase label such as `statement`, `expression`, `condition`, `block`, `map`, or `limit`.
- `recovered` is `galat` in first-error mode.
- Messages remain Hinglish and preserve current text where practical.

## Proposed parser error kinds

These are design names only; they are not implemented and are not lexer error kinds.

```text
parser_unexpected_token
parser_expected_token
parser_expected_line_end
parser_expected_ident
parser_expected_value
parser_expected_text
parser_unexpected_eof
parser_unmatched_khatam
parser_nahi_outside_agar
parser_missing_block_terminator
parser_parse_depth_exceeded
parser_condition_depth_exceeded
parser_empty_map
parser_nested_map
parser_map_missing_value
parser_bad_laao_path
parser_bad_arith_target
```

Mapping examples:

| Current case | Proposed kind |
|---|---|
| `expect(Tok::To, "to")` fails | `parser_expected_token` |
| extra token after statement | `parser_expected_line_end` |
| non-identifier in name position | `parser_expected_ident` |
| non-atom after `rakho` | `parser_expected_value` |
| `laao` without text | `parser_bad_laao_path` |
| parse depth counter exceeded | `parser_parse_depth_exceeded` |
| condition counter exceeded | `parser_condition_depth_exceeded` |
| EOF while block still open | `parser_missing_block_terminator` |
| `Khatam` at statement start | `parser_unmatched_khatam` |

Rust can adopt `BoliError::line_col_kind_span` later without changing `Display`; existing constructors keep `kind: None` and `span: None` (`src/ast.rs:29-69`).

## Recovery policy

### Default: none

The default future BOLI parser must match Rust's fail-fast behavior:

- stop at first error;
- do not synthesize AST nodes;
- do not skip tokens beyond the failure point;
- return only the partial state needed for diagnostics;
- keep `ok: galat` and `ast: khaali`.

This preserves current observable behavior and makes differential tests exact.

### Optional: conservative multi-error mode

A later opt-in mode may collect multiple errors with these rules:

1. Record the error and increment an error counter.
2. Stop recovery when the configured maximum error count is reached (default design: 20).
3. Synchronize only at `Newline` for statement-level errors.
4. Inside blocks, additionally synchronize at `Khatam` or `Nahi` without consuming them.
5. Never synthesize missing operands, missing keywords, or missing values.
6. Never report an error for a token already consumed as part of another error.
7. Mark cascade-suspect errors and drop them if their token position equals a previous error position.
8. Do not produce a partial AST in multi-error mode; produce an error report only.
9. Keep deterministic ordering by token position.
10. Keep first-error mode as the default for CLI and parity tests.

Recovery must not change accepted programs. It may only change how many diagnostics are produced in an explicitly requested mode.

## Synchronization details

### Statement level

- On statement-start error, skip tokens until the next `Newline`, `Khatam`, `Nahi`, or `Eof`.
- Do not consume `Khatam`/`Nahi`; let the enclosing block parser see them.
- At `Eof`, return immediately.

### Block level

- `parse_block` currently stops on `Khatam`, `Nahi`, or `Eof`.
- A missing `khatam` at `Eof` should be one `parser_missing_block_terminator` error at the EOF span.
- Do not auto-close blocks silently.
- Do not treat `nahi` outside `agar` as an else marker; keep `parser_nahi_outside_agar`.

### Expression level

- Arithmetic tails are rigid: `jodo` requires `aur`, `ghatao` requires `se`, `guna`/`bhaag` require `ko ... se`.
- Recovery should not guess a missing connector; report `parser_expected_token` and synchronize at line end.
- Postfix `hissa` requires an `AtomBase` index; failure is `parser_expected_value`.

### Condition level

- A comparison requires two operands, optional `se`, comparator, and `hai`.
- Do not infer operands from comparator tokens.
- On comparator failure, report `parser_expected_token` and synchronize at line end.
- Condition depth checks happen as connectors are consumed; do not continue building the chain after the limit error.

### Map level

- Empty map and nested `map` are immediate errors.
- A key without a value is an immediate error.
- Do not skip pairs; line-end synchronization is safer than trying to repair a pair list.

## Error output compatibility

- Current user-visible output starts with `Galti: line ...`.
- New parser error maps must be renderable into that same string.
- Adding `kind`/`span` must not change default text output.
- Canonical diagnostic dump for tests:

```text
KIND|message|line|col|start_byte|end_byte|start_line|start_col|end_line|end_col
```

This mirrors the lexer error dump in `docs/TOKEN_CONTRACT.md`.

## Test plan

1. **Message parity** — for every existing parser error branch, compare Hinglish message text against Rust.
2. **Position parity** — compare line and col of the offending token.
3. **Span correctness** — verify byte offsets and end line/col from token spans.
4. **Kind mapping** — verify proposed kind for each category above.
5. **Fail-fast parity** — ensure default mode returns exactly one error and no AST.
6. **EOF cases** — missing `khatam`, missing `do`, missing comparator, empty file after newline.
7. **Limit cases** — parse depth 64/65, condition depth 128/129, nested `hissa` chains.
8. **Recovery mode** — if implemented later, verify deterministic ordering, error cap, and no accept/reject changes.
9. **Lexer-error passthrough** — lexer failure is returned unchanged; parser must not wrap or relabel it.
10. **Golden dumps** — canonical diagnostic lines stored for regression.

## Risks

- Over-eager recovery can hide real syntax errors and change accepted programs.
- Span propagation for aggregate nodes could disagree with the offending token span; specs must state which span is used.
- Changing Rust parser errors to typed kinds may affect tests if message text is altered.
- BOLI step/call limits may be hit before a deep error is reported; this must be surfaced as a limit error, not a syntax error.

## Decision for Phase 4A

Design complete. Default parser error policy remains fail-fast with one error. Structured kinds and conservative multi-error recovery are future options only; no parser implementation or error-kind change is included in Phase 4A.
