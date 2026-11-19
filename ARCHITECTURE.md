# BOLI Architecture

**Components:** `lexer → parser → AST → tree-walking interpreter (default) | bytecode VM (experimental)`. Everything is plain Rust with no third-party dependencies.

```
source.hl ──► lexer.rs ──► parser.rs ──► ast.rs (Program) ─┬─► interpreter/ (Environment, default) ─► stdout
source text    Tokens        Recursive-descent  Statements  └─► vm.rs (bytecode, experimental) ────► stdout
               + line nos    parse, Hinglish    + line nos        scopes + kaam table / stack machine
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
| `src/interpreter/` | Tree-walking evaluator (`mod.rs`/`eval.rs`/`value.rs`): scopes stack + kaam table + env-based safety limits; output capture hook (`collect_output`/`take_output`) used by tests | `Environment`, `Value`, `BoliError` |
| `src/main.rs` | CLI (`run`, `help`, `version`, `--vm`, `BOLI_VM=1`) and the interactive REPL with quote/comment-aware depth tracking | binary `boli` |
| `src/vm.rs` | Bytecode VM (experimental): compiler (AST → `Op` chunks) + stack machine; shares `Value`, `eval_arith`, builtins and the budget helpers. `laao` is compiled inline; steps, scopes and stack reset per program and unwind on error | `boli::vm::Vm`, `Vm::stack_depth()` |
| `src/scope.rs` | Live-scope `Scope` + `value_size` accounting shared by interpreter and VM | `scope` module |
| `src/limits.rs` | Single source of truth for every safety limit (defaults + `BOLI_MAX_*` override), bounded file/stdin reads and `BOLI_IMPORT_ROOT` path resolution | `limits` module |
| `src/builtins.rs` | Built-in kaam (`lambayi`/`upar`/`neeche`/`text`/`sankhya`/`chaabiyan`) — pure Rust, user kaams shadow them | `builtins` module |
| `src/lib.rs` | Exposes the modules above as the library `boli` | used by `tests/boli.rs` and `tests/vm.rs` |

Data flow:

- **Parse**: text → Vec<Token> → Program (no tokenstream retained).
- **Interpret**: Environment::new() creates the global scope; `run_program` walks statements; function calls push/pop scope; loops iterate with a steps guard.
- **VM path**: AST → `Op` chunks → stack machine; `run_program` resets steps/scopes/stack and restores them on error, making a reused `Vm` clean. Both backends are parity-tested on a shared corpus.
- **Output**: in production, `likho` writes to stdout (binary); tests flip `Environment::captured = Some(Vec<String>)` to capture lines instead.
- **Errors**: every layer returns `BoliError { line, message }`; `Display` prints `Galti: line N me <Hinglish message>`.

Key invariants:

- Scopes are a stack of `HashMap<String, Value>`; every `push_scope` has an unconditional `pop_scope` (success or error).
- Every recursion axis is bounded by an env-tunable limit: parser nesting (64), condition `aur`/`ya` chain (128 parse + 128 eval), expression evaluation depth (256), kaam call depth (64), program steps (100,000), list items (10,000), string bytes (1 MiB), program tokens (1,000,000), REPL line bytes (64 KiB), live scope memory budget (64 MiB), import depth (32). See `src/limits.rs`. The lexer walks a two-char `Cursor` (no `Vec<char>`), source reads are bounded via `take(max+1)`, over-limit input lines are drained through the next newline, and optional `BOLI_IMPORT_ROOT` confines `laao` to a canonicalized directory in both engines.
- Functions are hoisted into a separate `kaam` table at call time (not closure capture); name collisions are rejected.
- Numbers are validated finite at three boundaries: literals, `pucho` parsing, and arithmetic results.
- No panic paths in user code: all runtime errors are `BoliError`s; the only `exit`s are `std::process::exit(0)` on closed stdout.

How the REPL works:

- It buffers lines; a depth counter (`agar`/`jab`/`kaam <name>`/`tak` +1, `khatam` −1) decides when the buffer is a complete program.
- Lines inside `"..."` and after `#` are ignored for depth calculation; a negative depth resets with a Hinglish message (`'khatam' bina kuch shuru kiye aaya`).
- Environment persists across REPL inputs (variables and kaam stay); `nikalo`/`exit`/`quit` exits cleanly.

## Interpreter layout

`src/interpreter/` is a module:
- `mod.rs` — `Environment`: scope stack, kaam table, limit counters, `run_program`/`run_block`/`run_stmt`, `call_kaam`, scope/limit wiring, public builders. Owns the trace stack for tracebacks.
- `eval.rs` — `eval_expr`/`eval_cond` (and `eval_expr_inner`/`eval_cond_inner`) as `impl Environment` methods, plus `eval_arith`, `compare`, `values_equal`, `as_number`, `checked_text`.
- `value.rs` — `Value` enum, `dikhane_laayak`/`type_naam`, and `value_size` (live-bytes budget accounting).

Split was done once the evaluator had 942 lines and a second concern (live-bytes accounting + trace stack) landed in the same type. The shared-state problem (eval needs `&mut Environment`) is handled by `impl Environment` in `eval.rs`, not by passing state around.
