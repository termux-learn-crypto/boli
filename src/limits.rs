// BOLI safety limits - ek hi jagah se env override aur defaults
// Har limit env variable se badal sakte ho (0 ya invalid = default)

pub const DEFAULT_MAX_STEPS: u64 = 100_000; // poora program ke steps
pub const DEFAULT_MAX_CALL_DEPTH: u64 = 64; // kaam call nesting
pub const DEFAULT_MAX_STRING_BYTES: u64 = 1024 * 1024; // 1 MiB string
pub const DEFAULT_MAX_SOURCE_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB source file
pub const DEFAULT_MAX_LIST_ITEMS: u64 = 10_000; // ek list me items
pub const DEFAULT_MAX_PARSE_DEPTH: u64 = 64; // nested statements/expressions
pub const DEFAULT_MAX_CONDITION_DEPTH: u64 = 128; // aur/ya chain length
pub const DEFAULT_MAX_EVAL_DEPTH: u64 = 256; // nested expression evaluation
pub const DEFAULT_MAX_TOKENS: u64 = 1_000_000; // max tokens ek program me
pub const DEFAULT_MAX_REPL_LINE_BYTES: u64 = 64 * 1024; // REPL input line
pub const DEFAULT_MAX_TOTAL_BYTES: u64 = 64 * 1024 * 1024; // live scope value bytes budget

// env var ko u64 me padho; 0 ya galat value par default
pub fn env_limit(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(default)
}
