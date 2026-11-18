BOLI - Bina + - * / ke, sirf shabdon se programming

**BOLI** ek Hinglish programming language hai — Python jaisi easy, lekin poori tarah Hinglish me.
Yahan `+ - * / { } ( ) = :` jaise symbols ki jagah shabd use hote hain. Poor poora program sirf shabdon, number aur text se likha jata hai.
Interpreter **Rust** me bana hai. File extension: **`.hl`**.

```boli
likho "Namaste Duniya"
```

Output:
```
Namaste Duniya
```

---

## BOLI chalane ke liye kya chahiye?

**Chalane ke liye sirf `boli` binary kaafi hai. Rust, Python, ya koi aur cheez install karne ki zaroorat NAHI.**
Rust sirf BOLI ko source se **build** karne ke liye chahiye. Ek baar ban gaya, to `boli` akela chalta hai — zero dependencies.

---

## Install karna

### Step 1: Source clone karke build karo (ek dafa)

```bash
git clone https://github.com/termux-learn-crypto/boli
cd boli
cargo build --release
```

### Step 2: Install script chalao

```bash
./install.sh
```

Yeh `boli` binary ko copy karke PATH me daal deta hai:

- **Termux**: `$PREFIX/bin` (automatic)
- **Linux**: `/usr/local/bin` (writable ho to), warna `~/.local/bin`

Custom directory me install karna ho to:

```bash
BOLI_INSTALL_DIR=/path/to/bin ./install.sh
```

### Step 3: Ab bina cargo ke chalao

```bash
boli run examples/namaste.hl
boli                     # REPL
boli help                # help
boli version             # version
```

Install nahi karna? Seedha binary bhi chal sakta hai:

```bash
./target/release/boli run examples/namaste.hl
```

> Windows par: `install.sh` Linux/Termux ke liye hai. Windows par `BOLI_INSTALL_DIR` style manually karo ya WSL use karo.

---

## 3 examples — seedha chalake dekho

```bash
boli run examples/namaste.hl     # input lena + print
boli run examples/calculator.hl  # 4 hisab dono forms me
boli run examples/loop.hl        # se tak loop, jab tak, agar
```

### 1. namaste.hl — shuruaat, variable, input

```boli
# namaste.hl
likho "Namaste Duniya"
naam rakho "Vilas"
likho naam
pucho tumhara_naam "Tumhara naam kya hai? "
likho tumhara_naam
```

### 2. calculator.hl — hisab jodo/ghatao/guna/bhaag

```boli
# calculator.hl (short me)
pucho a "Pehli sankhya: "
pucho b "Doosri sankhya: "
jodo jod a aur b
ghatao ghata a se b
guna gun a ko b se
bhaag bhag a ko b se
likho jod
likho ghata
likho gun
likho bhag
```

### 3. loop.hl — `se tak` range loop aur `jab tak` while loop

```boli
1 se 5 tak
    likho ginti
khatam

ginti2 rakho 1
jab tak ginti2 5 se chhota ya barabar hai to
    likho ginti2
    jodo ginti2 ginti2 aur 1
khatam
```

---

## Calculator ka demo — calculator2.hl

`examples/calculator2.hl` ek hisab wala calculator hai: user ek hi hisab chunta hai (jodo / ghatao / guna / bhaag) aur wahi hota hai. Bina ek bhi `+ - * /` symbol ke, sirf shabdon me:

```boli
# calculator2.hl (short me)
pucho a "Pehli sankhya: "
pucho b "Doosri sankhya: "
pucho kaam "kya karna hai? jodo, ghatao, guna, bhaag"

milgaya rakho galat

agar kaam "jodo" barabar hai to
    jodo jawab a aur b
    likho jawab
    milgaya rakho sahi
khatam

# ghatao / guna / bhaag bhi aise hi ...

agar milgaya galat barabar hai to
    likho "Galat kaam likha hai"
khatam
```

Chalane ka demo:

```bash
boli run examples/calculator2.hl
```

```
=== BOLI Calculator (ek hisab) ===
Pehli sankhya: 5
Doosri sankhya: 3
kya karna hai? jodo, ghatao, guna, bhaag jodo
8
```

Agar user `jod` jaisa galat hisab likhe, to `Galat kaam likha hai` print hota hai. Chaar `agar` check hote hain (string `barabar` tulna se), aur `milgaya` flag rakha gaya hai taaki error message ho sirf ek baar.

> Note: `kaam` BOLI me function keyword bhi hai aur `kaam` naam ka variable bhi ho sakta hai. Statement ki shuruwat me `kaam rakho 5` likhne pe woh variable banega; `kaam namaste` likhne pe woh function definition hoga.

---

## REPL (interactive shell)

Bina argument ke `boli` chalao, REPL khulega:

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

Multi-line block (jaise `agar` ... `khatam`) likhne pe `...>` prompt aata hai. Variable aur kaam environment REPL me bache rehte hain.

---

## BOLI ke niyam (rules)

### 1. Likhna — `likho`

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

Input ka type apne aap pehchana jata hai: `99.50` number ban jata hai, `sahi`/`galat`/`khaali` bhi pehchane jate hain, baaki sab text. Blank input `khaali` banta hai.

### 4. Hisab — `jodo`, `ghatao`, `guna`, `bhaag`

Variable me hisaab. **`rakho` yahan use NAHI hota**:

```boli
jodo jawab 5 aur 3        # jawab = 5 + 3
ghatao bacha 10 se 4      # bacha = 10 - 4
guna total 5 ko 2 se      # total = 5 * 2
bhaag hissa 10 ko 2 se    # hissa = 10 / 2
likho jawab
```

Seedha `likho` ke saath:

```boli
likho jodo 5 aur 3        # 8
likho ghatao 10 se 4      # 6
likho guna 5 ko 2 se      # 10
likho bhaag 10 ko 4 se    # 2.5
```

Nesting chalta hai: `likho jodo guna 5 ko 2 se aur 3` → `13`. Zero se bhaag karne pe galti. Text bhi `jodo` se jud jata hai.

### 5. Shart (Condition) — `agar ... nahi to ... khatam`

```boli
umar rakho 28
agar umar 18 se bada hai to
    likho "Adult ho"
nahi to
    likho "Bacche ho"
khatam
```

| BOLI | matlab |
|------|--------|
| `bada` | > |
| `chhota` | < |
| `barabar` | == |
| `bada ya barabar` | >= |
| `chhota ya barabar` | <= |

`se` optional hai — `agar umar 18 se bada hai to` aur `agar umar 18 bada hai to` dono chalte hain. `nahi to` optional hai.

```boli
# aur / ya
agar umar 25 se bada hai aur umar 60 se chhota hai to
    likho "Kam karne ki umar"
nahi to
    likho "Umr kuch aur hai"
khatam
```

**Precedence (BOLI me):** expression nesting → `hissa` → comparison → `aur` → `ya`

BOLI me symbol parentheses nahi hain — group karna ho to nested expression likho:

```boli
agar jodo 1 aur 2 barabar jodo 2 aur 1 hai to   # 3 == 3
    likho "sahi"
khatam
```

`aur` (AND) hamesha `ya` (OR) se pehle bind hota hai: `A ya B aur C` = `A ya (B aur C)`.

### 6. Loops

**Range loop** — `1 se 5 tak` (dono ends shamil, `ginti` variable apne aap milta hai):

```boli
1 se 5 tak
    likho ginti
khatam
```

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

### 8. Value wapas karna — `wapas do`

```boli
kaam jodo_kaam a b
    jodo jawab a aur b
    wapas do jawab
khatam

x rakho jodo_kaam 20 8
likho x               # 28
likho jodo_kaam 100 1 # seedha likho me bhi (101)
```

Bina value ke `wapas do` likhne pe `khaali` wapas milta hai. `wapas do` kaam ke andar hi allowed hai.

### 9. Types

| BOLI type | example |
|-----------|---------|
| text | `"Vilas"` |
| number (int/float, ek hi type) | `28`, `99.50` |
| sahi / galat | `sahi`, `galat` |
| khaali | `khaali` |

Number `28.0` ko print me `28` dikhaya jata hai, `99.5` waise ka-waisa dikhata hai.

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
```

---

## Help

```bash
boli help       # commands dikhao
boli version    # version dikhao
```

---

## Project structure

```
boli/
├── Cargo.toml
├── LICENSE             # MIT
├── install.sh          # Termux/Linux installer (binary ko PATH me dalta hai)
├── README.md
├── LANGUAGE_SPEC.md    # poora grammar/semantics reference
├── ARCHITECTURE.md     # code flow aur module design
├── CHANGELOG.md        # v0.1.0 → v0.2.0 changes
├── TESTING.md          # tests kaise chalayein, kaise likhein
├── AUDIT_REPORT.md     # audit + "how verified" notes
├── examples/
│   ├── namaste.hl
│   ├── calculator.hl
│   ├── calculator2.hl
│   └── loop.hl
├── src/
│   ├── main.rs         # CLI + REPL
│   ├── lib.rs          # library crate (tests isi pe based)
│   ├── lexer.rs        # code ko tokens me todta hai
│   ├── parser.rs       # tokens se AST banata hai
│   ├── interpreter.rs  # AST ko chalakar output deta hai
│   └── ast.rs          # AST nodes + Galti (error) type
└── tests/
    ├── boli.rs         # semantics + regression tests (library-level)
    └── cli.rs          # binary-level end-to-end tests (spawns boli)
```

## Testing

```bash
cargo test          # 67 tests chalenge
```

Details aur test-adding guide: [TESTING.md](TESTING.md). Spec: [LANGUAGE_SPEC.md](LANGUAGE_SPEC.md). Changes: [CHANGELOG.md](CHANGELOG.md).

Kaam karne ka flow: `source.hl` → **lexer** (tokens) → **parser** (AST) → **interpreter** (run).

---

## Feature list

- `likho`, `rakho`, `pucho`
- `jodo`, `ghatao`, `guna`, `bhaag` (inline aur variable-assign dono forms)
- `agar ... nahi to ... khatam`, `aur / ya` logic, `bada / chhota / barabar / bada ya barabar / chhota ya barabar`
- `se ... tak` range loop, `jab tak` while loop
- `kaam` definitions, calls, `wapas do`
- types: text, number, sahi/galat, khaali

## Aage kya aa sakta hai

- list/array type
- string formatting kaam
- kaam ke saath default arguments
- modules (`laao`)

---

## License

MIT — dekho [LICENSE](LICENSE).
