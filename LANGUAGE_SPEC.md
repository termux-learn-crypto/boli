# BOLI Language Specification (v0.3.1)

**Version:** 0.3.1 · **File extension:** `.hl` · **Implementation:** Rust (`cargo run -- run x.hl`, REPL via `cargo run`)

BOLI is a Hinglish-based programming language: Python-like simplicity, Hindi/English keywords, no symbols like `; { } ( ) = :`. Statements are one per line. Only words, numbers, quoted text and whitespace appear in a program.

---

## 1. Lexical rules

| Item | Rule |
|------|------|
| File encoding | UTF-8 |
| Statement terminator | newline (empty lines are ignored) |
| Comments | `#` to end of line |
| Text literal | `"..."` — escapes `\"`, `\\`, `\n`; embedded newline not allowed |
| Number literal | digits with optional single `.` and decimal part, optional leading `-` (only directly before a digit); must be **finite** — a literal exceeding f64 range is a Galti error |
| Identifier | Unicode letters/combined marks/digits/`_` (Devanagari and Latin both work), start with letter or `_` |
| Keywords | `likho pucho rakho jodo ghatao guna bhaag agar nahi to khatam se aur ya ko tak jab bada chhota barabar hai kaam wapas do sahi galat khaali hissa ruk agla break continue` |

Keywords are matched **case-insensitively** (the word is lowercased before matching, so `LIKHO`, `Likho` and `likho` are all the keyword `likho`). Identifiers are matched **case-sensitively** as written (`Naam` and `naam` are different names). Text literals are never touched by case handling.

## 2. Types

| Type | Notation | Notes |
|------|----------|-------|
| text | `"Vilas"` | Unicode string, max 1 MiB (`BOLI_MAX_STRING_BYTES`) |
| number | `28`, `99.50`, `-5` | f64 internally; must be finite; int-like values print without decimal; integers above 2^53 are not exact |
| sahi/galat | `sahi`, `galat` | bool |
| khaali | `khaali` | null |
| list | `cheezein rakho 1 2 3` | list of values, mixed types allowed; index with `hissa` (1-based); max 10,000 items (`BOLI_MAX_LIST_ITEMS`) |

No implicit NaN or ±∞ anywhere: input parsing, literals and arithmetic results are all validated finite.

## 3. Statements

```boli
# likho <expr>                  print
likho "text"
likho ginti
likho jodo 5 aur 3              # inline hisaab

# <naam> rakho <values...>     variable (multi-value = list)
naam rakho "Vilas"
cheezein rakho 1 2 3            # list [1, 2, 3]

# <naam> rakho jodo ...        hisaab rakho (rakho NAHI kaam karte hue)
jodo jawab 5 aur 3               # jawab = 8
ghatao bacha 10 se 4             # bacha = 6
guna total 5 ko 2 se             # total = 10
bhaag hissa 10 ko 2 se           # hissa = 5

# list index assignment (1 se ginti)
cheezein hissa 2 rakho 99

# pucho <naam> "sawal"          stdin line read (type auto-detect)
pucho naam "Tumhara naam"

# agar ... nahi to ... khatam    condition
# jab tak <cond> to ... khatam   while loop
# <from> se <to> tak ... khatam  range loop (inclusive, ginti naam ka variable)
# kaam <naam> <params> ... khatam  function
# wapas do <expr>                 return value
# ruk / agla                        break / continue (alias: break / continue)
```

## 4. Operators & precedence

Highest → lowest:

1. **Expression nesting** — nested `jodo/ghatao/guna/bhaag` groups inner-first (like parentheses in other languages)
2. **List indexing** — `hissa` postfix
3. **Comparisons** — `bada`, `chhota`, `barabar`, `bada ya barabar`, `chhota ya barabar`
4. **`aur`** (AND)
5. **`ya`** (OR)

There are **no symbol parentheses**; nesting works by writing nested expressions inside each other:

```boli
agar jodo 1 aur 2 barabar jodo 2 aur 1 hai to   # 3 == 3
    likho "same"
khatam
```

Logic reads left-to-right; `aur` binds tighter:

```boli
agar a sahi barabar hai ya b sahi barabar hai aur c sahi barabar hai to
```
is parsed as `a ya (b aur c)`.

## 5. Conditions & comparisons

Form: `<operand> <operand> [se] <op> [ya barabar] hai`

The `aur` / `ya` chain in one condition is capped at 128 (`BOLI_MAX_CONDITION_DEPTH`); longer chains are a Galti error instead of a stack overflow.

(`se` is optional; `hai` keyword marks the end of the condition.)

| Operator | Meaning |
|----------|---------|
| `bada` | > |
| `chhota` | < |
| `barabar` | == |
| `bada ya barabar` | >= |
| `chhota ya barabar` | <= |

Operands can be number, text (`bada`/`chhota` compare unicode) or list only with `barabar`. Mixed text/number comparison is a Galti error (except `barabar` → `galat`).

## 6. Functions (kaam)

```boli
kaam namaste naam
    likho "Namaste"
    likho naam
khatam

namaste "Rahul"
```

- Calls: `kaam_name` or `kaam_name arg1 arg2 ...` — arguments are evaluated in the caller's scope.
- Arguments are **flat atoms**: `f a b` means `f(a, b)` — never a nested call. For nested calls bind to a zero-argument kaam or to a variable: `t rakho g 1 2` then `f t`.
- Zero-argument kaam can be named in expressions: `likho jawab_do` → calls `jawab_do()`. A variable of that name shadows the kaam.
- `wapas do expr` returns a value; `wapas do` alone returns `khaali`.
- Recursion depth limited (`BOLI_MAX_CALL_DEPTH`, default 64).
- `ruk`/`agla` inside kaam without an enclosing loop are errors.

## 7. Scoping

- `rakho` always writes to the **current (innermost) scope**; it never mutates an outer/global variable silently (v0.2.0).
- Reading a name searches innermost → outermost → global.
- Loops push a temporary scope (`ginti` for range loops); popped at loop end.
- Kaam calls push a scope with the parameters; popped at kaam exit (even on errors).

## 8. Safety limits (configurable via environment)

| Limit | Default | Env var |
|-------|---------|---------|
| Steps (whole program) | 100,000 | `BOLI_MAX_STEPS` |
| Kaam call depth | 64 | `BOLI_MAX_CALL_DEPTH` |
| String bytes (literals, `pucho` input, concat result, formatted text) | 1 MiB | `BOLI_MAX_STRING_BYTES` |
| Source file bytes (`boli run` file read) | 10 MiB | `BOLI_MAX_SOURCE_BYTES` |
| List items (creation and `jodo` concat result) | 10,000 | `BOLI_MAX_LIST_ITEMS` |
| Parse nesting depth (statements/expressions) | 64 | `BOLI_MAX_PARSE_DEPTH` |
| Condition chain length (`aur` / `ya`, parse + eval) | 128 | `BOLI_MAX_CONDITION_DEPTH` |
| Expression evaluation depth | 256 | `BOLI_MAX_EVAL_DEPTH` |
| Program tokens | 1,000,000 | `BOLI_MAX_TOKENS` |
| REPL input line bytes | 64 KiB | `BOLI_MAX_REPL_LINE_BYTES` |
| Live scope bytes (memory budget) | 64 MiB | `BOLI_MAX_TOTAL_BYTES` |

All violations are Hinglish Galti errors, never panics/SIGSEGV/SIGKILL. Violations are deterministic. `std::process::exit(0)` on closed stdout pipe.

Input from `pucho` is read in bounded chunks (at most `BOLI_MAX_STRING_BYTES` + one buffer) — an unbounded-length stdin line cannot be buffered. Invalid UTF-8 from stdin is decoded lossy (U+FFFD) instead of crashing.

## 9. Errors

All runtime and syntax errors are Hinglish with line numbers:

```
Galti: line 4 me naam 'xyz' nahi mila
Galti: line 7 me zero se bhaag nahi kar sakte
Galti: line 1 me ';' nahi chahiye, BOLI me sirf shabd, number aur "text" use hota hai
```

## 10. Contextual keywords

- `kaam` — function keyword; also a valid variable name (`pucho kaam`, `kaam rakho` …).
- `hissa` — list-index keyword; also a valid variable name (`bhaag hissa 10 ko 2 se`).

In both cases a variable of that keyword name may be created, read and assigned anywhere an identifier is expected. `ruk`/`agla`/`break`/`continue` are the only reserved words that don't double as identifiers.

## 11. Record of behavior

### v0.3.1 vs v0.3.0

- `BOLI_MAX_EVAL_DEPTH` ab sach me configurable hai (pehle hardcoded 256 tha, env var ignore hota tha); error message bhi configured value dikhata hai.
- Lexer `Vec<char>` nahi banata (sliding cursor); source/literal/identifier/token byte caps (`BOLI_MAX_SOURCE_BYTES` / `BOLI_MAX_STRING_BYTES` / `BOLI_MAX_TOKENS`) — badi lines/files bhi safe tokenize hoti hain; Devanagari unchanged.
- `boli run` ab bounded read karta hai: `BOLI_MAX_SOURCE_BYTES` se ek byte zyada padhne pe Galti (file read ke dauraan grow ho to bhi safe).
- REPL line input `BOLI_MAX_REPL_LINE_BYTES` (64 KiB) se bounded; oversized line reject hoti hai, REPL zinda rehta hai; multiline blocks, quoted text, comments aur `nikalo` unchanged.
- Live scope memory budget `BOLI_MAX_TOTAL_BYTES` (64 MiB default): variables/list/kaam-scope bytes counted; replace-safe accounting; budget error Galti hai.
- Crash ya nahi: sab caps deterministic Galti error dete hain.

### v0.3.0 vs v0.2.0

- Initial string literals are now size-checked against `BOLI_MAX_STRING_BYTES` (concatenation was already checked).
- Lists are capped at 10,000 items (creation and `jodo` concatenation); overflow is a Galti error.
- `boli run file.hl` refuses files larger than `BOLI_MAX_SOURCE_BYTES` (10 MiB) before reading.
- Condition chains (`aur`/`ya`) are capped at 128 terms at both parse time and evaluation time.
- Expression evaluation is depth-capped at 256 (covers deep `hissa` chains).
- Statement/expression nesting during parsing is capped at 64 (`BOLI_MAX_PARSE_DEPTH`).
- `pucho` input is read with a hard cap; invalid UTF-8 input is decoded lossy.
- Keywords are documented as case-insensitive; identifiers case-sensitive.

### v0.2.0 vs v0.1.0

- `rakho` semantics: local write-only (was nonlocal-update).
- `f a b` flat argument parsing (was nested call nesting).
- `aur`/`ya` precedence: `aur` binds tighter (was equal/left-assoc).
- `pucho`/literals/arithmetic validate finite numbers (was inf/nan allowed).
- See CHANGELOG.md.
