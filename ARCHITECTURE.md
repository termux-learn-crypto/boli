# BOLI Architecture

**Components:** `lexer → parser → AST → tree-walking interpreter`. Everything is plain Rust with no third-party dependencies.

```
source.hl ──► lexer.rs ──► parser.rs ──► ast.rs (Program) ──► interpreter.rs (Environment) ──► stdout
source text    Tokens        Recursive-descent  Statements    scopes + kaam table     UTF-8 bytes
               + line nos    parse, Hinglish    + line nos        Vec<HashMap>         print only
               errors        errors                                      ^
                                                                         |
                                                                  main.rs (CLI + REPL)
```

Modules (each with a brief responsibility):

| Module | Responsibility | Public API |
|--------|----------------|------------|
| `src/ast.rs` | `Program`/`Stmt`/`Expr`/`Cond` types, `BoliError` (line-numbered Hinglish errors) | types |
| `src/lexer.rs` | UTF-8 tokenizer: keywords, identifiers (Unicode incl. Devanagari), numbers (finite-only), text with escapes, `#` comments, `-` negative numbers | `lex(&str) -> Result<Vec<Token>, BoliError>` |
| `src/parser.rs` | Recursive-descent parser; statement split by newlines; two-level condition precedence (`aur` > `ya`); `hissa` contextual index; flat call arguments | `parse_source(&str) -> Result<Program, BoliError>` |
| `src/interpreter.rs` | Tree-walking evaluator: scopes stack + kaam table + env-based safety limits; output capture hook (`collect_output`/`take_output`) used by tests | `Environment`, `Value`, `BoliError` |
| `src/main.rs` | CLI (`run`, `help`, `version`) and the interactive REPL with quote/comment-aware depth tracking | binary `boli` |
| `src/limits.rs` | Single source of truth for every safety limit (defaults + `BOLI_MAX_*` env override) | `limits` module |
| `src/lib.rs` | Exposes the modules above as the library `boli` | used by `tests/boli.rs` |

Data flow:

- **Parse**: text → Vec<Token> → Program (no tokenstream retained).
- **Interpret**: Environment::new() creates the global scope; `run_program` walks statements; function calls push/pop scope; loops iterate with a steps guard.
- **Output**: in production, `likho` writes to stdout (binary); tests flip `Environment::captured = Some(Vec<String>)` to capture lines instead.
- **Errors**: every layer returns `BoliError { line, message }`; `Display` prints `Galti: line N me <Hinglish message>`.

Key invariants:

- Scopes are a stack of `HashMap<String, Value>`; every `push_scope` has an unconditional `pop_scope` (success or error).
- Every recursion axis is bounded by an env-tunable limit: parser nesting (64), condition `aur`/`ya` chain (128 parse + 128 eval), expression evaluation depth (256), kaam call depth (64), program steps (100,000), list items (10,000), string bytes (1 MiB), program tokens (1,000,000), REPL line bytes (64 KiB), live scope memory budget (64 MiB). See `src/limits.rs`. The lexer walks a two-char `Cursor` (no `Vec<char>`), and source reads are bounded via `take(max+1)`.
- Functions are hoisted into a separate `kaam` table at call time (not closure capture); name collisions are rejected.
- Numbers are validated finite at three boundaries: literals, `pucho` parsing, and arithmetic results.
- No panic paths in user code: all runtime errors are `BoliError`s; the only `exit`s are `std::process::exit(0)` on closed stdout.

How the REPL works:

- It buffers lines; a depth counter (`agar`/`jab`/`kaam <name>`/`tak` +1, `khatam` −1) decides when the buffer is a complete program.
- Lines inside `"..."` and after `#` are ignored for depth calculation; a negative depth resets with a Hinglish message (`'khatam' bina kuch shuru kiye aaya`).
- Environment persists across REPL inputs (variables and kaam stay); `nikalo`/`exit`/`quit` exits cleanly.

## Why the interpreter is still one file

`Environment` owns the scope stack, the kaam table and the limit counters; `eval_expr`, `eval_cond`, `call_kaam`, `run_stmt` and `run_block` all mutate the same state and call each other mutually. Splitting `interpreter.rs` into `environment.rs` / `evaluator.rs` / `functions.rs` would only move that shared state across module boundaries without changing ownership — more ceremony, same coupling, and a needless diff risk to a working evaluator. The file stays single; `limits.rs` was extracted instead because lexer/parser/interpreter/CLI all need it. Revisit the split if the evaluator grows substantially (e.g. a bytecode VM, which is tracked in docs/UPGRADE_ROADMAP.md).
