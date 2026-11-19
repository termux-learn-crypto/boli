# BOLI-BC v1 — Bytecode Format Specification

Status: **frozen on paper, not implemented yet**.
Owner: Phase 1 Milestone 1.
Rule: is format implement hone se pehle VM ko stable mark nahi karna.

## 1. Purpose

BOLI-BC, BOLI source ko AST ke baad represent karne ka stable binary format hai. Iska use future BOLI compiler, disassembler aur bytecode loader karenge. Rust VM abhi AST se direct chalata hai; `.bc` loader abhi nahi hai.

Design goals:

- Deterministic bytes: same AST se hamesha same bytes.
- Small and decodable without external dependencies.
- Line information preserved for Hinglish errors.
- Jump targets textual offsets nahi, decoded instruction indices hain.
- Unknown opcodes audit reject ho jate hain.

## 2. Top-level layout

```text
header
section*
footer
```

All integers little-endian.

### Header — 16 bytes

| Offset | Size | Field | Value |
|---:|---:|---|---|
| 0 | 8 | magic | ASCII `BOLIBC01` |
| 8 | 2 | version | `1` |
| 10 | 2 | flags | `0` |
| 12 | 2 | section_count | number of sections |
| 14 | 2 | reserved | `0` |

### Section

| Size | Field | Meaning |
|---:|---|---|
| 1 | tag | section id |
| 1 | reserved | `0` |
| 4 | len | payload length in bytes |
| len | payload | section body |

Section order must be ascending by tag. Duplicate tags are invalid.

### Footer — 8 bytes

| Size | Field | Meaning |
|---:|---|---|
| 4 | total_len | total file length including footer |
| 4 | crc32 | CRC32 of all bytes before footer |

Loader must verify `total_len`, `crc32`, magic, version, section order and every index before execution.

## 3. Sections

### 0x01 CONST

```text
u32 count
repeat count:
  u8 kind
  u32 len
  bytes
```

`kind`:

| Value | Type | Encoding |
|---:|---|---|
| 0 | khaali | len = 0 |
| 1 | bool | len = 1, byte 0/1 |
| 2 | number | len = 8, f64 little-endian bits |
| 3 | string | UTF-8 bytes |

NaN payloads are canonicalized to one quiet NaN bit pattern before encoding.

### 0x02 NAMES

```text
u32 count
repeat count:
  u32 len
  UTF-8 bytes
```

Used by variables, kaam names and prompts.

### 0x03 FUNCS

```text
u32 count
repeat count:
  u32 name_idx
  u16 argc
  u32 entry
  u32 code_len
```

`entry` and `code_len` are instruction indices into the decoded CODE stream.

### 0x04 IMPORTS

```text
u32 count
repeat count:
  u32 len
  UTF-8 path bytes
```

Paths are kept relative. Absolute paths are rejected by loader policy.

### 0x05 LINES

```text
u32 count
repeat count:
  u32 instruction_index
  u32 line
  u32 col
```

A line entry applies from its instruction index until the next entry. Absent column is encoded as `0`.

### 0x06 CODE

Sequence of encoded instructions. Each instruction is one opcode byte followed by its operands. Decoder emits one VM `Op` per instruction, so jump targets are decoded instruction indices.

### 0x07 ERRORS

```text
u32 count
repeat count:
  u32 len
  UTF-8 bytes
```

Static Hinglish runtime error strings used by `FLOW_ERROR`.

## 4. Opcode table

| Code | Name | Operands | Runtime effect |
|---:|---|---|---|
| 0x01 | CONST | `u32 const_idx` | push constant |
| 0x02 | LOAD_NAME | `u32 name_idx` | push variable value |
| 0x03 | STORE_NAME | `u32 name_idx` | pop and store in current scope |
| 0x04 | OUT | none | pop and print |
| 0x05 | ARITH | `u8 op` | pop b, pop a, push `a op b` |
| 0x06 | CMP | `u8 op` | pop b, pop a, push bool |
| 0x07 | JUMP_IF_FALSE | `u32 target` | pop and jump when false |
| 0x08 | JUMP_IF_TRUE | `u32 target` | pop and jump when true |
| 0x09 | JUMP | `u32 target` | unconditional jump |
| 0x0A | POP_SCOPE | none | leave current scope |
| 0x0B | MAKE_LIST | `u32 count` | pop count values, push list |
| 0x0C | MAKE_MAP | `u32 pair_count` | pop 2*count values, push map |
| 0x0D | INDEX | none | pop index, pop target, push item |
| 0x0E | STORE_INDEX | none | pop value, index, target; mutate target |
| 0x0F | CALL | `u32 name_idx`, `u16 argc` | call kaam by name |
| 0x10 | DEFINE_KAAM | `u32 func_idx` | register kaam at runtime |
| 0x11 | IMPORT | `u32 import_idx` | run import once |
| 0x12 | RETURN | `u32 line` | return from kaam, line used in errors |
| 0x13 | FLOW_ERROR | `u32 error_idx` | controlled runtime error |
| 0x14 | POP_VALUE | none | discard stack top |
| 0x15 | PUCHO | `u32 name_idx`, `u32 prompt_idx` | read input into name |
| 0x16 | MAKE_RANGE | none | pop end, start; push range iterator |
| 0x17 | GINTI_NEXT | none | advance active range iterator |
| 0x18 | JUMP_IF_NO_LEFT | `u32 target` | jump when iterator exhausted |

`ARITH op`: `0` jodo, `1` ghatao, `2` guna, `3` bhaag.
`CMP op`: `0` bada, `1` chhota, `2` bada ya barabar, `3` chhota ya barabar, `4` barabar.

## 5. Validation rules

Loader must reject, with Hinglish error:

- unknown opcode or reserved operand value
- index outside CONST, NAMES, FUNCS, IMPORTS or ERRORS
- jump target outside CODE
- function entry outside CODE
- unterminated string or invalid UTF-8
- map pair count overflow
- total file length mismatch or checksum mismatch
- import path escaping the configured source root
- instruction count above `BOLI_MAX_STEPS`-compatible limit

## 6. Non-goals for v1

- No native code, no JIT, no architecture-specific bytes.
- No debug symbols beyond line/column table.
- No source maps for generated BOLI compiler files.
- No cross-version compatibility guarantee for versions after `1`.

Future versions must bump the version field and add a migration note here.
