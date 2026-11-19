# BOLI Standard Library (Core)

## Status

Core standard library abhi 37 built-in kaams par based hai. Ye sab Rust me implement hain aur interpreter/VM dono `src/builtins.rs` se call karte hain. Koi external dependency nahi hai.

User-defined `kaam` same naam se builtin ko shadow karta hai. Builtin ko nested argument ke roop me call nahi kar sakte; pehle result variable me rakho.

## Existing builtins

| Kaam | Signature | Result |
| --- | --- | --- |
| `lambayi` | `lambayi(value)` | Text chars, list items, map keys, ya `khaali` ke liye `0`. |
| `upar` | `upar(text)` | Uppercase text. |
| `neeche` | `neeche(text)` | Lowercase text. |
| `text` | `text(value)` | Kisi bhi value ka display text. |
| `sankhya` | `sankhya(text)` | Text se number; galat text par error. |
| `chaabiyan` | `chaabiyan(map)` | Map ki sorted keys list. |

## Text builtins

Index 1 se shuru hote hain. `kato` me dono ends inclusive hain. Character positions Unicode chars par count hote hain, bytes par nahi.

| Kaam | Signature | Result |
| --- | --- | --- |
| `kato` | `kato(text, start, end)` | Inclusive character slice. |
| `khojo` | `khojo(haystack, needle)` | 1-based pehla position, na mile to `khaali`. |
| `hai_text` | `hai_text(haystack, needle)` | Substring mila to `sahi`. |
| `badlo` | `badlo(text, old, new)` | Saare `old` ko `new` se badalta hai. |
| `baton` | `baton(text, separator)` | Separator par text ke tukdon ki list. |
| `jodo_text` | `jodo_text(text_list, separator)` | Text list ko separator se jodta hai; non-text item par error. |
| `chhilo` | `chhilo(text)` | Leading/trailing whitespace hataata hai. |
| `akshar` | `akshar(text, index)` | 1-based character; text return karta hai. |
| `escape_text` | `escape_text(text)` | `\`, `"`, newline, carriage return, tab ko escape karta hai. |
| `unescape_text` | `unescape_text(text)` | `\n`, `\r`, `\t`, `\\`, `\"` ko actual character banata hai; unknown escape par error. |

Examples:

```boli
likho kato "namaste" 2 5        # amas
likho khojo "namaste boli" "boli" # 9
parts rakho baton "a,b,c" ","
likho jodo_text parts " | "     # a | b | c
likho akshar "नमस्ते" 3         # स
```

Lexer source string me sirf `\"`, `\\` aur `\n` samajhta hai. `\r` aur `\t` source me directly likhne ke liye `unescape_text "a\\rb"` ya `unescape_text "a\\tb"` use karo.

## List builtins

Sab pure hain: nayi list return karte hain, input mutate nahi hota. Index 1-based hai.

| Kaam | Signature | Result |
| --- | --- | --- |
| `jodega` | `jodega(list, value)` | End me value jodta hai. |
| `dalein` | `dalein(list, index, value)` | Index par value insert karta hai; `len+1` tak allowed. |
| `hatao` | `hatao(list, index)` | Index ka item hataata hai. |
| `ant` | `ant(list)` | Last item; empty list par `khaali`. |
| `ant_hatao` | `ant_hatao(list)` | Last item hataakar nayi list. |
| `tukda` | `tukda(list, start, end)` | Inclusive 1-based slice. |
| `nakal` | `nakal(list)` | Shallow copy. |
| `saaf` | `saaf(list)` | Empty list. |
| `ultana` | `ultana(list)` | Reversed copy. |
| `ek_hai` | `ek_hai(list, value)` | Value list me hai to `sahi`. |

Example:

```boli
cheezein rakho 1 2 3
likho jodega cheezein 4        # 1, 2, 3, 4
likho dalein cheezein 2 9      # 1, 9, 2, 3
likho hatao cheezein 2         # 1, 3
likho ultana cheezein          # 3, 2, 1
```

## Math builtins

Finite inputs required. Non-finite output par error.

| Kaam | Signature | Result |
| --- | --- | --- |
| `purnank` | `purnank(number)` | Floor. |
| `uparank` | `uparank(number)` | Ceil. |
| `nirapeksh` | `nirapeksh(number)` | Absolute value. |
| `mool` | `mool(number)` | Square root; negative input par error. |
| `shesh` | `shesh(a, b)` | Remainder; `b = 0` par error. |
| `sabse_chhota` | `sabse_chhota(a, b)` | Minimum. |
| `sabse_bada` | `sabse_bada(a, b)` | Maximum. |

## Type and test builtins

| Kaam | Signature | Result |
| --- | --- | --- |
| `prakaar` | `prakaar(value)` | `text`, `number`, `sahi/galat`, `list`, `map`, `khaali`. |
| `prakaar_hai` | `prakaar_hai(value, type_text)` | Type match par `sahi`. |
| `sahi_banao` | `sahi_banao(value)` | Truthiness: false for `khaali`, `galat`, empty text/list/map, `0`; otherwise true. |
| `saman` | `saman(a, b)` | Equal par `sahi`, unequal par error. |

## Limits

- `BOLI_MAX_STRING_BYTES`: text-producing builtins output cap.
- `BOLI_MAX_LIST_ITEMS`: list-producing builtins item cap.
- `BOLI_MAX_VALUE_DEPTH`: nested value cap; builtins naye deep structures nahi banate.
- `BOLI_MAX_STEPS` aur `BOLI_MAX_CALL_DEPTH`: program/call budget.

Limit cross hone par builtin Hinglish error deta hai, silently truncate nahi karta.

## Tests

`tests/stdlib.rs` positive, negative aur interpreter-vs-VM parity cases chalaata hai. Limit enforcement direct `call_builtin` calls se test hoti hai.
