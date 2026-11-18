# Testing BOLI

## How to run the tests

```bash
cargo test          # everything (unit + integration)
cargo test --test boli      # library-level semantics (54 tests)
cargo test --test cli       # CLI/binary end-to-end (13 tests)
```

No extra tooling is required — only `cargo` and the BOLI binary (which `cargo test` builds automatically). The CLI tests use the `CARGO_BIN_EXE_boli` path cargo provides; the library tests use the `Environment`/`parse_source` API directly and capture `likho` output via `Environment::collect_output`.

## What is tested

### Lexer (`tests/boli.rs`)
- token stream for a simple statement
- negative number tokenizing
- string escapes (`\n`, `\"`)
- symbol rejection with Hinglish error
- Unicode identifiers

### Parser (`tests/boli.rs`)
- single-value vs multi-value `rakho` → AST (`Rakho`, `Expr::List`)
- `ruk`/`agla` → `Stmt::Ruk` / `Stmt::Agla`
- `hissa` postfix index AST (`Expr::Index`)
- `ListAssign` statement syntax
- flat call arguments (`f a b` → `(a, b)`, not `a(b)`)
- `aur`/`ya` precedence at AST level (`Ya(a, Aur(b, c))`)

### Interpreter semantics (`tests/boli.rs`)
- inline math (`likho jodo 5 aur 3` → `8`)
- assignments, types, conditions (then/else/nesting/`barabar` strings)
- range loops (basic/negative/empty/2^52-boundary/huge terminates via steps)
- while loops, `ruk`, `agla`, break-outside-loop error
- loop scope cleanup on success **and** on runtime errors; REPL state consistency
- kaam: zero/one/multiple/nested/expression args, arity errors, recursion limit
- global variable not overwritten from inside a kaam; kaam reads global fine
- lists: create/print/index/assign/concat/out-of-bounds error
- steps limit stops infinite loops with Hinglish error
- **BUG 5 regressions:** literal overflow rejected, large finite literals accepted, decimal literals work, arithmetic overflow errors instead of producing ∞, finite results unchanged

### CLI (`tests/cli.rs`, spawned binary)
- `boli run examples/loop.hl` and `examples/calculator2.hl` work end-to-end
- missing file → Hinglish `Galti:` to stderr, exit 1
- REPL: stray `khatam` doesn't latch the depth counter; multi-line blocks still work
- broken pipe (stdout closes early) exits 0 without panic
- **BUG 5 input regressions:** `nan`/`inf`/`Infinity`/`-Infinity` via `pucho` enter as *text*, not numbers; `99.50` parses as a number; a full-range f64 literal stays a number; arithmetic overflow exits non-zero with the Hinglish message

## Environment

- `cargo test` needs a BOLI build; it builds one automatically.
- CLI tests write temp `.hl` files under `CARGO_TARGET_TMPDIR` and use per-test unique file names so the parallel test runner doesn't race.

## Adding a test

Put assertion-style tests in `tests/boli.rs` (use `run_out(src) -> Result<Vec<String>, BoliError>` to get captured output), or black-box CLI tests in `tests/cli.rs` (`run_with_stdin("file_name.hl", src, stdin) -> (status, stdout, stderr)`). Prefix temp files with the test's purpose to avoid collisions.

## Manual regression

```bash
cargo run -- run examples/namaste.hl
cargo run -- run examples/calculator.hl
cargo run -- run examples/calculator2.hl   # input: 6 7 guna
cargo run -- run examples/loop.hl
cargo run                                   # REPL smoke test
```
