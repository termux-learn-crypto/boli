# VM Stability Status

## Current status

The bytecode VM is **experimental**. It is enabled only with `--vm` or `BOLI_VM=1`. The tree-walking interpreter remains the default execution engine.

This document tracks Phase 1 Milestone 1. It does not make the VM stable by itself.

## Done in this pass

- `tests/vm_corpus.rs`: shared interpreter-vs-VM corpus covering arithmetic, conditions, variables, lists, maps, loops, kaam, builtins, runtime errors and tracebacks.
- `tests/vm_fuzz.rs`: deterministic generated-source smoke test; `BOLI_FUZZ_CASES` controls case count, default is 2000. `vm_fuzz_100k` is an ignored full gate.
- `tests/vm_bench.rs`: ignored benchmark harness for loops, lists, maps and recursion; no external dependencies.
- `docs/BYTECODE_FORMAT.md`: BOLI-BC v1 opcode and file format frozen on paper.

Local checks passed:

- `cargo test --test vm_corpus`
- `cargo test --test vm_fuzz`
- `cargo test --test vm_fuzz -- --ignored` — 100k cases passed locally in ~8.66s.
- `cargo test --test vm_bench -- --ignored --nocapture`

## Remaining gates before “stable”

- [ ] BOLI-BC v1 serializer/loader implemented in Rust VM.
- [ ] Same AST produces identical `.bc` bytes across runs.
- [ ] VM runs a `.bc` corpus with output/error parity against AST execution.
- [ ] 100k deterministic fuzz cases pass without panic, hang or OOM.
- [ ] Full suite green on Linux, macOS and Windows CI.
- [ ] Reachable internal `expect`/`unwrap` panic paths audited and fixed.
- [ ] Disassembler available for debugging `.bc` files.
- [ ] Docs updated from **experimental** to **stable** only after all gates above are green.

## Behavioral compatibility rules

- Same source must produce the same printed output for both engines.
- Same source must fail under both engines with the same user-facing error string for agreed error cases.
- Supported feature differences must be documented here; silent differences are bugs.
- Resource limits must apply to both engines.
- Any VM-only optimization must preserve line numbers and Hinglish error messages.

## Rollback policy

All VM work stays behind `BOLI_VM` / `--vm`. If a compatibility gate fails, revert the VM-only commit. The interpreter and its 204-test default regression suite remain authoritative.
