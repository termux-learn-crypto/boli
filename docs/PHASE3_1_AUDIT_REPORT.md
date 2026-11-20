# Phase 3.1 Audit Report — Self-Hosted BOLI Lexer

**Scope:** audit, verify, fix, harden, test, and document the existing experimental BOLI lexer in `selfhost/lexer/`.
**Repository:** `/data/data/com.termux/files/home/boli`
**Baseline:** Phase 3 commit `c10129a BOLI Phase 3: experimental self-hosted lexer and differential tests`.
**Reference:** Rust lexer `src/lexer.rs`; differential harness `tests/phase3_lexer.rs`.
**Out of scope:** Phase 4 parser, AST helpers, compiler, bytecode, default CLI wiring, Rust lexer deletion.

## A. Feature matrix

Classifications use only source and test evidence.

| Feature | Classification | Evidence |
|---|---|---|
| Token kinds (`Likho`, `Number`, `Text`, `Word`, `Newline`, `Eof`) | IMPLEMENTED AND VERIFIED | `selfhost/lexer/token.boli`; canonical output asserted in `tests/phase3_lexer.rs`. |
| Keyword/identifier/number/string/comment/whitespace lexing | IMPLEMENTED AND VERIFIED | 8 differential parity tests cover the supported corpus. |
| 1-based line/column positions | IMPLEMENTED AND VERIFIED | Canonical comparison uses `KIND|value|line|col`. |
| UTF-8 byte caps for source, text literals, and words | IMPLEMENTED AND VERIFIED | `lexer_byte_length`; `phase3_unicode_byte_limits`. |
| Negative numeric literals | IMPLEMENTED AND VERIFIED | `lexer_peek_next` gate; `phase3_negative_number_boundaries`. |
| Finite-number precheck before `sankhya` | IMPLEMENTED AND VERIFIED for supported decimal syntax | Exact 309-digit boundary; `phase3_numeric_finite_boundaries`. |
| Error `line:col:message` parity with Rust | IMPLEMENTED AND VERIFIED | All differential error cases compare canonical `line:col:message`. |
| BOLI error `kind` values | IMPLEMENTED AND VERIFIED inside BOLI | `phase3_boli_error_kinds`; not compared to Rust because `BoliError` has no `kind`. |
| Internal cursor threading (`{line, col, pos}`) | IMPLEMENTED AND VERIFIED | Used by all lexer helpers and exercised by the harness. |
| Public incremental `state` in result maps | NOT IMPLEMENTED | `lexer_ok`/`lexer_fail` set `state` to `khaali`; documented as reserved. |
| Full Rust `char::is_alphanumeric()` identifier parity | NOT IMPLEMENTED | Rust has 858 ranges; BOLI has ASCII plus curated mark/Indic ranges. Greek `α` gap is test-documented. |
| Byte offsets / spans | NOT IMPLEMENTED | Token maps carry line/col only. |
| File I/O, bytecode, AST, parser, compiler, default wiring | NOT IMPLEMENTED | Explicit Phase 3 non-goals; no Phase 4 code added. |

## B. Confirmed bugs / issues and fixes

1. **A — Negative numbers:** `-` entered the number branch unconditionally. Bare `-`, `-abc`, `--5`, `-.5`, and glued forms were not reliably matched to Rust. Fixed by computing `neg_digit` from `lexer_peek_next st` and entering the number branch only for a digit or `-` immediately followed by a digit.
2. **B — Numeric overflow / finite parsing:** the old digit-count/leading-digit heuristic was approximate, and the post-`sankhya` roundtrip could hit a different interpreter error for non-finite values. Fixed with `lexer_overflow_prefix`, `lexer_trim_leading_zeros`, and an exact integer-part comparison against Rust's first non-finite 309-digit decimal boundary before `sankhya`.
3. **C — Unicode bytes vs chars:** `lambayi` counts Unicode scalar values, while Rust caps UTF-8 bytes. Fixed by adding `lexer_byte_length` and using it for `max_source`, `max_string`, and word-size checks.
4. **D — Unicode identifiers:** only ASCII plus the curated ranges copied from `src/lexer.rs` are recognized. Rust accepts the full `is_alphanumeric()` set. Not silently "fixed"; the gap is documented in `docs/LEXER_TOKEN_SPEC.md`, `docs/SELF_HOSTED_LEXER.md`, and `phase3_documented_unicode_identifier_gap`.
5. **E — State/result contract:** docs claimed `state` exposed a usable final cursor, but `errors.boli` has always returned `khaali`. Kept runtime behavior unchanged to avoid a wire-format change and corrected docs to reserve the field for future incremental use.
6. **F — Error categories:** docs listed stale kinds (`too_many_tokens`, `invalid_escape`, `number_too_large`, `internal`). Corrected to the actual emitted kinds: `source_too_large`, `token_limit`, `text_too_large`, `word_too_large`, `unterminated_text`, `number_not_finite`, `unexpected_character`, plus `stuck_minus`. Unknown escapes remain literal, matching Rust, so no `invalid_escape` kind exists.

Every confirmed mismatch above was fixed or explicitly documented with justification. No resource limit was removed and no failing test was hidden.

## C. Missing features and deliberate non-implementation

- Full Unicode identifier parity is not implemented; it needs a general `is_alphanumeric`-equivalent table or builtin.
- Public incremental lexer state is not implemented; exposing it now would change the result contract.
- Byte spans/offsets are not produced; tokens carry line/col only.
- A general `byte_lambayi` builtin is not added; Phase 3.1 uses a lexer-local helper only.
- Phase 4 parser, AST helpers, compiler, bytecode emitter, VM integration, and CLI wiring are not implemented.
- The Rust implementation remains authoritative; no Rust source behavior was changed.

## D. Changed files

Source and tests:

- `selfhost/lexer/lexer.boli` — negative-number gate, byte-length helper, leading-zero trim, exact finite boundary, byte-based caps, finite precheck, removed heuristic/roundtrip.
- `tests/phase3_lexer.rs` — five new tests: negative boundaries, finite boundaries, UTF-8 byte caps, BOLI error kinds, documented Unicode gap.

Documentation:

- `docs/PHASE3_1_AUDIT_REPORT.md` — this report.
- `docs/LEXER_TOKEN_SPEC.md` — byte caps, state contract, corrected number/identifier rules, corrected error kinds.
- `docs/SELF_HOSTED_LEXER.md` — internal vs public state, exact finite precheck, expanded harness coverage, updated limitations.
- `docs/PHASE3_REPORT.md` — Phase 3.1 summary, test list/counts, criteria, limitations, commit status.
- `docs/SELF_HOSTING_ROADMAP.md` — current snapshot, current gate, byte-length note, test baseline.
- `docs/TEST_RESULTS.md` — current gate and Phase 3.1 verification addendum.
- `docs/SECURITY_REPORT.md`, `docs/COMPLETE_AUDIT.md` — current gate count addenda.
- `README.md`, `TESTING.md`, `CHANGELOG.md` — current test count and unreleased Phase 3.1 note.

No files were deleted. No Rust source, CLI path, parser path, VM path, release workflow, or tag was changed.

## E. Exact test commands and results

Environment prefix used for every cargo command:

```bash
export PATH=/data/data/com.termux/files/usr/bin:$PATH
```

Phase 3 harness:

```bash
cargo test --test phase3_lexer --locked --quiet
# 10 passed; 0 failed
```

Demo:

```bash
cargo run --quiet -- run "selfhost/lexer/demo.boli"
# Likho||1|1
# Number|5|1|7
# Newline||1|8
# Eof||2|1
```

Full repository gate:

```bash
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked --quiet
cargo build --release --locked
cargo run --release --locked --quiet -- version
```

Observed: all commands passed; test totals **214 passed, 0 failed, 2 ignored**; release binary reports `BOLI version 0.6.5`.

Additional checks:

```bash
python3 - <<'PY'
from pathlib import Path
import yaml
for path in sorted(Path('.github/workflows').glob('*.yml')):
    yaml.safe_load(path.read_text())
    print(f"OK {path}")
PY
# OK .github/workflows/ci.yml
# OK .github/workflows/release.yml

git diff --check
# clean
```

Boundary strings were probed with temporary `rustc` files outside the repository; no probe files were added to the repo.

## F. Security impact

- All existing resource caps remain enforced; `max_source`, `max_string`, and token limits were tightened to Rust-equivalent byte semantics where applicable.
- `lexer_byte_length` performs bounded, linear work over source-bounded input and adds no allocation.
- `lexer_trim_leading_zeros` allocates only within an already source-bounded numeric token.
- Finite-number rejection happens before `sankhya`, avoiding non-finite values entering the token stream.
- No file I/O, network, process execution, raw bytes, FFI, or new global mutable state was introduced.
- Error messages remain Hinglish and bounded by source/limit values; no debug dumps or secrets are exposed.

## G. Remaining limitations

- Full Unicode identifier parity remains missing; Greek `α` is an intentional verified gap.
- `state` is reserved but always `khaali`.
- `lexer_byte_length` walks every character, so very large inputs cost more BOLI steps than Rust's `len()`.
- No byte offsets/spans; AST/parser diagnostics will need a future span design.
- BOLI `kind` values are internal only; Rust `BoliError` has no comparable typed field.
- Finite-boundary verification covers the decimal syntax both lexers accept; exponent notation is not part of this corpus.
- `run.boli` executes at import time; the harness deliberately imports implementation modules only.
- The lexer remains additive and is not wired into the default CLI, parser, interpreter, or VM.

## H. Phase 3 status choice

**Keep Phase 3 status as: experimental self-hosted lexer complete at the lexer boundary only.**

Do **not** promote this to "self-hosting complete" or a stable Phase 4 foundation. The lexer now has stronger differential coverage and byte/number parity for the tested corpus, but parser, AST, bytecode, spans, and runtime integration do not exist. Phase 3.1 is an audit/fix pass over Phase 3, not a new compiler milestone.

## I. Phase 4 readiness score

| Readiness area | Score | Basis |
|---|---:|---|
| Token-stream fidelity for supported syntax | 4/5 | 8 parity tests, negative/finite/byte fixes; Unicode identifier gap remains. |
| Parser input contract | 3/5 | Tokens have line/col and stable formatting, but no spans or typed Rust parity. |
| Error/diagnostic fidelity | 3/5 | `line:col:message` matches; BOLI kinds are internal only. |
| Integration maturity | 2/5 | Additive only; no parser, default wiring, or build pipeline. |
| **Overall Phase 4 parser readiness** | **3/5** | Safe to design as an additive experiment; not safe to call stable. |

## J. Conclusion

Phase 3.1 is complete: A–F were audited, mismatches were fixed or explicitly documented, the harness grew from 5 to 10 tests, and the full repository gate is green. Commit and push status is recorded in git history rather than this report; review `git status` and `git diff` before any further commit, and commit only when explicitly asked. No Phase 4 implementation was performed.
