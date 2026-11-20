# BOLI Parser AST Specification (Phase 4A Design)

## Status

This document specifies the AST surface that a future BOLI parser must produce. It is design-only. Phase 4A did not add parser code, AST code, new syntax, or runtime changes.

Authority order:

1. `src/ast.rs` — current Rust AST.
2. `src/parser.rs` — actual AST construction.
3. `src/interpreter/eval.rs` and `src/interpreter/mod.rs` — tree-walking consumer.
4. `src/vm.rs` — bytecode compiler consumer.
5. `docs/PARSER_GRAMMAR.md` — source grammar.

## Current Rust AST inventory

`Program` is `Vec<Stmt>` (`src/ast.rs:250-251`).

### Expression variants

From `src/ast.rs:132-148`:

| Variant | Fields | Produced by |
|---|---|---|
| `Number` | `f64`, `line` | number literal |
| `Text` | `String`, `line` | text literal |
| `Bool` | `bool`, `line` | `sahi` / `galat` |
| `Khaali` | `line` | `khaali` |
| `Var` | `String`, `line` | identifier / contextual `kaam` / `hissa` |
| `Arith` | `ArithOp`, `Box<Expr>`, `Box<Expr>`, `line` | nested `jodo`/`ghatao`/`guna`/`bhaag` |
| `Call` | `String`, `Vec<Expr>`, `line` | name followed by flat atom args |
| `List` | `Vec<Expr>`, `line` | multi-atom `rakho` value |
| `Index` | `Box<Expr>`, `Box<Expr>`, `line` | postfix `hissa` |

`ArithOp` is `Jodo`, `Ghatao`, `Guna`, `Bhaag` (`src/ast.rs:90-108`). `naam()` returns the Hinglish word.

### Condition variants

From `src/ast.rs:150-161`:

| Variant | Fields |
|---|---|
| `Cmp` | `left: Expr`, `op: CmpOp`, `right: Expr`, `line` |
| `Aur` | `Box<Cond>`, `Box<Cond>`, `line` |
| `Ya` | `Box<Cond>`, `Box<Cond>`, `line` |

`CmpOp` is `Bada`, `Chhota`, `BadaYaBarabar`, `ChhotaYaBarabar`, `Barabar` (`src/ast.rs:110-130`). `naam()` returns the Hinglish phrase.

### Statement variants

From `src/ast.rs:163-248`:

| Variant | Fields |
|---|---|
| `Likho` | `Expr`, `line` |
| `Rakho` | `name: String`, `value: Expr`, `line` |
| `Pucho` | `name: String`, `prompt: String`, `line` |
| `ArithAssign` | `target: String`, `op: ArithOp`, `left: Expr`, `right: Expr`, `line` |
| `Agar` | `cond: Cond`, `then_block: Vec<Stmt>`, `else_block: Option<Vec<Stmt>>`, `line` |
| `RangeLoop` | `from: Expr`, `to: Expr`, `body: Vec<Stmt>`, `line` |
| `WhileLoop` | `cond: Cond`, `body: Vec<Stmt>`, `line` |
| `KaamDef` | `name: String`, `params: Vec<String>`, `body: Vec<Stmt>`, `line` |
| `KaamCall` | `name: String`, `args: Vec<Expr>`, `line` |
| `Wapas` | `value: Option<Expr>`, `line` |
| `Ruk` | `line` |
| `Agla` | `line` |
| `Laao` | `path: String`, `line` |
| `ListAssign` | `target: String`, `index: Expr`, `value: Expr`, `line` |
| `MapRakho` | `name: String`, `pairs: Vec<(Expr, Expr)>`, `line` |

## Line and span policy

Current state:

- Every AST node stores only `line: usize`.
- `Expr::line_for_error` and `Stmt::line_for_error` return that best-effort line (`src/ast.rs:253-291`).
- `Cond` line access is local to interpreter/VM helpers (`src/interpreter/eval.rs:152-154`, `src/vm.rs:1193-1194`).
- Tokens have full six-field spans (`docs/TOKEN_CONTRACT.md`), but the parser does not copy them into AST nodes.
- `BoliError` can carry `kind` and boxed `span`, but current parser errors use `BoliError::line_col`, leaving both fields `None`.

Future BOLI AST requirement:

- Every node should carry `line`, `col`, and a six-field `span` copied from the first token that started the construct.
- `line` and `col` must remain aliases of `span.start_line` and `span.start_col` for compatibility with current diagnostics.
- Aggregate nodes (`Arith`, `Call`, `List`, `Index`, comparisons, blocks, statements) should use a span from start token through the last consumed token of that construct when available.
- If a construct ends at `Eof`, the EOF span is used; this may be zero-length.
- Span computation must not alter token positions or consume extra tokens.

Compatibility rule: a future golden AST dump should support two comparison modes:

1. `strict`: compare all fields including spans.
2. `legacy`: compare only node shape plus `line`, matching the current Rust AST.

Legacy mode is the baseline for Rust-to-BOLI differential tests until Rust AST gains spans. Span work is outside Phase 4A.

## Tagged-map representation for BOLI

BOLI has no enums, sum types, pattern matching, or spatial cost-free closures. The intended future representation is a tagged map.

Common fields:

```text
"tag"   -> text node tag, exact variant name
"line"  -> number, mirror of span.start_line
"col"   -> number, mirror of span.start_col
"span"  -> map with start/end byte and line/col fields
```

Variant-specific field names should match Rust field names where practical:

```text
Expr::Call      => tag "Call",      name: text, args: list of Expr maps
Expr::Arith     => tag "Arith",     op: text, left: Expr map, right: Expr map
Expr::List      => tag "List",      items: list of Expr maps
Expr::Index     => tag "Index",     target: Expr map, index: Expr map
Stmt::Rakho     => tag "Rakho",     name: text, value: Expr map
Stmt::Agar      => tag "Agar",      cond: Cond map, then_block: list, else_block: list or khaali
Stmt::MapRakho  => tag "MapRakho",  name: text, pairs: list of two-item lists
Cond::Cmp       => tag "Cmp",       left: Expr map, op: text, right: Expr map
Cond::Aur       => tag "Aur",       left: Cond map, right: Cond map
Cond::Ya        => tag "Ya",        left: Cond map, right: Cond map
```

Operators should be stored with the `naam()` strings from `src/ast.rs`, not symbolic `+ - * / > <` strings, because BOLI source uses Hinglish words.

Nested AST values must respect `BOLI_MAX_VALUE_DEPTH` (default 64). Deeply nested expressions may therefore hit a runtime value-depth limit before or during AST construction; a future parser must report this as a parse-state limit rather than crash.

## Runtime compatibility matrix

Both engines exhaustively match the current AST variants:

- Interpreter statements: `src/interpreter/mod.rs:302-541`.
- Interpreter expressions and conditions: `src/interpreter/eval.rs:29-154`.
- VM expressions and conditions: `src/vm.rs:121-210`.
- VM statements: `src/vm.rs:249-451`.

| AST family | Interpreter | VM | Phase 4A action |
|---|---|---|---|
| Literal expressions | supported | supported | preserve shape |
| `Var` | supported | supported | preserve lookup semantics |
| `Arith` | supported | supported | preserve operand order |
| `Call` | supported | supported | preserve flat arguments |
| `List` | supported | supported | preserve item order |
| `Index` | supported | supported | preserve 1-based runtime indexing |
| `Cmp` / `Aur` / `Ya` | supported | supported | preserve short-circuit behavior |
| All 15 `Stmt` variants | supported | supported | preserve shapes and field names |

Adding or changing a node in a later phase requires updating both `eval.rs`/`interpreter/mod.rs` and `vm.rs`. Phase 4A adds none.

## Semantic invariants for a future parser

1. `Program` order is source order; newlines carry no AST node.
2. Blocks are plain lists of statements; no block scope node exists.
3. `RangeLoop` does not store a loop variable name; runtime uses `ginti`.
4. `Wapas.value == khaali` means `wapas do` with no expression, not `wapas do khaali`.
5. Multi-atom `rakho` becomes `Expr::List`; single-atom `rakho` stays that atom.
6. Call arguments are flat atoms; nested calls are not valid arguments under the current grammar.
7. `kaam` and `hissa` are contextual names; they must become `Var`/`String` names, not keyword AST nodes.
8. Map pairs are ordered as written; BOLI maps use text keys at runtime but pair keys here are expressions.
9. Parse errors are values/maps in a future BOLI parser, not exceptions.
10. Parser state must be threaded explicitly because BOLI assignment writes only the innermost scope.

## Non-goals

- No enums or algebraic data types added to BOLI.
- No AST execution semantics changed.
- No span field added to Rust AST.
- No new AST node.
- No parser implementation in Phase 4A.
