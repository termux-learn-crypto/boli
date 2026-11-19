# BOLI Runtime Spec (Phase 2)

## Scope

Yeh document runtime ke observable behavior ko lock karta hai: values, scope, evaluation order, builtins, errors aur safety limits. Isme File I/O nahi hai; uska design `FILE_IO_SECURITY.md` me hai aur implementation abhi nahi hua.

Interpreter default engine hai. VM (`--vm` ya `BOLI_VM=1`) same behavior produce karna hai. Koi bhi difference bug hai, feature nahi.

## Values

| BOLI type | Rust value | Notes |
| --- | --- | --- |
| `text` | `Value::Text(String)` | UTF-8; byte cap `BOLI_MAX_STRING_BYTES`; characters count Unicode chars. |
| `number` | `Value::Number(f64)` | Non-finite results error; display minimal Hinglish number text. |
| `sahi/galat` | `Value::Bool(bool)` | `sahi` / `galat`. |
| `khaali` | `Value::Khaali` | Empty/null value; length `0`. |
| `list` | `Value::List(Vec<Value>)` | Ordered, indexed 1..n; item cap `BOLI_MAX_LIST_ITEMS`. |
| `map` | `Value::Map(BTreeMap<String, Value>)` | Sorted string keys; key text cap applies. |

Values copy semantics rakhte hain. `cheezein hissa 2` se naya value milta hai; original list mutate nahi hoti. Pure mutation aur references abhi language me nahi hain. Nested containers `BOLI_MAX_VALUE_DEPTH` se limited hain.

## Evaluation order

- Expressions left-to-right evaluate hote hain.
- Function/builtin arguments bhi left-to-right evaluate hote hain.
- Call se pehle callee resolve hota hai; user `kaam` same naam ka builtin shadow karta hai.
- Builtin call nested argument ke roop me supported nahi hai: `likho jodo_text baton ...` parse nahi hota. Pehle intermediate variable banayein: `parts rakho baton ...`, phir `likho jodo_text parts ...`.
- Zero-arg user `kaam` auto-call hote hain; builtins ke liye explicit arguments required hain jab tak signature 0-arg na ho.

## Scope

- Assignment hamesha innermost scope me likhti hai.
- Har call nayi scope banata hai; parent scope read-only hai.
- Global `kaam` table alag hai; andar se kisi outer variable ko assign nahi kar sakte.
- Global variable shadowing allowed hai, lekin outer mutation nahi.
- Closures/function values nahi hain; `kaam` sirf statement-level definition hai.

## Builtins

Builtin registry `src/builtins.rs` me hai. Poori list `STANDARD_LIBRARY.md` me hai. Dispatch `call_builtin(name, args, line, max_string_bytes, max_list_items)` se hota hai. Interpreter aur VM dono yahi function use karte hain, isliye behavior same hai.

Har builtin:
- Exact arity check karta hai.
- Argument types validate karta hai.
- Output ko string/list/item caps me rakhta hai.
- Finite number outputs hi deta hai.
- Error par `BoliError::line(line, ...)` return karta hai.

## Errors

Runtime error par program fail-fast rukta hai; catch/try nahi hai. Error me `line`, `col`, `message`, `trace` hote hain. File path aur machine-readable error kind abhi `BoliError` me nahi hain; ye self-hosting ke liye future milestone hai.

CLI user-facing error `Galti: ...` format me print karta hai. Library tests error string compare karte hain. VM aur interpreter ka error text agreed cases me same hona chahiye.

## Limits

Defaults `src/limits.rs` me hain. `0` ya invalid env value default use karti hai.

| Env var | Default | Kya rokta hai |
| --- | --- | --- |
| `BOLI_MAX_STEPS` | 100000 | Total executed steps. |
| `BOLI_MAX_CALL_DEPTH` | 64 | `kaam` recursion depth. |
| `BOLI_MAX_STRING_BYTES` | 1048576 | Text/builtin output bytes. |
| `BOLI_MAX_SOURCE_BYTES` | 10485760 | Source file size. |
| `BOLI_MAX_LIST_ITEMS` | 10000 | List size. |
| `BOLI_MAX_PARSE_DEPTH` | 64 | Parser nesting. |
| `BOLI_MAX_CONDITION_DEPTH` | 128 | `aur`/`ya` chain length. |
| `BOLI_MAX_EVAL_DEPTH` | 256 | Expression eval nesting. |
| `BOLI_MAX_IMPORT_DEPTH` | 32 | `laao` import nesting. |
| `BOLI_MAX_TOKENS` | 1000000 | Program token count. |
| `BOLI_MAX_REPL_LINE_BYTES` | 65536 | REPL single line. |
| `BOLI_MAX_REPL_BUFFER_BYTES` | 1048576 | REPL buffer. |
| `BOLI_MAX_TOTAL_BYTES` | 67108864 | Live scope value bytes. |
| `BOLI_MAX_VALUE_DEPTH` | 64 | Nested list/map depth. |
| `BOLI_IMPORT_ROOT` | unset | Canonical import sandbox root. |

Phase 2 me koi limit remove nahi hua. Naye builtins unhi limits ko respect karte hain.

## Compatibility rules

- Same source → same stdout lines on interpreter and VM.
- Same source → same error string on agreed failure cases.
- Resource limits dono engines par same lagte hain.
- VM optimization line numbers aur Hinglish messages badal sakta hai to invalid hai.
- Nayi runtime capability tabhi add hogi jab docs + tests + limits saath aa jayein.
