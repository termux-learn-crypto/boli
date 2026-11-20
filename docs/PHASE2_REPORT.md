# Phase 2 Report — Stable Runtime + Core Standard Library

## Result summary

Phase 2 ka core runtime aur standard-library portion complete hai. 31 naye additive builtins add hue; existing 6 builtins, parser, interpreter, VM, security limits aur existing tests preserve rahe. File I/O implement **nahi** hua — uska safe design `FILE_IO_SECURITY.md` me documented hai.

Status: **core runtime ready for compiler-stage design; full self-hosting readiness still blocked** by VM stability gates, typed errors, function values, and file capabilities. Interpreter abhi authoritative default hai.

## Inspect kiya gaya

- `src/builtins.rs`: 6 builtins, direct dispatch, no registry constants.
- `src/interpreter/value.rs`: 6 copy-semantics values; no integer, bytes, function, error, iterator, reference.
- `src/interpreter/mod.rs` aur `eval.rs`: innermost assignment, call scopes, separate global `kaam`, builtin shadowing.
- `src/vm.rs`: yahi builtin dispatch; same value representation; VM experimental.
- `src/ast.rs`: `BoliError { line, col, message, trace }`; no `kind`/`path` at that time. Phase 3.2 later added lexer `kind` and boxed `span`; `path` remains absent.
- `src/limits.rs`: 15 safety limits with env overrides; defaults preserved.
- `tests/`: 198 default tests before this phase, 2 ignored.
- CLI/help/docs: stale builtin list aur test counts mile.

## Plan (follow kiya gaya)

1. Pehle Milestone 1 VM-stability support finish karo: corpus, fuzz, bench, bytecode/stability docs.
2. Runtime surface audit karo without changing behavior.
3. Additive builtins design karo: string, list, math, type/testing.
4. Har builtin ke liye arity/type/limit/finite checks.
5. Interpreter + VM parity tests likho.
6. Docs lock karo: runtime spec, stdlib, errors, file-I/O design.
7. Full gate chain run karo; failures hide mat karo.

## Changes

### Code

- `src/builtins.rs`
  - `BUILTINS` constant with 37 names.
  - New dispatch arms aur helper implementations.
  - Shared helpers: `expect_text`, `expect_list`, `expect_number`, `index_1`, `slice_bounds`, `checked_list`, `checked_text_parts`.
  - Limit-aware outputs: `max_string_bytes`, `max_list_items`.
  - 1-based inclusive indexing, finite-number checks, explicit Hinglish errors.
- `src/main.rs`
  - CLI help abhi 37 builtins ka grouped summary aur `docs/STANDARD_LIBRARY.md` pointer dikhata hai.

### Tests

- `tests/stdlib.rs` new: 6 tests.
  - string ops, list ops, math ops, type/test ops.
  - negative cases and interpreter-vs-VM error parity.
  - direct `call_builtin` limit enforcement tests.
- Milestone 1 support files bhi is branch me hain:
  - `tests/vm_corpus.rs` — 6 differential tests, 33 inline cases.
  - `tests/vm_fuzz.rs` — default 2000 deterministic cases; ignored 100k gate.
  - `tests/vm_bench.rs` — ignored micro-benchmark harness.

### Docs

- `docs/RUNTIME_SPEC.md` new.
- `docs/STANDARD_LIBRARY.md` new.
- `docs/ERROR_HANDLING.md` new.
- `docs/FILE_IO_SECURITY.md` new (design only).
- `docs/PHASE2_REPORT.md` new.
- `docs/BYTECODE_FORMAT.md`, `docs/VM_STABILITY.md`, `docs/SELF_HOSTING_ROADMAP.md` from Milestone 1 support.
- `docs/VM_STABILITY.md` test count updated to current suite.

## Standard library surface

Poori list `STANDARD_LIBRARY.md` me hai. Summary:

- Existing (6): `lambayi`, `upar`, `neeche`, `text`, `sankhya`, `chaabiyan`.
- Text (10): `kato`, `khojo`, `hai_text`, `badlo`, `baton`, `jodo_text`, `chhilo`, `akshar`, `escape_text`, `unescape_text`.
- List (10): `jodega`, `dalein`, `hatao`, `ant`, `ant_hatao`, `tukda`, `nakal`, `saaf`, `ultana`, `ek_hai`.
- Math (7): `purnank`, `uparank`, `nirapeksh`, `mool`, `shesh`, `sabse_chhota`, `sabse_bada`.
- Type/test (4): `prakaar`, `prakaar_hai`, `sahi_banao`, `saman`.

Compatibility note: nested builtin calls as arguments parse nahi hote. Intermediate variable use karo.

## Verification

Command:

```sh
export PATH=/data/data/com.termux/files/usr/bin:$PATH
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked --quiet
cargo build --release --locked
cargo run --release --locked --quiet -- version
```

Results:

- `cargo fmt --all -- --check` — pass.
- `cargo check --all-targets --locked` — pass.
- `cargo clippy --all-targets --locked -- -D warnings` — pass.
- `cargo test --all-targets --locked --quiet` — pass.
  - 204 tests passed, 2 ignored, 206 declared.
  - Breakdown: lib 4, boli 114, cli 34, stdlib 6, vm 39, vm_corpus 6, vm_fuzz 1.
- `cargo build --release --locked` — pass.
- `cargo run --release --locked --quiet -- version` → `BOLI version 0.6.5`.
- Ignored full gate: `cargo test --test vm_fuzz -- --ignored` → 1 passed in ~8.66s.
- Ignored benchmark: `cargo test --test vm_bench -- --ignored --nocapture` → pass.
  - `loop_1000`: interp 10.69ms, VM 13.21ms.
  - `list_40`: interp 3.10ms, VM 2.88ms.
  - `map_100`: interp 2.91ms, VM 3.12ms.
  - `recursion_15`: interp 106.55ms, VM 79.03ms.

Clippy fix during verification: `tests/vm_fuzz.rs` me `% 7 == 0` ko `is_multiple_of(7)` kiya gaya; koi behavior change nahi.

## Non-goals / deferred

- File I/O: not implemented; capability+security design ready.
- Typed errors: no `kind` or `path` field yet.
- Function values/closures: absent.
- Integer/bytes/iterator/error value types: absent.
- Outer-scope mutation: absent.
- VM declared stable: nahi; `VM_STABILITY.md` gates abhi open hain.
- Existing Rust code remove/divide/release: nahi.

## Self-hosting readiness

Compiler ke liye abhi stable base:
- predict 37 builtins with fixed signatures,
- documented value/scope/limit/error model,
- parity tests between interpreter aur VM,
- stdlib helpers jo compiler-generated code call kar sakta hai.

Compiler se pehle ya parallel me ye zaroori hain:
1. `.bc` serializer/loader aur deterministic bytecode corpus.
2. Typed `BoliError` fields with backward-compatible display.
3. File capability + security tests (sirf tab enable jab user approve kare).
4. Value model me function/reference ya compiler ABI equivalent.
5. Full CI green on Linux, macOS, Windows with 206 declared tests plus future compiler tests.

## Rollback / preservation

- Koi existing test delete nahi hua.
- Koi security limit remove nahi hua.
- Rust bootstrap/reference code preserve hai.
- VM changes feature flag ke peeche hain.
- Current work commit/push/release nahi kiya gaya.
