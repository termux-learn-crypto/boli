# BOLI — Upgrade Roadmap (post-v0.6.4)

Prioritized by fit with BOLI's philosophy: **Hinglish syntax, no symbols, learning-friendly, zero-dependency Rust, predictable behavior.** Every item below must keep existing `.hl` programs valid. Estimates assume one experienced Rust engineer.

**Current status:** v0.6.4. Maps, modules, built-ins, tracebacks, diagnostics, the experimental VM, bounded reads, import depth/cycle checks, optional `BOLI_IMPORT_ROOT` confinement, bounded hints/REPL buffers, nested-value depth caps, runtime-accurate VM registration, and four audit passes are done. Authoritative test/status page: [TEST_RESULTS.md](TEST_RESULTS.md). Security scope: [SECURITY_REPORT.md](SECURITY_REPORT.md).

## Guiding constraints
1. No new symbols (`{ } ( ) = :` stay illegal) — only keywords/words.
2. No mandatory dependencies. Any future dep needs a written justification and a CVE review.
3. Every resource growth path gets an env-tunable `BOLI_MAX_*` (or `BOLI_*_ROOT`) limit and a matching entry in SECURITY_REPORT.md.
4. Semantics changes ship only behind a minor-version bump + CHANGELOG callout; regressions are bugs.
5. A new backend is not authoritative until it passes the same differential corpus as the interpreter; step counts are backend-specific and must not be presented as identical.
6. Numbers stay f64 unless a dedicated `puran` (integer) type is adopted with i128 semantics — see below.

## Done (do not re-plan)
- v0.4.0: built-in kaams (`lambayi`, `upar`, `neeche`, `text`, `sankhya`), column diagnostics and caret output.
- v0.5.0: runtime tracebacks, interpreter module split.
- v0.6.0: maps (`mapa`), modules (`laao`), experimental bytecode VM, did-you-mean hints.
- v0.6.1: bounded import reads, `BOLI_MAX_IMPORT_DEPTH`, nested relative imports, 64-bit VM operands, error-path unwinding, built-in caps.
- v0.6.2: drained over-limit input, VM stack/step reset, `BOLI_IMPORT_ROOT`, differential corpus tests, refreshed audit/docs status.
- v0.6.3: bounded did-you-mean hints, aggregate REPL buffer cap, canonical entry-file module registration, third audit pass.
- v0.6.4: full-audit fix pass (15 findings), nested value depth cap, overflow-safe limits, runtime-accurate VM kaam/import effects, parser/REPL hardening, fourth audit pass.

## v0.7.0 (next — tooling and predictability)
| Feature | Sketch | Why / risk |
|---|---|---|
| **`boli fmt` (formatter)** | Re-indent by block depth; idempotent; parse-only | Teaches style; reuses parser; no semantics. |
| **`boli check`** | Parse + lint: unused kaam, unreachable statements, import graph preview | Cheap feedback loop; must not execute `.hl`. |
| **`puran` integer literals** | `puran 5` typed values, exact ≤ 2^127 | Fixes >2^53 precision loss; opt-in keyword, backward compatible. Needs new cap entries for integer text/ops. |
| **Richer `pucho` types** | `pucho sahi naam` etc. explicit type hints | Removes guessy inference; opt-in keyword form. |
| **REPL paste mode** | `paste … .` block delimiters | Fixes multiline heredoc paste; small, self-contained. |

## v0.8.0 (capabilities — explicit opt-in only)
| Feature | Sketch | Risk |
|---|---|---|
| **File I/O under explicit capability** | `padho`, `likho_file` gated by `BOLI_ALLOW_FS=1` + `BOLI_FS_ROOT`; off by default | Highest security risk. Must never default on; every primitive needs its own cap and SECURITY_REPORT entry. Reuse canonicalized-root pattern from `BOLI_IMPORT_ROOT`. |
| **Standard library expansion** | More pure Rust kaams (`chhota`/`bada`, join/split) | Same rules: no I/O, caps enforced, parity tests for both backends. |
| **REPL trace mode** | `boli trace x.hl` dumps step/scope/kaam-call trace for teaching/debugging | Output can be large; cap trace frames and redact nothing by default (source is local). |
| **Editor integration** | Tree-sitter grammar + VS Code extension | Grammar already exists in effect as the lexer; keep keyword table synchronized. |

## Long-term / research (no version)
- **Operational semantics doc** — written PLT-Redex-style rules for BOLI's evaluation, including limits.
- **Concurrency sketch** — `ek saath … khatam` parallel blocks over immutable values only (no shared mutable scope); needs real design time.
- **Self-hosted stdlib** — once capabilities are stable, implement parts of stdlib in BOLI itself.
- **WASM build** — run BOLI in the browser for the tutorial site; pure logic, `pucho` stubbed, separate security review.
- **Language server** — `boli lsp` for editor integration after the AST/symbol table stabilizes.

## Versioning policy
- Patch: crash → error fixes, docs, tests, hardening with no syntax change (v0.6.1, v0.6.2, v0.6.3, v0.6.4).
- Minor (0.x): new syntax, new limits, new capabilities, or behavior old programs may notice — all called out in CHANGELOG.
- 1.0.0 when: modules + stdlib stable, VM parity suite green, no known correctness bugs, and ≥1 external contributor.

## Explicitly rejected (and why)
- Symbols (`{}` `()` `=`): violates the language's core identity.
- Exceptions with symbols: BOLI uses `Galti` + structured recovery only.
- Dynamic `eval` of text: breaks every safety limit guarantee; rejected permanently.
- Unbounded integers by default: would hide f64 semantics people rely on; hence opt-in `puran`.
- Package manager / registry: no network dependency; `laao` stays local-file only.

## Migration notes for maintainers
- Any change to `atoms`/`hissa` grammar is breaking → minor bump + LANGUAGE_SPEC §11 entry.
- Adding a limit must update: `src/limits.rs`, SECURITY_REPORT matrix, README/LANGUAGE_SPEC limits tables, `run_out`/CLI/VM tests.
- Adding a root/confinement variable must update `resolve_import_path` callers (interpreter + VM), SECURITY_REPORT, and at least one CLI test per engine.
- New keyword: add to `keyword()` in lexer, SPEC §1 keywords table, REPL depth tracker, and ≥3 tests.
