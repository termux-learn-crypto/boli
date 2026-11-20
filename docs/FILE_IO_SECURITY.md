# BOLI File I/O Security Design (planned)

## Status

**Implementation status: not implemented.** File I/O abhi BOLI standard library me nahi hai. `src/builtins.rs` me `padho`, `likho_file`, etc. nahi hain. Yeh document future capability design hai taaki self-hosting ke liye safe boundary pehle decide ho jaye.

Reason: Phase 1 VM abhi experimental hai (`VM_STABILITY.md`), aur error model me typed `kind`/`path` fields nahi hain (`ERROR_HANDLING.md`). Inke bina File I/O add karna unsafe aur inconsistent hoga.

## Threat model

BOLI programs untrusted input ho sakte hain. File capability se ye risks aate hain:

- Absolute paths ya `..` se sandbox se bahar nikalna.
- Symlink race aur TOCTOU: check ek file par, open doosri par.
- Editor/source/config files overwrite karna.
- Secret files (SSH keys, tokens, credentials) read karna.
- Symlink follow karke target replace karna.
- Bahut bade file se memory exhaustion.
- Partial writes se corrupt output.
- Arbitrary extension write karna (executable/script drop).
- Import boundary (`BOLI_IMPORT_ROOT`) se alag I/O boundary banana.

## Capability model

File access opt-in hoga. Default deny.

| Env var | Meaning |
| --- | --- |
| `BOLI_ALLOW_FS` | `1` hona chahiye; default unset/0 matlab saare file builtins denied. |
| `BOLI_FS_ROOT` | Canonical sandbox directory. Iske bahar koi path nahi. |
| `BOLI_MAX_FILE_BYTES` | Read/write per-file byte cap; default small, e.g. 1 MiB. |
| `BOLI_MAX_TOTAL_BYTES` | Live value budget already exists; I/O usi budget me count hoga. |
| `BOLI_IMPORT_ROOT` | Imports ke liye alag canonical root; file root se merge nahi hoga. |

Rules:

- `BOLI_ALLOW_FS` unset ya `0` par har file builtin `capability-denied` error.
- `BOLI_FS_ROOT` set na ho to file access denied.
- Root `canonicalize` ho, directory honi chahiye, symlink itself allowed nahi.
- Program-provided path relative hona chahiye; absolute path refuse.
- Path `..`, empty component, `.` only, ya NUL contain kare to refuse.
- Final path `canonicalize` karke `starts_with(root)` check karo.
- Intermediate symlinks refuse (`symlink_metadata` se components check), follow mat karo.
- TOCTOU se bachne ke liye `openat`-style directory-relative open ya canonical parent + `O_NOFOLLOW` equivalent use karo; platform fallback me post-open `metadata` + inode check.
- Source files aur import files alag capability se treat karo; `BOLI_IMPORT_ROOT` se read allowed but generic file-write se protected.

## Proposed APIs

```boli
likho padho_file "data/notes.txt"
likho likho_file "out/notes.txt" "namaste"
likho jodo_file "out/notes.txt" "aur line"
likho file_hai "data/notes.txt"
likho mitao_file "out/notes.txt"
```

Signatures (planned, abhi implement nahi):
- `padho_file(path)` → text; missing file par typed error.
- `likho_file(path, text)` → `sahi`; atomic replace with temp file + rename.
- `jodo_file(path, text)` → `sahi`; single bounded open/append.
- `file_hai(path)` → `sahi/galat`; denied path par error, silent false nahi.
- `mitao_file(path)` → `sahi`; symlink/path escape par error.
- `file_size(path)` → number; metadata-only, cap se upar bhi size report kar sakta hai but read nahi.

Extension policy: default allow-list `.txt`, `.json`, `.csv`, `.md`, `.log`. `.boli`, `.rs`, `.sh`, `.bat`, `.exe` aur hidden dotfiles deny. Root owner allow-list dashboard/CI me configure karega; program khud deny override nahi kar sakta.

## Write safety

- Pehle temp file same directory me unique name se likho.
- Byte cap se pehle check karo; stream likhte waqt bhi count karo.
- `fsync` file, then atomic `rename`.
- Rename fail ho to temp file delete karo; original intact rahe.
- Existing target symlink ho to refuse; overwrite follow mat karo.
- Existing target directory ho to refuse.
- Partial write ke baad error aaye to original file unchanged hona chahiye.

## Error rules

Typeded categories add karne honge (`ERROR_HANDLING.md`):
- `capability-denied`: `BOLI_ALLOW_FS` ya root missing.
- `path-invalid`: absolute, `..`, NUL, empty, hidden, bad extension.
- `path-escape`: canonical path root ke bahar.
- `symlink-refused`: symlink component/target.
- `file-too-large`: byte cap.
- `io-error`: OS error, with path context.
- `limit-exceeded`: total/value budget.

Har error Hinglish display rakhega, e.g. `Galti: line 4 me path 'x' sandbox root se bahar hai`.

## Test plan (before implementation)

- Default-deny: bina `BOLI_ALLOW_FS` sab file builtins error.
- Missing root, invalid root, file root, relative/absolute path, `..`, symlink file, symlink directory.
- `BOLI_IMPORT_ROOT` aur `BOLI_FS_ROOT` alag hone par cross-access deny.
- Empty/NaN/very large text, multibyte UTF-8 byte counting.
- Existing file atomic replace, crash-simulation(temp leftover cleanup), read-only directory failure.
- Extension allow-list aur hidden files.
- Interpreter-vs-VM error/output parity.
- Windows path separators aur case behavior CI par.

## Gate before enabling

- [ ] `BoliError` me `path` field, aur parser/runtime `kind` values, backward-compatible display ke saath. Lexer `kind` Phase 3.2 me add ho gaya hai.
- [ ] VM stability gates se kam se kam 100k fuzz + `.bc` corpus green.
- [ ] Upari test plan green on Linux, macOS, Windows.
- [ ] Docs update: `RUNTIME_SPEC.md`, `STANDARD_LIBRARY.md`, `ERROR_HANDLING.md`.
- [ ] Explicit user approval for capability flags and default policy.
