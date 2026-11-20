# BOLI Parser Grammar (Observed Rust Reference)

## Status

This document describes the grammar actually implemented by `src/parser.rs` at Phase 4A. It is a design source only. No parser implementation, AST implementation, lexer change, or new syntax was added in Phase 4A.

Authority order:

1. `src/parser.rs` — implemented grammar.
2. `src/ast.rs` — produced node set.
3. `src/lexer.rs` — token inventory.
4. `docs/TOKEN_CONTRACT.md` — token/span contract.
5. `LANGUAGE_SPEC.md` — user-facing description; where it disagrees with `src/parser.rs`, this document follows the code.

## Lexical prerequisites

The parser consumes the token list defined in `docs/TOKEN_CONTRACT.md`. Relevant facts:

- Statements are separated by `Newline`; a trailing `Eof` is required by the lexer contract.
- Comments are removed by the lexer; their newline remains.
- Token line/column are one-based character positions; spans are zero-based UTF-8 byte offsets with end-exclusive `end`.
- There are no symbol statements: no `; { } ( ) = : .`; `@` is rejected by the lexer.
- Keywords are case-insensitive; identifiers are case-sensitive.
- Only `Number`, `Text`, and `Ident` carry payload text. Unit tokens carry empty text.

Token inventory used by the parser (`src/lexer.rs:8-52`):

```text
Likho Rakho Pucho Jodo Ghatao Guna Bhaag
Agar Nahi To Khatam Se Aur Ya Ko Tak Jab
Bada Chhota Barabar Hai Kaam Wapas Do
Sahi Galat Khaali Hissa Map Laao Ruk Agla
Number(f64) Text(String) Ident(String) Newline Eof
```

## Notation

```text
X?             zero or one X
X*             zero or more X
X+             one or more X
A | B          alternative
"word"         exact keyword token
<field>        field value placeholder
NEWLINE        Tok::Newline
EOF            Tok::Eof
LINE_END       NEWLINE+ | EOF
```

`LINE_END` is implemented by `expect_line_end` (`src/parser.rs:92-112`). It consumes all consecutive newlines and accepts EOF without consuming. Any other next token is an immediate parse error.

## Program and blocks

```text
Program      ::= NEWLINE* (Stmt NEWLINE*)* EOF
Block        ::= NEWLINE* (Stmt NEWLINE*)*
                 until lookahead is "khatam" | "nahi" | EOF
```

`parse_program` is at `src/parser.rs:165-174`. `parse_block` is at `src/parser.rs:176-187`. A block stops on `Khatam`, `Nahi`, or `Eof` **without consuming** the terminator; the caller consumes what it expects.

## Atom start set

`atom_start` (`src/parser.rs:189-205`) decides whether the next token can begin a value:

```text
AtomStart ::= Number | Text | "sahi" | "galat" | "khaali"
            | Ident | "kaam" | "hissa"
            | "jodo" | "ghatao" | "guna" | "bhaag"
```

`Map`, `Laao`, `Ruk`, `Agla`, `Agar`, `Jab`, `Wapas`, and punctuation-like control tokens are not atom starts.

## Expressions

```text
Expr        ::= Arith | Atom
Arith       ::= "jodo"   Atom "aur" Atom
              | "ghatao" Atom "se" Atom
              | "guna"   Atom "ko" Atom "se"
              | "bhaag"  Atom "ko" Atom "se"
Atom        ::= AtomBase HissaPostfix*
HissaPostfix::= "hissa" AtomBase
AtomBase    ::= Number
              | Text
              | "sahi"
              | "galat"
              | "khaali"
              | Name
              | Arith
CallOpt     ::= Name Atom+        -- only when allow_call is true
Name        ::= Ident | "kaam" | "hissa"
Operand     ::= Atom              -- allow_call = false
RakhoValue  ::= Atom+             -- one atom -> that atom; >1 -> List(atom+)
```

Implementation anchors:

- `parse_expr`: `src/parser.rs:554-565`.
- `parse_atom` / `atom`: `src/parser.rs:567-609`.
- `atom_base`: `src/parser.rs:611-686`.
- `parse_operand`: `src/parser.rs:572-576`.
- Arithmetic tails: `src/parser.rs:688-709`.
- `parse_rakho_value`: `src/parser.rs:139-163`.

Important behavior:

- Arithmetic is **prefix Hinglish**, not infix symbols. `"jodo 5 aur jodo 3 aur 4"` nests because `Atom` can itself be `Arith`.
- There is no traditional precedence table. Nesting is explicit by writing one arithmetic keyword inside another. `LANGUAGE_SPEC.md` calls this "expression nesting"; it is the only grouping mechanism.
- Postfix `"hissa"` is left-associative: `a hissa 1 hissa 2` parses as `(a[1])[2]`.
- Index expressions use `AtomBase`, not full `Atom`; an index cannot itself carry another postfix `hissa`.
- `Name` followed by atom-start tokens becomes `Call(name, flat_args)` only when `allow_call` is true. Call arguments are flat: `f a b` is `f(a, b)`, not `f(a(b))`.
- Call arguments cannot start another call because `atom(false)` is used for each argument.
- Arithmetic operands use full `Atom`, so a call may appear inside arithmetic, e.g. `likho jodo f 1 aur 2`.
- `parse_operand` uses `atom(false)`, so comparison operands do **not** become calls even if followed by another atom.

## Conditions

```text
Cond         ::= YaChain
YaChain      ::= AurChain ("ya" AurChain)*
AurChain     ::= Comparison ("aur" Comparison)*
Comparison   ::= Operand Operand "se"? CmpOp "hai"
CmpOp        ::= "bada"
               | "bada" "ya" "barabar"
               | "chhota"
               | "chhota" "ya" "barabar"
               | "barabar"
```

Implementation anchors:

- `parse_cond`: `src/parser.rs:711-727`.
- `parse_aur`: `src/parser.rs:750-764`.
- `parse_comparison`: `src/parser.rs:766-825`.
- Depth counting: `src/parser.rs:729-748`.

Important behavior:

- `ya` has lower precedence than `aur`: `a ya b aur c` parses as `a ya (b aur c)`.
- A comparison always reads exactly two operands before the comparator: `umar 18 se bada hai`.
- The `se` before the comparator is optional: `umar 18 bada hai` is accepted.
- `barabar ya barabar` is not a `>=`-style operator; it stops at `Barabar`.
- The condition chain counter starts at `1` and increments once per `aur`/`ya`; the default cap is `128` via `BOLI_MAX_CONDITION_DEPTH` (`src/limits.rs:10`).

## Statements

### `likho`

```text
"likho" Expr LINE_END
```

Produces `Stmt::Likho(expr, line)` (`src/parser.rs:228-235`).

### `pucho`

```text
"pucho" Name Text? LINE_END
```

Produces `Stmt::Pucho { name, prompt, line }`. Missing prompt text defaults to `"Pucho: "` (`src/parser.rs:236-249`). `Name` includes contextual `kaam`/`hissa`.

### Arithmetic assignment

```text
("jodo" | "ghatao" | "guna" | "bhaag") ArithTarget ArithTail LINE_END
ArithTarget ::= Ident | "hissa"
ArithTail   ::= same form as the arithmetic tail in `Expr`
```

Produces `Stmt::ArithAssign { target, op, left, right, line }` (`src/parser.rs:250-285`).

Note: the statement-form target accepts `Ident` or `hissa`, but not the contextual `kaam` token. A variable literally named `kaam` cannot currently be the target of this statement form.

### `agar`

```text
"agar" Cond "to" Block
  ("nahi" "to" Block)?
"khatam" LINE_END
```

Produces `Stmt::Agar { cond, then_block, else_block, line }` (`src/parser.rs:286-308`). `nahi` is recognized as the `else` marker only here.

### `jab tak`

```text
"jab" "tak" Cond "to" Block "khatam" LINE_END
```

Produces `Stmt::WhileLoop { cond, body, line }` (`src/parser.rs:309-320`).

### Range loop

```text
Number "se" Atom "tak" Block "khatam" LINE_END
```

Produces `Stmt::RangeLoop { from, to, body, line }` (`src/parser.rs:321-337`).

The parser starts this branch only when the first token is `Number`. The runtime injects the loop counter name `ginti`; the AST does not store a loop-variable name.

### `kaam` forms

`Tok::Kaam` is contextual (`src/parser.rs:338-406`):

```text
"kaam" "hissa" Operand "rakho" Atom LINE_END
    -> ListAssign { target: "kaam", ... }

"kaam" "rakho" RakhoValue LINE_END
    -> Rakho { name: "kaam", ... }

"kaam" (Number | Text | "sahi" | "galat" | "khaali") Atom*
    -> KaamCall { name: "kaam", ... }

"kaam" Name Name* Block "khatam" LINE_END
    -> KaamDef { name, params, body, ... }
```

The "call" test checks `peek_at(1)` for a literal token. A normal call such as `namaste "Rahul"` is parsed by the `Ident` branch below, not by the `kaam` branch.

Function parameters accept `Ident`, `kaam`, or `hissa` (`src/parser.rs:392-396`).

### `wapas do`

```text
"wapas" "do" Expr? LINE_END
```

Produces `Stmt::Wapas { value, line }`. If the next token is `Newline` or `Eof`, `value` is `None` (`src/parser.rs:407-418`).

### Name-led statements

For `Ident` or `hissa` as the first token (`src/parser.rs:419-498`):

```text
Name "rakho" "map" Pair+ LINE_END
    -> MapRakho { name, pairs, line }
Pair ::= AtomBase AtomBase

Name "rakho" RakhoValue LINE_END
    -> Rakho { name, value, line }

Name "hissa" Operand "rakho" Atom LINE_END
    -> ListAssign { target: name, index, value, line }

Name Atom* LINE_END
    -> KaamCall { name, args: Atom*, line }
```

Map rules (`src/parser.rs:431-469`):

- Map key and value are `atom(false)`.
- A nested `map` token inside the pair list is rejected.
- Empty map is rejected.
- Missing value for a key is rejected.

List assignment accepts exactly one atom as the new value; it does not accept multi-atom list values (`src/parser.rs:474-489`).

### `laao`, `ruk`, `agla`

```text
"laao" Text LINE_END
"ruk" LINE_END
"agla" LINE_END
```

Produce `Stmt::Laao`, `Stmt::Ruk`, and `Stmt::Agla` (`src/parser.rs:499-532`). `Laao` requires a quoted text path.

### Invalid standalone terminators

- Standalone `khatam` → error: used without opening `agar`/`jab tak`/`kaam` (`src/parser.rs:533-537`).
- Standalone `nahi` → error: `nahi to` only inside `agar` (`src/parser.rs:538-542`).

## Contextual keywords

- `kaam` can be a function-definition keyword, a variable name, a call target name, or part of list assignment.
- `hissa` can be a postfix index operator, a variable name, a function parameter name, or a list-assignment marker.
- The disambiguation is local and uses one- or two-token lookahead; it is not a general operator-precedence system.

## Limits that affect parsing

From `src/parser.rs` and `src/limits.rs:4-17`:

| Limit | Default | Env var | Where enforced |
|---|---:|---|---|
| Statement/expression nesting | 64 | `BOLI_MAX_PARSE_DEPTH` | `parse_stmt`, `atom`, `atom_base` |
| `aur`/`ya` condition chain | 128 | `BOLI_MAX_CONDITION_DEPTH` | `count_cond_node` |
| Program tokens | 1,000,000 | `BOLI_MAX_TOKENS` | lexer |
| Source bytes | 10 MiB | `BOLI_MAX_SOURCE_BYTES` | file/CLI input path |
| String bytes | 1 MiB | `BOLI_MAX_STRING_BYTES` | lexer/runtime |

## Grammar-versus-document notes

- `LANGUAGE_SPEC.md` lists keyword aliases `break` and `continue`; `src/lexer.rs:8-52` has only `Tok::Ruk` and `Tok::Agla`, with alias handling outside the `Tok` enum where implemented. A future parser must consume the token stream it is given, not re-implement lexer aliases.
- `LANGUAGE_SPEC.md` describes precedence; the implemented grammar uses explicit nested prefix arithmetic and left-associative postfix `hissa`.
- `docs/PARSER_INTERFACE_SPEC.md` remains the token-boundary spec. This document is the grammar spec. Neither document changes behavior.

## Non-goals

- No new syntax.
- No punctuation grouping.
- No unsupported infix operators.
- No parser implementation in Phase 4A.
- No change to Rust `Expr`, `Stmt`, or `Cond`.
