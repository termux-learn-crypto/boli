# BOLI

**BOLI** ek Hinglish programming language hai — Python jaisi easy, lekin poori tarah Hinglish me.
Koi symbol `; { } ( ) = : .` use nahi hota. Sirf shabd, number aur text se poore program ban sakte hain.
Implementation language: **Rust**. File extension: **`.hl`**.

```boli
likho "Namaste Duniya"
```

Output:
```
Namaste Duniya
```

---

## Install aur chalana

Rust aur Cargo hone chahiye. Fir:

```bash
cargo build --release
```

### File chalana

```bash
cargo run -- run examples/namaste.hl
```

### REPL (interactive shell)

Bina kuch likhe `cargo run` likhao, REPL khulega:

```bash
cargo run
```

```
BOLI REPL - likhne ke liye tayyar!
REPL se nikalne ke liye 'nikalo' likho.
boli> likho "Namaste"
Namaste
boli> 1 se 3 tak
...> likho ginti
...> khatam
1
2
3
boli> nikalo
BOLI band ho raha hai. Alvida!
```

REPL me multi-line block (jaise `agar` ... `khatam`) likhne pe `...>` prompt aata hai, aur variable/kaam environment me bache rehte hain.

### Help

```bash
cargo run -- help      # kya command hain
cargo run -- version   # version
```

---

## BOLI ke niyam (rules)

### 1. Likhna — `likho`

Text print karo, ya kisi variable ki value:

```boli
likho "Namaste Duniya"
naam rakho "Vilas"
likho naam
```

### 2. Variable banana — `rakho`

```boli
naam rakho "Vilas"     # text
umar rakho 28          # number (int)
paisa rakho 99.50      # number (float)
shaadi rakho sahi      # sahi / galat
vichar rakho khaali    # khaali (null)
```

### 3. Input lena — `pucho`

```boli
pucho naam "Tumhara naam kya hai"
likho naam
```

Input ka type apne aap pehchana jata hai: `99.50` jaise input number ban jata hai, `sahi`/`galat`/`khaali` bhi pehchane jate hain, aur baaki sab text. Blank input `khaali` banta hai.

### 4. Hisab — `jodo`, `ghatao`, `guna`, `bhaag`

Variable me hisaab rakho. **`rakho` yahan use NAHI hota**, bas jodo/ghatao/guna/bhaag:

```boli
jodo jawab 5 aur 3        # jawab = 5 + 3
ghatao bacha 10 se 4      # bacha = 10 - 4
guna total 5 ko 2 se      # total = 5 * 2
bhaag hissa 10 ko 2 se    # hissa = 10 / 2
likho jawab
```

Direct hisaab seedha `likho` ke saath:

```boli
likho jodo 5 aur 3        # 8
likho ghatao 10 se 4      # 6
likho guna 5 ko 2 se      # 10
likho bhaag 10 ko 4 se    # 2.5
```

Nesting bhi chalta hai: `likho jodo guna 5 ko 2 se aur 3` → `13`.
Zero se bhaag karne pe galti aati hai. Text bhi `jodo` se jud jata hai (`likho jodo "Naam: " aur Vilas`).

### 5. Shart (Condition) — `agar ... nahi to ... khatam`

```boli
umar rakho 28
agar umar 18 se bada hai to
    likho "Adult ho"
nahi to
    likho "Bacche ho"
khatam
```

Tulna ke operator:

| BOLI | matlab |
|------|--------|
| `bada se` | > |
| `chhota se` | < |
| `barabar` | == |
| `bada ya barabar` | >= |
| `chhota ya barabar` | <= |

`se` optional hai — `agar umar 18 se bada hai to` aur `agar umar 18 bada hai to` dono chalte hain.

Logic operator:

```boli
agar umar 18 se bada ya barabar hai to        # >=
    likho "18 ya usse zyada"
khatam

# aur / ya
agar umar 25 se bada hai aur umar 60 se chhota hai to
    likho "Kam karne ki umar"
nahi to
    likho "Umr kuch aur hai"
khatam
```

`nahi to` optional hai.

### 6. Loops

**Range loop** — `1 se 5 tak` (dono ends shamil, `ginti` variable apne aap milta hai):

```boli
1 se 5 tak
    likho "Namaste"
    likho ginti     # 1 se 5 tak print hoga
khatam
```

Range me variable bhi use kar sakte hain: `1 se n tak ...`.

**Jab tak (while) loop** — `jab tak <condition> to ... khatam`:

```boli
ginti rakho 1
jab tak ginti 5 se chhota ya barabar hai to
    likho ginti
    jodo ginti ginti aur 1
khatam
```

### 7. Kaam (Function) — `kaam ... khatam`

```boli
kaam namaste naam
    likho "Namaste"
    likho naam
khatam

namaste "Rahul"
```

Function call ke saath arguments space se alag hote hain.

### 8. Value wapas karna — `wapas do`

```boli
kaam jodo_kaam a b
    jodo jawab a aur b
    wapas do jawab
khatam

x rakho jodo_kaam 20 8
likho x              # 28
likho jodo_kaam 100 1  # seedha likho me bhi (101)
```

Bina value ke `wapas do` likhne pe `khaali` wapas milta hai. `wapas do` kaam ke andar hi allowed hai.

### 9. Types

| BOLI type | Rust equivalent | example |
|-----------|-----------------|---------|
| text | String | `"Vilas"` |
| number (int/float ek hi type hai) | f64 | `28`, `99.50` |
| sahi / galat | bool | `sahi`, `galat` |
| khaali | null | `khaali` |

Number `28.0` ko print me `28` dikhaya jata hai, `99.5` waise ka-waisa dikhata hai. `likho` sab type dikha sakta hai.

---

## Comments

`#` se lekar line ke ant tak comment hai:

```boli
# yeh comment hai
likho "Yeh nahi"   # yeh bhi comment hai
```

---

## Error messages

Galti ki line number ke saath Hinglish message aata hai:

```
Galti: line 2 me naam 'xyz' nahi mila
Galti: line 4 me zero se bhaag nahi kar sakte
Galti: line 1 me ';' nahi chahiye, BOLI me sirf shabd, number aur "text" use hota hai
Galti: line 6 me 'khatam' chahiye tha, par 'file ka ant' mila
```

---

## Calculator chalakar dekho

```bash
cargo run -- run examples/calculator.hl
```

Input me `10` aur `4` dijiye:

```
=== BOLI Calculator ===
Pehli sankhya: 10
Doosri sankhya: 4
Jod (a + b):
14
...
```

---

## Ek hisab wala calculator — calculator2.hl

Agar user ko sirf **ek** hisab chahiye, to `examples/calculator2.hl` use "kya karna hai? jodo, ghatao, guna, bhaag" puchta hai aur wahi hisab kar deta hai:

```boli
# calculator2.hl (short me)
pucho a "Pehli sankhya: "
pucho b "Doosri sankhya: "
pucho kaam "kya karna hai? jodo, ghatao, guna, bhaag "

milgaya rakho galat

agar kaam "jodo" barabar hai to
    jodo jawab a aur b
    likho "Jawab hai:"
    likho jawab
    milgaya rakho sahi
khatam

# ... ghatao / guna / bhaag ke liye bhi aise hi ...

agar milgaya galat barabar hai to
    likho "Galat kaam likha hai"
khatam
```

Chaar `agar` check hote hain (string `barabar` tulna se). `milgaya` flag rakha gaya hai taaki galat naam — jaise `jod` — pe `Galat kaam likha hai` print ho aur ho sirf ek baar.

```bash
cargo run -- run examples/calculator2.hl
```

```
=== BOLI Calculator (ek hisab) ===
Pehli sankhya: 5
Doosri sankhya: 3
kya karna hai? jodo, ghatao, guna, bhaag jodo
Jawab hai:
8
```

> Note: `kaam` BOLI me function ka keyword bhi hai aur `kaam` naam ka variable bhi ho sakta hai. Statement ki shuruwat me `kaam rakho 5` likhne pe woh variable banega; `kaam namaste` likhne pe woh function definition hoga.

---

## Project structure

```
boli/
├── Cargo.toml
├── README.md
├── examples/
│   ├── namaste.hl
│   ├── calculator.hl
│   ├── calculator2.hl
│   └── loop.hl
└── src/
    ├── main.rs         # CLI + REPL
    ├── lexer.rs        # code ko tokens me todta hai
    ├── parser.rs       # tokens se AST banata hai
    ├── interpreter.rs  # AST ko chalakar output deta hai
    └── ast.rs          # AST nodes + Galti (error) type
```

Kaam karne ka flow: `source.hl` → **lexer** (tokens) → **parser** (AST) → **interpreter** (run).

---

## Pehle version me kya-kya hai

- `likho`, `rakho`, `pucho`
- `jodo`, `ghatao`, `guna`, `bhaag` (inline aur variable-assign dono forms)
- `agar ... nahi to ... khatam` with `bada / chhota / barabar / bada ya barabar / chhota ya barabar`
- `aur / ya` logic
- `se ... tak` range loop aur `jab tak` while loop
- `kaam` definitions, calls aur `wapas do`
- types: text, number, sahi/galat, khaali

## Aage kya aa sakta hai

- list/array type (jaise `cheezein rakho 1 aur 2 aur 3`)
- string formatting aur join kaam (`jodao`)
- kaam ke saath default arguments
- modules (`laao`)

---

## License

MIT
