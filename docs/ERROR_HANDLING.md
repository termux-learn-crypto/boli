# BOLI Error Handling (Phase 2)

## Current model

BOLI abhi fail-fast runtime use karta hai. Error panics nahi; `Result<T, BoliError>` se propagate hota hai. Program/user ke liye error `Galti: ...` se shuru hota hai.

`BoliError` fields:

| Field | Type | Purpose |
| --- | --- | --- |
| `line` | `Option<usize>` | Source line, jahan available ho. |
| `col` | `Option<usize>` | Lexer/parser column; runtime me mostly `None`. |
| `message` | `String` | Hinglish explanation. |
| `trace` | `Vec<(String, usize)>` | Nested `kaam` call frames, innermost pehle. |

Display format:

```text
Galti: line 2 me built-in kaam 'sankhya' galat text ke liye chal gaya
    kaam 'baat' ke andar (line 5)
```

`fmt::Display` trace ko reverse me print karta hai taaki outermost call pehle dikhe.

## Error phases

- **Lexer**: unterminated text, bad number, unknown character, excessive token/line bytes. Usually `line, col` milta hai.
- **Parser**: unexpected token, missing `khatam`, bad expression, depth/condition limits. Usually `line, col` milta hai.
- **Runtime**: unknown name, type mismatch, arity mismatch, index out of range, limit exceeded, import failures. `line` milta hai; column generally nahi.
- **Builtins**: sab naye builtins `BoliError::line(...)` return karte hain; arity/type/limit messages `STANDARD_LIBRARY.md` ke rules follow karte hain.

## Guarantees

- Error par koi partial stdout flush nahi hota; jo `likho` pehle chal chuke hain wo rehte hain.
- VM aur interpreter agreed cases me same user-facing error string dete hain (`tests/vm_corpus.rs`, `tests/stdlib.rs`).
- Resource limit errors normal runtime errors hain; panic/kill nahi.
- Stdlib errors input values mutate nahi karte.
- Unknown escape, negative square root, zero remainder, invalid 1-based index aur wrong type explicit error dete hain.

## Non-goals abhi ke liye

- `try`/`catch` ya error value nahi.
- Machine-readable `kind` field nahi.
- File path/context field nahi.
- Error chaining/source spans (`start..end`) nahi.
- OS error struct ko preserve karne wala typed I/O error nahi.

Ye missing capabilities self-hosting ke later milestones me add honge. Tabhi stable declare kiya jayega jab interpreter aur VM dono naye error shape par same behavior dein.

## Test strategy

- `tests/boli.rs` aur `tests/cli.rs`: lexer/parser/runtime error snapshots aur exit behavior.
- `tests/vm_corpus.rs`: interpreter-vs-VM error parity.
- `tests/stdlib.rs`: builtin negative cases aur limit errors.
- Naya error feature add karne par kam se kam ek positive aur ek negative test mandatory hai.

## Future compatibility plan

1. `BoliError` me optional `kind` aur `path` fields add karo, default existing messages preserve karte hue.
2. Typed categories: `syntax`, `type`, `arity`, `index`, `limit`, `io`, `import`, `assertion`.
3. `Galti` se structured form tak gradual migration; old display string compatibility rakho.
4. I/O errors ko capability-denied aur filesystem-error me separate karo (`FILE_IO_SECURITY.md`).
5. VM trace aur interpreter trace parity test karo.
