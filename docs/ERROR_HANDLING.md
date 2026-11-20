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
| `kind` | `Option<&'static str>` | Typed lexer error category; other layers use `None`. |
| `span` | `Option<Box<Span>>` | Six-field source span for lexer errors; other layers use `None`. |

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
- Machine-readable `kind` field ab lexer errors ke liye hai; parser/runtime categories abhi typed nahi hain.
- File path/context field nahi.
- Source spans ab lexer errors/tokens ke liye hain; AST/runtime error chaining aur spans (`start..end`) nahi.
- OS error struct ko preserve karne wala typed I/O error nahi.

Ye missing capabilities self-hosting ke later milestones me add honge. Tabhi stable declare kiya jayega jab interpreter aur VM dono naye error shape par same behavior dein.

## Test strategy

- `tests/boli.rs` aur `tests/cli.rs`: lexer/parser/runtime error snapshots aur exit behavior.
- `tests/vm_corpus.rs`: interpreter-vs-VM error parity.
- `tests/stdlib.rs`: builtin negative cases aur limit errors.
- Naya error feature add karne par kam se kam ek positive aur ek negative test mandatory hai.

## Future compatibility plan

1. `BoliError` me `kind` add ho gaya hai; `path` field aur parser/runtime kind values abhi baaki hain. Purane default messages preserve karo.
2. Typed categories: `syntax`, `type`, `arity`, `index`, `limit`, `io`, `import`, `assertion`.
3. `Galti` se structured form tak gradual migration; old display string compatibility rakho.
4. I/O errors ko capability-denied aur filesystem-error me separate karo (`FILE_IO_SECURITY.md`).
5. VM trace aur interpreter trace parity test karo.
