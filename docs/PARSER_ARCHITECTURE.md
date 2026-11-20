# BOLI Parser Architecture (Phase 4A Design)

## Status

This document is a design blueprint for a future self-hosted BOLI parser. Phase 4A did not create parser modules, AST constructors, or runtime support. Nothing here changes `src/parser.rs`, `src/ast.rs`, the lexer, CLI, or tests.

Input contract: `docs/TOKEN_CONTRACT.md`.
Grammar contract: `docs/PARSER_GRAMMAR.md`.
AST contract: `docs/PARSER_AST_SPEC.md`.
Error strategy: `docs/PARSER_ERROR_RECOVERY.md`.

## Design goal

Build a BOLI parser in BOLI that:

1. consumes only public `lexer_lex` result token maps;
2. reproduces the current Rust recursive-descent grammar exactly;
3. produces tagged-map AST values compatible with `docs/PARSER_AST_SPEC.md`;
4. returns explicit result maps instead of using exceptions;
5. respects all existing parse limits and deterministic ordering;
6. can be differentially tested against `src/parser.rs` using legacy line-only AST dumps.

## Chosen architecture: state-threaded recursive descent

The Rust parser is recursive descent with limited lookahead and no backtracking (`src/parser.rs:22-28`, `src/parser.rs:48-66`). The BOLI design keeps that shape because:

- BOLI has functions, recursion, lists, maps, and conditions sufficient for prefix grammar;
- a Pratt/precedence parser would invent a precedence model the language does not have;
- a table-driven LR parser would add generated tables and complexity with no compatibility benefit;
- flat call arguments and contextual `kaam`/`hissa` are easier to preserve with direct recursive functions.

## Public boundary

The parser must not call lexer internals. It receives the token list from the public result map:

```text
{
  "ok": sahi,
  "tokens": list of token maps,
  "state": khaali,
  "error": khaali
}
```

Required token fields, in order:

```text
kind value line col start_byte end_byte start_line start_col end_line end_col
```

If `ok` is `galat`, the parser returns that lexer error unchanged and does not parse.

## Parser state

Rust keeps `tokens`, `pos`, `depth`, `max_depth`, and `max_condition_depth` in `Parser` (`src/parser.rs:22-46`). BOLI assignment writes only the innermost scope, so an analogous state map must be passed into and returned from every parse function.

State schema (design only):

```text
{
  "tokens":       list of token maps,
  "pos":          number,
  "depth":        number,
  "max_depth":    number,
  "max_condition_depth": number,
  "condition_nodes": number,
  "errors":       list of parse error maps,
  "options":      map of parser options
}
```

Rules:

- `pos` never moves backwards.
- `peek` and `peek_at(offset)` never mutate `pos`.
- Out-of-range lookahead returns a synthetic `Eof` token with zero-length span.
- `depth` is incremented on entry to statement, atom, `atom_base`, and postfix `hissa` parsing, and decremented on exit.
- `condition_nodes` starts at `1` for each top-level condition and increments once per `aur`/`ya`.
- State must be returned even on error so callers can report position and diagnostics deterministically.

## Result shape

Initial compatibility target is first-error behavior, matching Rust. Proposed result map:

```text
{
  "ok": sahi | galat,
  "ast": Program map | khaali,
  "error": parse error map | khaali,
  "state": parser state map | khaali
}
```

A later multi-error mode may add an `errors` list, but it must be opt-in. Default output must remain first-error to avoid changing diagnostics or golden tests.

## Function decomposition

Names below are design names only; no BOLI functions are added in Phase 4A.

| Future function | Responsibility | Rust anchor |
|---|---|---|
| `parser_parse_program` | skip leading newlines, parse statements until EOF | `parse_program` |
| `parser_parse_block` | parse statements until `khatam`/`nahi`/EOF without consuming terminator | `parse_block` |
| `parser_parse_stmt` | depth guard and statement dispatch | `parse_stmt`, `parse_stmt_inner` |
| `parser_parse_likho` | `likho <expr>` | statement branch |
| `parser_parse_pucho` | `pucho name [prompt]` | statement branch |
| `parser_parse_arith_assign` | arithmetic statement form | statement branch |
| `parser_parse_agar` | `agar` / optional `nahi to` / `khatam` | statement branch |
| `parser_parse_while` | `jab tak` | statement branch |
| `parser_parse_range` | numeric range loop | statement branch |
| `parser_parse_kaam` | contextual `kaam` forms | statement branch |
| `parser_parse_wapas` | `wapas do [expr]` | statement branch |
| `parser_parse_name_stmt` | `rakho`, `map`, list assign, call | statement branch |
| `parser_parse_laao` | import statement | statement branch |
| `parser_parse_control` | `ruk`, `agla`, standalone terminator errors | statement branch |
| `parser_parse_expr` | arithmetic keyword or atom | `parse_expr` |
| `parser_parse_atom` | atom plus postfix `hissa` chain | `atom` |
| `parser_parse_atom_base` | literal, name/call, nested arithmetic | `atom_base_inner` |
| `parser_parse_arith_tail` | operator-specific tail words | `parse_arith_tail` |
| `parser_parse_cond` | `ya` chain | `parse_cond` |
| `parser_parse_aur` | `aur` chain | `parse_aur` |
| `parser_parse_comparison` | two operands + comparator + `hai` | `parse_comparison` |

Each function takes `state` and returns `{ok, node, error, state}` or an equivalent deterministic shape.

## Token-stream layer

A future `token_stream.boli` module should own:

- `peek(state)` — current token or synthetic EOF;
- `peek_at(state, offset)` — bounded lookahead or synthetic EOF;
- `next(state)` — clone current token and increment `pos` only when inside the token list;
- `skip_newlines(state)` — consume all `Newline` tokens;
- `expect(state, kind, label)` — exact token check with Hinglish error;
- `expect_line_end(state)` — `Newline+` or `Eof`, otherwise error;
- `expect_ident(state)` — `Ident`, `Kaam`, or `Hissa`;
- `atom_start(kind)` — predicate from `docs/PARSER_GRAMMAR.md`.

The synthetic EOF must match the Rust constant (`src/parser.rs:8-20`): kind `Eof`, empty value, line `1`, col `1`, zero-length span at line `1`, col `1`.

## Dispatch rules

BOLI has no pattern matching, so dispatch must be explicit ordered condition chains over token `kind`:

1. statement dispatch follows the order in `src/parser.rs:225-551`;
2. `kaam` contextual checks must occur in this order:
   - `kaam hissa ... rakho`
   - `kaam rakho ...`
   - `kaam` followed by literal token → call of function named `kaam`
   - otherwise function definition
3. name-led dispatch must occur in this order:
   - `name rakho map ...`
   - `name rakho ...`
   - `name hissa index rakho atom`
   - otherwise `KaamCall`
4. expression dispatch checks arithmetic keyword first, then atom base;
5. condition dispatch checks `ya`, then `aur`, then one comparison.

Order is part of compatibility. Reordering changes errors and AST shape.

## Limits and safety

Defaults from `src/limits.rs:4-17`:

- `BOLI_MAX_PARSE_DEPTH` = 64;
- `BOLI_MAX_CONDITION_DEPTH` = 128;
- `BOLI_MAX_CALL_DEPTH` = 64;
- `BOLI_MAX_STEPS` = 100,000;
- `BOLI_MAX_TOKENS` = 1,000,000;
- `BOLI_MAX_VALUE_DEPTH` = 64;
- `BOLI_MAX_STRING_BYTES` = 1 MiB;
- `BOLI_MAX_SOURCE_BYTES` = 10 MiB.

A full BOLI parser will likely need explicit build-time overrides for steps, call depth, and value depth. Those overrides are a later implementation decision and must not change normal program defaults.

Recursion must be bounded before calling BOLI recursion, otherwise a native stack overflow could occur before the Hinglish limit is reached.

## Proposed future module layout

Design-only layout; none of these files are created in Phase 4A:

```text
selfhost/parser/ast.boli          AST constructors and canonical dump helpers
selfhost/parser/token_stream.boli token cursor, peek/next, expect helpers
selfhost/parser/expr.boli         expression, atom, arithmetic parsing
selfhost/parser/cond.boli         condition parsing
selfhost/parser/stmt.boli         statement and block parsing
selfhost/parser/parser.boli       program orchestration
selfhost/parser/run.boli          imports and CLI-facing parse entry
```

Import rule: parser modules may import `errors.boli` and `token.boli` for shared contracts, but must not import `reader.boli`, `unicode.boli`, or `lexer.boli` internals. The lexer result map is the boundary.

## Determinism

- Iterate token lists by index, never by map iteration order.
- Iterate AST lists in source order.
- Iterate map keys only through BOLI's sorted map behavior; do not depend on insertion order.
- Error messages must be fixed Hinglish strings with token `describe()` text inserted at the same place as Rust.
- AST dumps must be canonical: stable field order, stable operator names, stable representation of `khaali` versus missing values.

## Testing strategy for a future implementation

1. Unit-table tests over token maps for `peek`, `peek_at`, `next`, synthetic EOF, `expect_line_end`.
2. Golden AST dumps for every statement and expression variant.
3. Differential tests against `src/parser.rs` using legacy line-only AST dumps.
4. Error tests that compare kind, message, line, col, and span where available.
5. Limit tests for parse depth, condition depth, token count, and nested AST value depth.
6. Contextual keyword tests for `kaam` and `hissa`.
7. Flat-call tests: `f a b` versus nested-call rejection.
8. Full-file corpus tests using `examples/*.hl`.

## Compatibility and migration rules

- Keep Rust parser as reference/bootstrap; do not delete it.
- Do not change default CLI wiring in the parser phase.
- Do not add new syntax to make parser implementation easier.
- Do not introduce punctuation grouping.
- Do not move to multi-error output by default.
- Do not call the VM stable because a BOLI parser exists; that is a separate milestone.

## Non-goals

- No parser implementation.
- No AST implementation.
- No bytecode compiler.
- No VM changes.
- No lexer changes.
- No CLI changes.
