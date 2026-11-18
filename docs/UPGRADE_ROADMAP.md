# BOLI — Upgrade Roadmap (post-v0.3.0)

Prioritized by fit with BOLI's philosophy: **Hinglish syntax, no symbols, learning-friendly, zero-dependency Rust, predictable behavior.** Every item below must keep existing `.hl` programs valid. Estimates assume one experienced Rust engineer.

## Guiding constraints
1. No new symbols (`{ } ( ) = :` stay illegal) — only keywords/words.
2. No mandatory dependencies. Any future dep needs a written justification and a CVE review.
3. Every resource growth path gets an env-tunable `BOLI_MAX_*` limit and a matching entry in SECURITY_REPORT.md.
4. Semantics changes ship only behind a minor-version bump + CHANGELOG callout; regressions are bugs.
5. Numbers stay f64 unless a dedicated `puran` (integer) type is adopted with i128 semantics — see below.

## v0.4.0 (next, stability + ergonomics)
| Feature | Sketch | Why / risk |
|---|---|---|
| **Map / dictionary type** | `ginti_map rakho` + `ginti_map hissa "naam"` (text keys only) | The single most requested missing type; mirrors list syntax exactly, no new symbols. Keys as text keeps `hissa` grammar valid. |
| **`puran` integer literals** | `puran 5` typed values, exact ≤ 2^127 | Fixes >2^53 integer precision loss; opt-in via type keyword, backward compatible. |
| **Richer `pucho` types** | `pucho sahi naam` etc. explicit type hints | Removes guessy inference; opt-in keyword form. |
| **REPL paste mode** | `paste … .` block delimiters | Fixes multiline heredoc paste; small, self-contained. |
| **`boli fmt` (formatter)** | Re-indent by block depth; idempotent | Teaches style; reuses parser; no semantics. |

## v0.6.0 (modules & files — behind explicit keywords)
| Feature | Sketch | Risk |
|---|---|---|
| **Modules: `laao "path.hl"`** | Loads sibling `.hl`, exposes its top-level `kaam`s in an isolated namespace | Needs import cycles error, path confinement (`--allow-fs` flag or `BOLI_FS_ROOT`), tests. High value, high care. |
| ~~Standard library (built-in kaams)~~ **Done in v0.4.0** (`lambayi`, `upar`, `neeche`, `text`, `sankhya`) | More: `chhota`/`bada` min-max, `jodai` join/split | Same rules: Rust-only, caps enforced. |
| **File I/O under explicit capability** | `padho "file.hl"`, `likho_file` — gated by `BOLI_ALLOW_FS=1`; off by default | Security: must never default on. Feeds SECURITY_REPORT update. |
| **Better error locations** | Column numbers; spans under `^^^` in diagnostics | Parser already tracks lines; columns are moderate work. |

## v0.7.0 (tooling)
| Feature | Sketch |
|---|---|
| **Syntax highlighting** | Tree-sitter grammar (`tree-sitter-boli`) + VS Code extension — grammar already exists in effect as the lexer. |
| **Bytecode compiler + VM** | `boli compile x.hl -o x.bolc`; stack VM; 5–20× speedup; same limits enforced; `Environment` preserved for parity tests (run both backends on the 82-test suite). |
| **REPL trace mode** | `boli trace x.hl` dumps step/scope/kaam-call trace for teaching debugging. |
| **`boli check`** | Parse-only + lint (unused kaam, unreachable statements) without executing. |

## Long-term / research (no version)
- **Algebraic-ish teaching semantics doc** — a written operational semantics (PLT-Redex-style) for BOLI's evaluation rules.
- **Concurrency sketch** — `ek saath … khatam` parallel blocks over immutable values only (no shared mutable scope) — needs real design time.
- **Self-hosted stdlib** — once modules land, parts of stdlib in BOLI itself.
- **WASM build** — run BOLI in the browser for the tutorial site; pure logic, no stdio (pucho stubbed).
- **Language server** — `boli lsp` for editor integration after the AST/symbol table is stable.

## Versioning policy
- Patch: crash → error fixes, docs, tests.
- Minor (0.x): new syntax, new limits, behavior that old programs may notice (all called out in CHANGELOG).
- 1.0.0 when: modules + stdlib stable, VM parity suite green, no known correctness bugs, and ≥1 external contributor.

## Explicitly rejected (and why)
- Symbols (`{}` `()` `=`): violates the language's core identity.
- Exceptions with symbols: BOLI uses `Galti` + structured recovery only.
- Dynamic `eval` of text: breaks every safety limit guarantee; rejected permanently.
- Unbounded integers by default: would hide f64 semantics people rely on; hence the opt-in `puran`.
- Package manager: no registry; `laao` for local files only.

## Migration notes for maintainers
- Any change to `atoms`/`hissa` grammar is breaking → minor bump + LANGUAGE_SPEC §11 entry.
- Adding a limit must update: `src/limits.rs`, SECURITY_REPORT matrix, `run_out`-style tests, README limits section.
- New keyword: add to `keyword()` in lexer, SPEC §1 keywords table, REPL depth tracker, and ≥3 tests.
