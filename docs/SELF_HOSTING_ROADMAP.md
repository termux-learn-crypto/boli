# BOLI — Rust Independence & Self-Hosting Architecture Audit

**Status:** Phase 1 + Phase 2 + Phase 3 implemented, Phase 3.1 audit/fix applied, Phase 3.2 lexer hardening applied, Phase 4A parser design complete (design only)
**Repository:** `/data/data/com.termux/files/home/boli`
**Audited snapshot:** `main` at `c10129a BOLI Phase 3: experimental self-hosted lexer and differential tests` for the original roadmap; release tag `v0.6.5` points to `13bc28b`
**Current task scope:** Phase 3.2 lexer spans, structured errors, Unicode parity, differential hardening, and documentation only. No Phase 4 parser implementation.

> **Phase 3.2 update:** the original roadmap below is retained as the Phase 3/3.1 baseline. Phase 3.2 closed the Unicode identifier gap with `selfhost/lexer/unicode.boli`, added six span fields to tokens and errors, added Rust `BoliError.kind`/boxed span, and expanded the full gate from 214 to **215 passed / 0 failed / 2 ignored**. Parser-facing truth is `docs/TOKEN_CONTRACT.md` and `docs/PARSER_INTERFACE_SPEC.md`; claims below that spans or typed Rust error kinds are missing are historical. Phase 4 readiness is **READY WITH LIMITATIONS**, not full self-hosting readiness.

> **Phase 4A update:** parser design is now documented in `docs/PARSER_GRAMMAR.md`, `docs/PARSER_AST_SPEC.md`, `docs/PARSER_ARCHITECTURE.md`, `docs/PARSER_ERROR_RECOVERY.md`, and `docs/PHASE4A_REPORT.md`. This phase was design-only: no BOLI parser, AST, lexer, VM, CLI, or Rust parser implementation was added. The design keeps the current recursive-descent grammar, flat call arguments, contextual `kaam`/`hissa`, fail-fast errors, and line-only Rust AST compatibility. Phase 4B implementation readiness is **READY WITH BLOCKERS**; the roadmap below remains the implementation order.

---

## 0. Executive summary

### 0.1 Clear answer: kitna ready hai?

**BOLI abhi self-hosting ke liye “early foundation” stage me hai. Ready nahi hai — lekin road blocked bhi nahi hai.**

| Area | Readiness | Basis |
|---|---:|---|
| Stage 0 Rust reference implementation | **5/5** | `src/` me complete lexer, parser, AST, interpreter, VM, limits, builtins, CLI, tests hain. 215 tests passed, 2 ignored in the current gate. |
| Core language logic for a compiler | **3/5** | Recursion, `kaam`, lists, maps, `hissa`, comparisons, `wapas do` exist. Enums/pattern matching/closures nahi hain, par tagged maps + explicit state threading se prototype ban sakta hai. |
| Systems capability for a compiler | **1/5** | No general file read/write, no bytes type, no binary output, no structured catchable errors, no source spans in AST, no BOLI-level bytecode API. |
| VM as a stable compiler target | **3/5** | 39 VM tests + parity suite exist, but VM explicitly experimental (`src/vm.rs:1`), no bytecode serialization/loader, no fuzz corpus dedicated to VM, no benchmarks. |
| Build/release independence from Rust | **1/5** | `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `cargo ... --locked` in CI/release, `target/release/boli` in installer. |
| **Overall self-hosting readiness** | **2/5** | Design shuru karne ke liye kaafi hai; Stage 1 compiler likhne ke liye nahi. |

**Start gate:** compiler migration tab shuru nahi karni jab tak Milestone 1 (VM stabilization), Milestone 2 (runtime primitives) aur Milestone 3 (sandboxed file/byte I/O) complete aur green na ho. Ye user rule se direct aata hai: “VM ko stable banaye bina compiler migration start mat karo.”

### 0.2 Final goal ke against current truth

| Long-term goal | Current status |
|---|---|
| Compiler BOLI me likha ho | **Nahi.** Compiler Rust me hai (`src/lexer.rs`, `src/parser.rs`, `src/vm.rs:62` `Compiler`). |
| Lexer BOLI me ho | **Nahi.** `src/lexer.rs` Rust. |
| Parser BOLI me ho | **Nahi.** `src/parser.rs` Rust. |
| AST BOLI me ho | **Nahi.** `src/ast.rs` Rust enums; BOLI side me AST value type exist nahi karta. |
| VM/runtime BOLI me ho | **Nahi.** `src/vm.rs`, `src/interpreter/*`, `src/builtins.rs`, `src/limits.rs` Rust. |
| BOLI apne compiler ko khud compile kare | **Nahi.** Iske liye Milestone 9 chahiye. |
| Final build/release me Rust required na ho | **Nahi.** `ci.yml`, `release.yml`, `install.sh`, `Cargo.toml` Rust/Cargo-centric hain. |
| ARM64/x86-64 native codegen | **Nahi.** Release matrix sirf 4 fixed Rust targets deta hai; codegen exist nahi karta. |

### 0.3 Recommended sequence

`VM stabilize` → `runtime primitives` → `sandboxed FS/bytes` → `bytecode format + Rust compiler emits bytecode` → `BOLI lexer` → `BOLI parser` → `BOLI AST helpers` → `BOLI bytecode compiler` → `self-host check` → `reproducibility` → `Rust-free release` → `native codegen`.

Har step par existing gate (ab 215 passed, 2 ignored) plus naye tests green honi chahiye. Rust implementation ko delete nahi karna; use **bootstrap/reference implementation** ke roop me preserve karna hai.

---

## 1. Phase 1 — Existing repository audit

### 1.1 Module-by-module audit and Rust dependency map

| Module | Actual file(s) | What it does today | Rust dependency? | Compiler migration relevance |
|---|---|---|---|---|
| Lexer | `src/lexer.rs` | `Tok`, `Token { tok, line, col, span }`, byte-aware cursor, keywords, strings, numbers, comments, limits. | Yes, pure Rust. | BOLI lexer ka reference. Phase 3.2 added full Unicode identifier parity via `unicode.boli`; general `byte_lambayi` builtin abhi bhi nahi hai. |
| Parser | `src/parser.rs` | Recursive descent, 2-token lookahead, no backtracking, statement/expression/condition parsing. | Yes. | BOLI parser same recursive style me likha ja sakta hai; token boundary is now documented in `docs/TOKEN_CONTRACT.md`. |
| AST | `src/ast.rs` | Rust enums `Expr`, `Stmt`, `Cond`, `ArithOp`, `CmpOp`; AST nodes still store line only. | Yes. | BOLI me tagged maps se emulate hoga. Missing enums/pattern matching is main friction. |
| Error system | `src/ast.rs` | `BoliError { line, col, message, trace, kind, span }`, Hinglish `Display`. | Yes. | Lexer errors now have typed kind and boxed span; BOLI still has no catchable structured error value. |
| Scope | `src/scope.rs:8-28` | `Scope { map: HashMap<String, Value>, bytes: u64 }`. | Yes. | BOLI `kaam` writes sirf innermost scope me (`interpreter/mod.rs:232-252`), isliye compiler state explicitly thread karna padega. |
| Limits | `src/limits.rs:4-247` | 14 env-tunable caps, bounded reads, path resolution, `BOLI_IMPORT_ROOT`. | Yes. | Security model ka reference. BOLI programs se ye APIs accessible nahi hain. |
| Interpreter | `src/interpreter/mod.rs:33-687`, `eval.rs`, `value.rs` | Tree-walker default engine, scopes, kaam table, imports, budgets, value accounting. | Yes. | Semantic oracle. VM aur future BOLI compiler ke output isi se compare honge. |
| VM | `src/vm.rs:1-1270` | Experimental bytecode stack machine; AST → `Vec<Op>` + line table; runtime kaam/import ops. | Yes. | Compiler target. Stable opcode/format abhi nahi. |
| Builtins | `src/builtins.rs:8-191` | `lambayi`, `upar`, `neeche`, `text`, `sankhya`, `chaabiyan`. | Yes. | Compiler ke liye string/list/map basics hain, par file/bytes/binary nahi. |
| CLI / REPL | `src/main.rs:23-461` | `run`, `.hl` shortcut, help/version, `--vm`, REPL with depth heuristic. | Yes. | REPL interpreter-only hai (`main.rs:306`) even with `--vm`. New build pipeline ko CLI flags reproduce karne honge. |
| Imports | `interpreter/mod.rs:170-206`, `vm.rs:451-505,1066-1097`, `limits.rs:147-187` | `laao`, canonical path, root confinement, dedup, depth guard. | Yes. | Read-only imports exist; general file I/O nahi. Module namespace/export nahi. |
| Tests | `tests/boli.rs`, `tests/cli.rs`, `tests/vm.rs`, `tests/phase3_lexer.rs`, embedded unit tests | Historical v0.6.5 baseline 191 tests; current gate 215 passed, 2 ignored. | Rust test harness. | Golden corpus preserve karna hai. BOLI-level test runner abhi nahi. |
| CI/CD | `.github/workflows/ci.yml`, `release.yml`, `dependabot.yml` | 3-OS CI, 4 release targets, checksums, pinned actions, Dependabot. | Cargo/Rust required. | Rust-free release ke liye replace hona hai. |
| Installer | `install.sh:1-57` | Copies local `target/release/boli`. | Needs Cargo-built binary path. | Non-Cargo artifact path ya installer change chahiye. |

### 1.2 Existing BOLI features — sufficiency for writing a compiler

**Fact basis:** `src/ast.rs:99-215`, `src/builtins.rs:8`, `src/interpreter/eval.rs`, `src/interpreter/mod.rs:600-687`.

| Needed by a compiler | Exists today? | Exact evidence | Verdict |
|---|---|---|---|
| Variables/assignment | Yes | `Stmt::Rakho`, `ArithAssign`, `ListAssign`, `MapRakho` (`ast.rs:136-154,203-214`) | Sufficient |
| Numbers | Yes, `f64` only | `Value::Number(f64)` (`value.rs:10-16`) | Risky for offsets; need `puran` integer |
| Text | Yes | `Value::Text`, escapes `\"`, `\\`, `\n` (`lexer.rs:287-356`) | Partially sufficient |
| Lists | Yes | `Expr::List`, `hissa` 1-based indexing, `jodo` concat | Sufficient for token streams |
| Maps | Yes, text keys only | `BTreeMap<String, Value>` (`value.rs:16`), `chaabiyan` | Sufficient for symbol tables |
| Recursion | Yes | `call_kaam` depth cap 64 (`mod.rs:639-647`) | Sufficient with limit tuning |
| Functions | Yes | `kaam`, params, `wapas do` (`ast.rs:175-192`) | Sufficient, but no first-class functions |
| Conditions/loops | Yes | `agar`, `jab tak`, range loop, `ruk`, `agla` | Sufficient |
| Imports | Yes, read-only | `laao` | Not enough for multi-file compiler output |
| File read | No general API | Only internal `read_source_capped` (`limits.rs:113-145`) and `laao` | Missing |
| File write | No | No statement/builtin | Missing |
| Bytes/binary | No | `Value` has no byte type (`value.rs:9-17`) | Missing |
| Structured errors | Partial | Rust lexer errors now have `kind` and boxed `span`; BOLI has no `try`/`throw` | Missing catchable BOLI value |
| Source spans | Partial | Lexer tokens/errors have full spans; AST only stores line | Missing for compiler-quality AST diagnostics |
| Enums / sum types | No | No `enum` in language; Rust uses enums | Workaround: tagged maps |
| Pattern matching | No | Only `agar ... barabar hai` | Workaround: nested condition chains |
| Closures / captured state | No | `Kaam` stores only params+body (`mod.rs:20-24`) | Workaround: explicit state passing |
| Bytecode emission API | No | `Op` Rust-only (`vm.rs:16-41`) | Missing |
| BOLI-level test runner | No | Rust `#[test]` only | Missing |
| Build manifest | No | Cargo-only | Missing |

**Conclusion:** existing language is enough for an **in-memory compiler prototype** if compiled state is threaded explicitly through functions and AST is encoded as tagged maps. It is **not enough for an end-to-end self-hosted toolchain** because compiler needs to read its own sources, write bytecode, inspect bytes, report structured diagnostics, and be tested from inside BOLI.

### 1.3 Important constraints discovered from actual code

1. **No symbols except `#` and decimal point.** Lexer rejects `; { } ( ) = : . @` (`lexer.rs:459-467`). A BOLI compiler must keep Hinglish word syntax; do not introduce punctuation casually.
2. **Statement lines terminate on newline** (`Tok::Newline`, `parser.rs:84-104`). Bytecode/text formats emitted by BOLI should be line-oriented or use a new explicitly designed string escape layer.
3. **Context-sensitive keywords.** `kaam` and `hissa` have special contextual meanings (`parser.rs:330-398,570-600`). A BOLI lexer written in BOLI must preserve these heuristics exactly.
4. **No error recovery.** Parser returns first error (`parser.rs:157-166`). Compiler diagnostics will need either a new multi-error convention or accept single-error behavior initially.
5. **Runtime errors lose columns.** AST stores only line, so `BoliError::line` is used at runtime. For self-hosting compiler diagnostics, spans must be added to the BOLI-side AST even if Rust AST stays unchanged.
6. **State writes are local.** `Environment::set` always writes innermost scope (`mod.rs:232-252`). Compiler written in BOLI must return updated state maps from each phase; global mutable compiler state is not available.
7. **Module imports share environment.** `laao` executes imported code in current environment (`mod.rs:170-206`), so module isolation must be designed if BOLI compiler is split across files.
8. **Limits are strict.** Defaults include 100k steps, 64 call depth, 1 MiB strings, 10 MiB source, 10000 list items, 64 parse depth, 64 MiB total bytes (`limits.rs:4-17`). A self-hosted compiler will hit these quickly; per-build limit overrides must be explicit and documented.
9. **Release matrix excludes ARM Linux/Android.** Current targets: Linux x86-64, macOS x86-64, macOS aarch64, Windows x86-64 (`release.yml:12-25`).
10. **VM is experimental but already substantial.** It has 23 opcodes (`vm.rs:16-41`), compile-time import handling, runtime imports, scopes, trace, limits, and 39 dedicated tests.

---

## 2. Phase 2 — Missing language capabilities required for self-hosting

### 2.1 Capability gaps with security constraints

#### 2.1.1 Controlled file read

**Needed for:** compiler source loading, include graph, test fixtures.

**Proposed capability (design only):**
- New builtins: `padho "<path>"`, `padho_bytes "<path>"`.
- Default **off**. Enable only with `BOLI_ALLOW_FS=1`.
- Root confinement with `BOLI_FS_ROOT` (same spirit as existing `BOLI_IMPORT_ROOT`, `limits.rs:159-185`).
- Canonicalize path and reject `..`, absolute escapes, symlink escapes.
- Cap bytes with `BOLI_MAX_FILE_BYTES`; reject non-UTF-8 for text reads.
- Return structured error map `{ok: galat, reason: "...", line, col}` instead of panic.

**Tests:** path escape, symlink escape, oversized file, missing file, non-UTF-8, cap boundary, root unset => error.

#### 2.1.2 Controlled file write

**Needed for:** emitting `.bc` bytecode, generated tests, build artifacts.

**Proposed capability:**
- `likho_file "<path>" <text>`, `likho_bytes "<path>" <bytes>`.
- Same opt-in/root/caps as read.
- Atomic write: temp file in same directory + rename.
- Allow-list extensions by default: `.bc`, `.txt`, `.json`? Since language avoids symbols, a custom `.bc` binary is fine.
- No append mode initially; full replace only.
- Refuse writing over source files unless explicit `BOLI_ALLOW_SOURCE_WRITE=1` — better: never allow by default.

**Tests:** atomicity on failure, no partial bytecode, path escape, overwrite refusal, permissions error.

#### 2.1.3 String manipulation

Current: `upar`, `neeche`, `text`, `sankhya`, `lambayi`, `hissa` char access. Missing:
- `hissa_text` substring/slice.
- `byte_lambayi` UTF-8 byte length (general builtin; the Phase 3.1 lexer uses a local `lexer_byte_length` helper instead).
- `utf8_bytes` / `utf8_text`.
- `milao` already via `jodo`, but need bounded join/split.
- `shuru`, `ant`, `baad`, `pehle`, `badlo`.
- Codepoint vs byte distinction must be explicit because `lambayi` is Unicode-char count (`builtins.rs:121-136`) while Rust lexer caps bytes. Phase 3.1 added a lexer-local `lexer_byte_length`; a general builtin is still missing.

**Security:** every result bounded by `BOLI_MAX_STRING_BYTES`; no regex/backtracking blowup; simple linear algorithms.

#### 2.1.4 List and map manipulation

Current enough for basic compiler data structures: list literal, `hissa` index/assign, `jodo` concat, map literal/index/assign, `chaabiyan`.

Add:
- `jodon` push, `hatao` remove, `ultao` reverse, `nikal` slice.
- `map_ha` key exists, `mitao` delete key.
- Stable iteration via `chaabiyan` already sorted because `BTreeMap`.

**Security:** caps for list length, key bytes, total bytes; no cyclic structures because values are cloned/replaced.

#### 2.1.5 Byte operations and binary output

**Needed for:** bytecode files. Missing entirely.

Add:
- `byte` value type or `bytes` list constrained to 0..255.
- `byte_banao <number>`, `number_banao <byte>`.
- `pack_little`, `unpack_little`, `pack_big`, `unpack_big` for u8/u16/u32/u64/f64.
- `likho_bytes` to write bytes to file/stdout.
- No pointer arithmetic, no raw memory, no FFI.

**Security:** explicit bounds checks; no reinterpretation of arbitrary memory; all lengths capped.

#### 2.1.6 Error handling and diagnostics

Current errors are host `BoliError`; language cannot construct/catch them.

Design:
- First step: convention, not syntax — parser/compiler returns `{ok, node, error}` maps. `error` = `{kind, msg, line, col, start, end, path}`.
- Second step: builtin `galti_banao`, `galti_hai`, `galti_msg`, `galti_line`, `galti_col`.
- Later: `pakdo` block if needed. Avoid introducing it in first self-hosting pass unless convention proves insufficient.
- Multi-error collection: `errors` list in compiler state; sort by `(path, line, col)`.

#### 2.1.7 Source locations

Must add to BOLI-side AST/tokens:
- `line`, `col`, `start_byte`, `end_byte`, `path`.
- Rust lexer currently discards byte indices (`lexer.rs:177-179`), so Rust side may need a compatible span field only if we want exact differential spans. Minimal path: BOLI lexer computes spans; Rust parser corpus comparison normalizes spans away initially.
- Opcode debug table carries line/span index, like current VM line side-table (`vm.rs:3,648-651`).

#### 2.1.8 Testing facilities inside BOLI

Needed:
- `test` declaration or a simple convention: `.hl` files under `tests/bolic/` exporting `test_*` kaams returning `sahi`/`galat`.
- Runner `boli test <dir>` implemented initially in Rust, later in BOLI.
- Golden files for token streams, AST dumps, bytecode hex dumps, error messages.
- Deterministic seed for fuzz-like tests.

#### 2.1.9 Build system

Needed:
- Manifest read by BOLI tools: `boli-build.hl` or `boli.toml` equivalent in Hinglish syntax.
- Fields: name, version, entry, module roots, target format, output path, required limits, compiler seed hash.
- `boli build` orchestrates lex→parse→compile→emit.
- Deterministic ordering: sorted files, sorted map keys (already `BTreeMap`), no timestamps in bytecode unless explicitly disabled.

### 2.2 Feature decision table

| Feature | Add before Stage 1? | Reason |
|---|---|---|
| Structured error maps / `{ok,error}` convention | Yes | Compiler cannot propagate diagnostics reliably otherwise. |
| Source spans in BOLI AST | Yes | Needed for compiler diagnostics and bytecode debug table. |
| `puran` integer type | Yes | f64 offsets are dangerous; existing roadmap already lists `puran` (`UPGRADE_ROADMAP.md:25-30`). |
| File read/write opt-in | Yes | Compiler must load sources and emit bytecode. |
| Bytes + binary output | Yes | Bytecode is binary. |
| More string/list/map helpers | Yes | Lexer/parser implementation needs slicing/search. |
| Closures / first-class functions | No | Explicit state threading is enough and simpler. |
| Exceptions `try/pakdo` | No initially | Return-convention sufficient; add only if needed. |
| Module namespace/export | No initially | `laao` shared-scope model can work with disciplined prefixes. |
| Native codegen | No | Long-term Milestone 12. |

---

## 3. Phase 3 — Architecture design

### 3.1 Target pipeline

```
BOLI Source (.hl)
  → BOLI Lexer        → Token stream + spans
  → BOLI Parser       → AST + spans + parse errors
  → BOLI AST helpers  → tagged maps, visitors, dump/compare
  → BOLI Compiler     → symbol table, constant pool, code emission
  → Bytecode (.bc)    → versioned binary module
  → BOLI VM/runtime   → execution
```

Default interpreter and current Rust VM remain available during migration. New pipeline is additive.

### 3.2 Exact compiler architecture (BOLI implementation)

Proposed new files (plan only; not created in this task):

```
boli/
  main.hl                 # CLI: build / run / test / dump-bc
  build.hl                # manifest reader + orchestration
  compiler/
    errors.hl             # error constructors, renderer, sorting
    span.hl               # span helpers
    lexer.hl              # BOLI lexer
    parser.hl             # BOLI parser
    ast.hl                # AST constructors/visitors/dump
    resolve.hl            # kaam/import resolution, duplicate checks
    symbols.hl            # name tables, scopes for compiler symbols
    codegen.hl            # emit opcodes/operands/line table
    bytecode.hl           # serialize/deserialize .bc
    disasm.hl             # bytecode disassembler for tests
  stdlib/
    text.hl               # bounded string helpers
    list.hl               # bounded list helpers
    map.hl                # bounded map helpers
    bytes.hl              # byte/pack/unpack helpers
    fs.hl                 # sandboxed file wrappers
    diag.hl               # Hinglish diagnostic renderer
tests/
  bolic_lexer.hl
  bolic_parser.hl
  bolic_codegen.hl
  selfhost.hl
  sandbox_fs.hl
bootstrap/
  stage0.rust.md          # reference-only notes
  seed.sha256             # frozen Stage 2 bytecode hash
```

#### 3.2.1 Internal value representation

Because BOLI has no enums:
- Token = map with `"kind"` text plus fields.
  Example conceptual shape: `{kind: "Number", value: 5, line: 1, col: 7, start: 6, end: 7}`.
- AST node = map with `"kind"` and child nodes/field lists.
- Error = map `{kind: "error", msg, line, col, start, end, path}`.
- Compile result = map `{ok, node, errors, state}`.
- Compiler state = explicitly passed map containing `tokens`, `pos`, `symbols`, `constants`, `names`, `code`, `lines`, `imports`, `errors`, `limits`, `path`.

**Reason for explicit state:** BOLI functions cannot capture outer state and `set()` writes only to current scope (`mod.rs:232-252`). Returning state is safer than adding global mutation early.

#### 3.2.2 Bytecode format v1 (design)

Keep it small, deterministic, and easy to emit from BOLI.

```
Header
  magic: 4 bytes  = "BOLI"
  version: u16 LE = 1
  flags: u16 LE   = 0
  source_hash: 8 bytes (deterministic hash of canonical source set)

Constant pool
  count: u32 LE
  entries:
    tag u8:
      1 = khaali
      2 = bool (u8 0/1)
      3 = f64 (8 bytes LE)
      4 = puran/i64 (8 bytes LE)
      5 = text (u32 len + UTF-8 bytes)
      6 = bytes (u32 len + raw bytes)
      7 = list (u32 count + u32 indexes)
      8 = map (u32 count + repeated: u32 key index, u32 value index)

Name table
  count: u32 LE
  names: repeated u32 len + UTF-8 bytes

Function table
  count: u32 LE
  each: name_idx u32, param_count u16, code_offset u32, code_len u32,
        line_table_offset u32, line_table_len u32

Import table
  count: u32 LE
  each: canonical_path_idx u32, chunk_offset u32, chunk_len u32

Code section
  count: u32 LE
  each opcode:
    u8 opcode
    operand bytes depending on opcode (u16/u32 indexes; u32 jump targets)

Line/span table
  count: u32 LE
  each: start_pc u32, line u32, col u32, start_byte u32, end_byte u32

Checksum footer
  sha256 of all preceding bytes (32 bytes)
```

Notes:
- Use little-endian fixed-width integers; no f64 for jump offsets.
- Reuse current opcode semantics from `src/vm.rs:16-41` but freeze as `BOLI-BC v1`.
- Add `LoadConstIdx`, `LoadPuran`, `Pack`, etc. only when Milestone 2/3 primitives are stable.
- Include source hash and checksum for reproducibility, not for security sandboxing.

#### 3.2.3 Compiler phases inside `compile.hl`

1. `lex(source, path, limits) -> {tokens, errors, state}`
   Must match Rust lexer token stream on golden corpus. Phase 3.1 proved byte caps are possible with a lexer-local `lexer_byte_length`; a general `byte_lambayi` builtin is still preferred for the final compiler.
2. `parse(tokens, path, limits) -> {ast, errors, state}`
   Recursive descent with explicit `pos` returned; no global mutable pos.
3. `resolve(ast, imports, limits) -> {ast, symbols, errors, state}`
   Duplicate kaam check, import graph, source-dir handling, cycle dedup.
4. `codegen(ast, symbols, limits) -> {code, constants, names, lines, imports, errors, state}`
   Deterministic emission; sorted symbol order; no timestamp.
5. `serialize(...) -> bytes`
   Writes `.bc` through sandboxed FS.
6. `diagnose(errors) -> text`
   Hinglish diagnostics with line/col/caret, path, trace.

### 3.3 Bootstrap architecture — exact process

#### Stage 0 — Rust reference (exists today)

- Use current Rust `boli` binary and Rust VM (`src/vm.rs`) as oracle.
- Add capability to emit/load `.bc` v1 (Milestone 4). This is a Rust-side extension, not a rewrite.
- Existing gate (215 passed, 2 ignored) remains the compatibility baseline.

Conceptual command:

```
boli build --emit-bc bootstrap/out/bolic-stage0.bc --entry boli/main.hl
boli run-bc bootstrap/out/bolic-stage0.bc --emit-bc bootstrap/out/app.bc
```

#### Stage 1 — BOLI compiler written in BOLI

- Implement `boli/**/*.hl` using Stage 0 runtime.
- Stage 1 compiler compiles its own source to `bolic-stage1.bc`.
- Compare behavior against Rust Stage 0 compiler on token/AST/bytecode golden corpus.

#### Stage 2 — self-compile

- Run `bolic-stage1.bc` on Stage 0 Rust VM.
- It compiles the same `boli/**/*.hl` source to `bolic-stage2.bc`.
- Acceptance: `sha256(bolic-stage1.bc) == sha256(bolic-stage2.bc)`.
- If not equal, dump/disassemble and fix determinism before moving on.

#### Stage 3 — BOLI VM/runtime in BOLI, non-Rust host decision

Important honest point: **bytecode alone cannot run without a VM host.** To make final release Rust-independent, one of these must be chosen:

| Option | Pros | Cons |
|---|---|---|
| A. Tiny C/assembly launcher for `BOLI-BC v1` | Simple, auditable, no Rust in release. | Adds C/assembly toolchain; not zero-toolchain. |
| B. BOLI VM compiled to native by Stage 2 BOLI codegen | Aligns with long-term goal. | Depends on Milestone 12; highest risk. |
| C. Checked-in frozen bootstrap binary | Fastest short-term. | Reproducibility/trust problem; avoid as final answer. |

Recommended: **A as bridge, B as final direction.** Rust stays in repo as reference/bootstrap, but release pipeline no longer invokes it once A or B is green.

#### Stage 4 — reproducible Rust-free build

Inputs:
- BOLI compiler source `boli/**/*.hl`
- Frozen seed bytecode or tiny launcher source
- Manifest and limit config

Outputs:
- `bolic.bc` or native `bolic`
- deterministic `.bc` artifacts with checksums
- release archives

Acceptance: two clean checkouts produce identical hashes.

---

## 4. Phase 4 — Migration roadmap

Effort legend: `S` small, `M` medium, `L` large, `XL` multi-step research.

### Phase 4A — Parser design specification (complete, design only)

**Status:** complete.
**Required docs:** `docs/PARSER_GRAMMAR.md`, `docs/PARSER_AST_SPEC.md`, `docs/PARSER_ARCHITECTURE.md`, `docs/PARSER_ERROR_RECOVERY.md`, `docs/PHASE4A_REPORT.md`.
**Method:** inspected `src/parser.rs`, `src/ast.rs`, `src/lexer.rs`, interpreter, VM, builtins, limits, tests, language docs, Phase 3.2 report, token contract, and parser interface spec.

Design decisions:

- grammar source of truth is the implemented Rust parser, not an invented conventional grammar;
- future BOLI parser keeps state-threaded recursive descent with two-token lookahead;
- arithmetic remains explicit nested prefix Hinglish, not symbol infix precedence;
- call arguments remain flat atoms; `f a b` is `f(a, b)`;
- `kaam` and `hissa` keep their contextual disambiguation;
- AST compatibility is tagged maps with exact current node/field names;
- Rust AST stays line-only, so golden AST dumps need legacy and strict modes;
- default errors stay fail-fast with one error; structured kinds and multi-error recovery are later opt-ins;
- no parser, AST, lexer, VM, CLI, or syntax change was made.

**Decision:** Phase 4B implementation is **READY WITH BLOCKERS**. Blockers are runtime/step/call/value-depth limits, absence of a BOLI AST representation, absence of a parser differential corpus, and the standing rule that VM stabilization precedes compiler migration.

### Milestone 1 — VM stabilization

**Effort:** L
**Required files:** `tests/vm.rs`, new `tests/vm_corpus.rs`, new `tests/vm_fuzz.rs`, `docs/VM_STABILITY.md`, `src/vm.rs` only if fixes needed.
**Required features:**
- Freeze `BOLI-BC v1` opcode semantics on paper.
- Add bytecode serialize/load API to Rust VM (still experimental label until green).
- VM differential corpus: interpreter vs VM outputs and errors.
- VM fuzz corpus: lex/parse/compile/run with deterministic seeds.
- Benchmark harness without external deps: time loops, maps, lists, imports.
- Remove/fix internal `.expect("stack underflow")` paths that user input can reach if any are found.

**Dependencies:** existing 39 VM tests, 191 suite.
**Risks:** semantic drift; step counts differ by backend; deadline pressure to call VM stable too early.
**Tests:** all 191 existing + new differential/fuzz/benchmark smoke.
**Acceptance criteria:**
- 191/191 green on 3 OS.
- VM corpus has zero mismatches against interpreter for agreed semantics.
- Fuzz corpus no panic/hang/OOM for 100k deterministic cases.
- Opcode v1 document approved; VM marked stable in docs.
- Tree-walker remains default.

**Rollback strategy:** keep all VM changes behind `BOLI_VM`; revert VM-only commits; interpreter stays authoritative.

---

### Milestone 2 — Runtime improvements

**Effort:** M
**Required files:** `src/interpreter/value.rs`, `src/builtins.rs`, `src/limits.rs`, `tests/boli.rs`, `LANGUAGE_SPEC.md` addendum.
**Required features:**
- Add `puran` integer or at least checked integer helpers for bytecode offsets.
- Add `byte_lambayi`, `utf8_bytes`, `utf8_text`, `slice_text`, `has_key`, `delete_key`, `push`, `pop`.
- Add structured error helper maps via builtins (no syntax change yet).
- Add span type to BOLI-side data model plan; Rust AST can remain line-only for compatibility.

**Dependencies:** M1.
**Risks:** new semantics break old programs; f64 vs integer mismatch; Unicode byte/char confusion.
**Tests:** boundary tests for every new builtin; old 191 regression; Unicode byte-vs-char tests.
**Acceptance criteria:** new builtins bounded by existing limits; no existing program output changes; docs updated in Hinglish.
**Rollback strategy:** feature-gate new builtins via env flag initially; remove calls from new compiler code if regression appears.

---

### Milestone 3 — File I/O capabilities

**Effort:** L
**Required files:** new `src/fs.rs`, `src/builtins.rs`, `src/limits.rs`, new `tests/cli_fs.rs`, new `docs/FS_SECURITY.md`.
**Required features:**
- `BOLI_ALLOW_FS`, `BOLI_FS_ROOT`, `BOLI_MAX_FILE_BYTES`, `BOLI_MAX_OPEN_FILES`, extension allow-list.
- `padho`, `padho_bytes`, `likho_file`, `likho_bytes`.
- Canonicalization, symlink refusal, atomic write, no source overwrite by default.
- All failures as structured Hinglish errors.

**Dependencies:** M2.
**Risks:** sandbox escape, TOCTOU, partial writes, accidental overwrite of `.hl` sources.
**Tests:** path escape, symlink, size cap, missing root, atomicity, extension denial, both engines.
**Acceptance criteria:** FS disabled by default; enabled mode passes adversarial suite; existing 191 green.
**Rollback strategy:** leave `BOLI_ALLOW_FS=0`; remove FS builtins from registry; no source files touched.

---

### Milestone 4 — Compiler infrastructure

**Effort:** L
**Required files:** new `docs/BYTECODE_FORMAT.md`, new `tests/bytecode_roundtrip.rs`, Rust-side serializer/loader, new `boli/**` skeleton.
**Required features:**
- `.bc` v1 format implemented in Rust Stage 0.
- Constant pool, name table, function table, import table, line/span table, checksum footer.
- Rust compiler can emit `.bc`; Rust VM can load and run `.bc`.
- Disassembler for debugging.

**Dependencies:** M1-M3.
**Risks:** format churn; over-complex format; nondeterministic ordering.
**Tests:** roundtrip identity, golden hex dumps, corrupted checksum rejection, import chunks.
**Acceptance criteria:** same AST → same `.bc` bytes; VM runs `.bc` with identical output/error corpus.
**Rollback strategy:** AST-based VM path remains default; `.bc` loader additive.

---

### Milestone 5 — BOLI lexer

**Effort:** M
**Required files:** `boli/compiler/lexer.hl`, `tests/bolic_lexer.hl`, token golden files.
**Required features:** token kinds, keywords, Unicode identifiers, strings/escapes, comments, numbers, spans, limits.
**Dependencies:** M4.
**Risks:** byte vs Unicode char differences; contextual `kaam`/`hissa`; exact error message drift.
**Tests:** token stream diff vs Rust lexer for full corpus including error cases.
**Acceptance criteria:** normalized token streams and locations match Rust lexer for golden corpus; 191 Rust tests still green.
**Rollback strategy:** BOLI lexer not wired into default build until M8.

---

### Milestone 6 — BOLI parser

**Effort:** L
**Required files:** `boli/compiler/parser.hl`, `tests/bolic_parser.hl`, AST dump golden files.
**Required features:** recursive descent, 2-lookahead, all statements/expressions/conditions, structured `{ok,error}` returns.
**Dependencies:** M5, provisional AST map shape from M4.
**Risks:** contextual ambiguity; no backtracking; error recovery differences.
**Tests:** parse corpus vs Rust parser normalized AST; error position tests.
**Acceptance criteria:** normalized AST and error messages match for corpus; no panic/hang on fuzz input.
**Rollback strategy:** keep Rust parser authoritative; BOLI parser output compared only in tests.

---

### Milestone 7 — BOLI AST

**Effort:** M
**Required files:** `boli/compiler/ast.hl`, `boli/compiler/span.hl`, `boli/compiler/diag.hl`, `tests/bolic_ast.hl`.
**Required features:** AST constructors, visitors, pretty dump, span propagation, error renderer.
**Dependencies:** M4-M6.
**Risks:** map-based tagged nodes become verbose; accidental field-name drift.
**Tests:** AST dump golden files, span preservation, Hinglish diagnostic rendering.
**Acceptance criteria:** every AST node has kind+span; dump stable across runs; parser output passes AST validators.
**Rollback strategy:** keep Rust `ast.rs` unchanged; BOLI AST helpers are isolated library files.

---

### Milestone 8 — BOLI bytecode compiler

**Effort:** XL
**Required files:** `boli/compiler/resolve.hl`, `symbols.hl`, `codegen.hl`, `bytecode.hl`, `disasm.hl`, `tests/bolic_codegen.hl`.
**Required features:** resolution pass, symbol tables, imports, function chunks, constant interning, code emission, span table, `.bc` serialization.
**Dependencies:** M1-M7.
**Risks:** output mismatch with Rust compiler; nondeterminism; limits exceeded on compiler source itself.
**Tests:** BOLI compiler output vs Rust compiler output byte-for-byte for deterministic subset; semantic equivalence for full corpus.
**Acceptance criteria:** BOLI compiler can compile examples and its own stdlib to loadable `.bc`; 191 Rust tests + BOLI tests green.
**Rollback strategy:** Rust compiler remains release compiler; BOLI compiler output validated in shadow mode.

---

### Milestone 9 — Self-hosting

**Effort:** L
**Required files:** `bootstrap/*`, `tests/selfhost.hl`, `docs/SELF_HOSTING.md`.
**Required features:** Stage 1 and Stage 2 flow from section 3.3; compiler compiles itself.
**Dependencies:** M8.
**Risks:** chicken-and-egg bugs; output nondeterminism; resource limits too low.
**Tests:** `stage1.bc` vs `stage2.bc` hash equality; run full corpus through self-compiled compiler.
**Acceptance criteria:** byte-identical self-compile; self-compiled compiler passes all compiler tests; no Rust source changes required to fix compiler bugs.
**Rollback strategy:** keep Stage 0 Rust compiler in release; mark self-hosting experimental until M10.

---

### Milestone 10 — Bootstrap reproducibility

**Effort:** M
**Required files:** `scripts/bootstrap.sh`, `docs/BOOTSTRAP.md`, `bootstrap/seed.sha256`, CI job.
**Required features:** hermetic env, pinned limits, sorted inputs, stable hashes, two-checkout verification.
**Dependencies:** M9.
**Risks:** path/timestamp leakage; platform line endings; unordered maps (currently `BTreeMap` helps).
**Tests:** build in two clean directories; compare all artifact hashes; reject seed mismatch.
**Acceptance criteria:** identical hashes on same target across clean checkouts; seed documented and reviewed.
**Rollback strategy:** pin previous seed hash; keep deterministic test corpus unchanged.

---

### Milestone 11 — Rust removal from release pipeline

**Effort:** L
**Required files:** `.github/workflows/release.yml`, `.github/workflows/ci.yml`, `build.boli`, `install.sh`, `docs/RUST_BOOTSTRAP.md`.
**Required features:** release builds without `cargo`/`rustc`; Rust job optional reference only; non-Rust host from Stage 3 option A/B.
**Dependencies:** M10.
**Risks:** target support gaps; native launcher review; installer path mismatch.
**Tests:** release workflow log contains no cargo; artifact smoke on Linux/macOS/Windows; checksums verify; installer test with generated artifact.
**Acceptance criteria:** tagged release produced from BOLI sources + seed/launcher; Rust source preserved but not required; all existing tests still available via reference job.
**Rollback strategy:** revert workflow files only; Rust source and tests remain intact.

---

### Milestone 12 — Native code generation

**Effort:** XL
**Required files:** `boli/codegen/x86_64.hl`, `boli/codegen/arm64.hl`, `boli/codegen/abi.hl`, `tests/codegen/*`.
**Required features:** lower `.bc` to native machine code for x86-64 and ARM64; calling convention; value representation; small runtime support; linker/launcher integration.
**Dependencies:** M11, Stage 3 option B preferred.
**Risks:** ABI bugs, memory safety, platform-specific codegen, no GC yet, SecurityReport scope expansion.
**Tests:** cross-target suite, qemu ARM64 smoke, differential vs VM, native artifact checksums.
**Acceptance criteria:** native binaries pass same corpus as VM; bytecode VM remains fallback/default for safety.
**Rollback strategy:** keep bytecode releases; disable native codegen flag; no deletion of VM.

---

## 5. Testing strategy

### 5.1 Preserve existing suite

- Keep all existing tests; current gate is 215 passed, 2 ignored.
  - `tests/boli.rs`, `tests/cli.rs`, `tests/vm.rs`, embedded unit tests
  - `tests/phase3_lexer.rs` (11 tests after Phase 3.2)
- No test deletion. New behavior adds new tests; no test rewritten to hide regressions.
- CI continues to run Rust reference suite even after Rust is removed from release path.

### 5.2 New test layers

| Layer | Purpose | Tooling |
|---|---|---|
| Lexer differential | BOLI lexer vs Rust lexer token streams | golden JSON-like text dumps |
| Parser differential | BOLI parser vs Rust parser normalized AST | AST dump files |
| Bytecode golden | `.bc` deterministic bytes | hex dumps + sha256 |
| VM differential | interpreter vs Rust VM vs BOLI VM outputs/errors | shared corpus |
| Self-host reproducibility | Stage1 vs Stage2 bytecode identical | sha256 compare |
| Sandbox FS | path/root/symlink/cap attacks | adversarial fixtures |
| Fuzz | no panic/hang across lex/parse/compile/run | deterministic xorshift seeds |
| Benchmark | detect gross regressions, not gate correctness | time-only reports |
| Release smoke | artifacts downloadable and runnable | `sha256sum -c` + `--version` |

### 5.3 Golden corpus rules

- Keep fixtures small and line-oriented.
- Include positive and negative cases.
- Include Unicode identifiers and strings.
- Include all limit boundary cases.
- Include import graphs, cycles, missing files.
- Include VM parity cases already in `tests/vm.rs:281-475`.
- Normalize spans in early parser comparison; require exact spans by M7.

### 5.4 Gate policy

Every milestone must pass:
1. `cargo fmt --all -- --check`
2. `cargo check --all-targets --locked`
3. `cargo clippy --all-targets --locked -- -D warnings`
4. `cargo test --all-targets --locked --quiet`
5. `cargo build --release --locked`
6. `cargo run --release --locked --quiet -- version`
7. New milestone-specific tests
8. `git diff --check`

After M11, add equivalent BOLI pipeline gates and keep Rust gates as reference-only.

---

## 6. Risk register

| Risk | Severity | Trigger | Mitigation | Owner milestone |
|---|---:|---|---|---|
| VM called stable too early | High | Skipping differential/fuzz | Hard gate in M1; tree-walker stays default | M1 |
| f64 used for bytecode offsets | High | Reusing `Number` everywhere | Add `puran`/i64 in M2 | M2, M4 |
| Unicode byte/char confusion | High | lexer caps bytes, `lambayi` chars | Lexer-local `lexer_byte_length` proven in Phase 3.1; add general `byte_lambayi` and explicit conversion | M2, M5 |
| Sandbox escape via FS | Critical | `..`/symlink/absolute path | canonicalize + root + extension allow-list | M3 |
| Partial bytecode on write failure | High | crash during emit | atomic temp+rename | M3, M4 |
| Nondeterministic bytecode | High | timestamps, hash maps, path strings | sorted inputs, fixed hash, no timestamp | M4, M10 |
| Missing structured errors | Medium | compiler returns strings only | `{ok,error}` convention from M2 | M2-M8 |
| Contextual keyword drift | Medium | `kaam`/`hissa` heuristics | token/parse differential corpus | M5, M6 |
| Resource limits hit by compiler itself | Medium | 100k steps / 64 depth too low | explicit build limits, documented caps | M4-M9 |
| Rust removal before reproducible self-host | High | pipeline switched early | M10 acceptance required | M11 |
| Native codegen security | Critical | raw machine code | keep bytecode default; fuzz + sandbox | M12 |
| Docs drift | Medium | old v0.6.5 claims | Hinglish addenda per milestone | all |

---

## 7. Rust dependency removal — safe sequence

**Do not delete Rust source.** Keep `src/*.rs`, `tests/*.rs`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` as `reference/bootstrap` forever or until a separate explicit decision.

Removal means:
1. Remove Rust from **required build path**, not from repository.
2. Add BOLI compiler source under `boli/`.
3. Add BOLI VM/runtime or non-Rust launcher.
4. Prove reproducibility (M10).
5. Flip release workflow (M11) so `cargo` is absent from build steps.
6. Keep optional Rust reference CI job running the full current suite (214 passed, 2 ignored at Phase 3.1).
7. Update installer to consume new artifact path or keep `boli` binary name.

Existing user rules honored:
- Existing code delete nahi.
- Existing features break nahi.
- Unnecessary rewrite nahi.
- Rust initial bootstrap/reference ke roop me preserve.
- Existing tests preserve.
- Har step par tests.
- VM stable hue bina compiler migration nahi.
- Ek hi phase me poora rewrite nahi.
- Hinglish docs maintain.

---

## 8. Current decision

**Abhi koi implementation nahi.** Ye report sirf architecture audit aur roadmap hai.

Recommended next action (when implementation is approved): **Milestone 1 — VM stabilization**. Isme bhi pehle Step 1 hoga: VM differential corpus aur bytecode format spec banana; BOLI compiler code tab nahi likhna jab tak VM stability gate green na ho.

---

## 9. Key source references used

- `Cargo.toml:1-13`, `Cargo.lock:1-7`, `rust-toolchain.toml:1-3`
- `src/lexer.rs:8-486`
- `src/parser.rs:14-835`
- `src/ast.rs:8-258`
- `src/scope.rs:8-28`
- `src/limits.rs:4-247`
- `src/interpreter/mod.rs:20-687`, `src/interpreter/eval.rs`, `src/interpreter/value.rs`
- `src/builtins.rs:8-191`
- `src/vm.rs:1-1270`
- `src/main.rs:23-461`
- `src/lib.rs:3-10`
- `tests/boli.rs`, `tests/cli.rs`, `tests/vm.rs`
- `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.github/dependabot.yml`
- `install.sh:1-57`
- `README.md`, `ARCHITECTURE.md`, `LANGUAGE_SPEC.md`, `TESTING.md`, `CHANGELOG.md`
- `docs/UPGRADE_ROADMAP.md`, `docs/SECURITY_REPORT.md`, `docs/COMPLETE_AUDIT.md`, `docs/TEST_RESULTS.md`
