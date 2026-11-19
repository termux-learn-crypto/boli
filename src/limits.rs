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
pub const DEFAULT_MAX_IMPORT_DEPTH: u64 = 32; // module import nesting (laao)
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

use std::io::BufRead;

// Bounded stdin line read: max_bytes tak hi padhta hai (invalid UTF-8 -> lossy)
// EOF par Khaali milta hai (caller handle kare)
#[derive(Debug, PartialEq)]
pub(crate) enum InputLine {
    Line(String),
    TooLarge,
    Eof,
}

pub(crate) fn read_input_line(max_bytes: u64) -> std::io::Result<InputLine> {
    let mut stdin = std::io::stdin().lock();
    read_input_line_from(&mut stdin, max_bytes)
}

pub(crate) fn read_input_line_from<R: BufRead>(
    reader: &mut R,
    max_bytes: u64,
) -> std::io::Result<InputLine> {
    let mut buf: Vec<u8> = Vec::new();
    loop {
        let chunk = match reader.fill_buf() {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        if chunk.is_empty() {
            return Ok(if buf.is_empty() {
                InputLine::Eof
            } else {
                InputLine::Line(String::from_utf8_lossy(&buf).into_owned())
            });
        }
        match chunk.iter().position(|&b| b == b'\n') {
            Some(pos) => {
                if buf.len() as u64 + pos as u64 > max_bytes {
                    let len = pos + 1;
                    reader.consume(len);
                    return Ok(InputLine::TooLarge);
                }
                buf.extend_from_slice(&chunk[..pos]);
                reader.consume(pos + 1);
                return Ok(InputLine::Line(String::from_utf8_lossy(&buf).into_owned()));
            }
            None => {
                let len = chunk.len();
                if buf.len() as u64 + len as u64 > max_bytes {
                    reader.consume(len);
                    drain_through_newline(reader)?;
                    return Ok(InputLine::TooLarge);
                }
                buf.extend_from_slice(chunk);
                reader.consume(len);
            }
        }
    }
}

fn drain_through_newline<R: BufRead>(reader: &mut R) -> std::io::Result<()> {
    loop {
        let chunk = match reader.fill_buf() {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        if chunk.is_empty() {
            return Ok(());
        }
        let len = chunk
            .iter()
            .position(|&b| b == b'\n')
            .map_or(chunk.len(), |pos| pos + 1);
        let hit_newline = chunk[len - 1] == b'\n';
        reader.consume(len);
        if hit_newline {
            return Ok(());
        }
    }
}

use crate::ast::BoliError;
use std::io::Read;
use std::path::Path;

// Bounded file read: max_source + 1 bytes tak hi padhta hai; zyada ho to Galti
// file read ke dauraan grow/replace bhi ho sakti hai (metadata alone TOCTOU-unsafe)
pub(crate) fn read_source_capped(
    path: &Path,
    max_source: u64,
    line: usize,
) -> Result<String, BoliError> {
    let file = std::fs::File::open(path).map_err(|e| {
        BoliError::line(line, format!("file '{}' nahi khuli: {}", path.display(), e))
    })?;
    let mut buf: Vec<u8> = Vec::new();
    file.take(max_source + 1)
        .read_to_end(&mut buf)
        .map_err(|e| {
            BoliError::line(
                line,
                format!("file '{}' padh nahi paye: {}", path.display(), e),
            )
        })?;
    if buf.len() as u64 > max_source {
        return Err(BoliError::line(
            line,
            format!(
                "file '{}' bahut badi hai ({} bytes se zyada, max {})",
                path.display(),
                buf.len() - 1,
                max_source
            ),
        ));
    }
    String::from_utf8(buf)
        .map_err(|_| BoliError::line(line, format!("file '{}' UTF-8 nahi hai", path.display())))
}

pub(crate) fn resolve_import_path(
    raw_path: &str,
    base: &Path,
    line: usize,
) -> Result<std::path::PathBuf, BoliError> {
    let joined = if std::path::Path::new(raw_path).is_absolute() {
        std::path::PathBuf::from(raw_path)
    } else {
        base.join(raw_path)
    };
    let canonical = std::fs::canonicalize(&joined)
        .map_err(|e| BoliError::line(line, format!("file '{}' nahi mili: {}", raw_path, e)))?;
    if let Some(root_raw) = std::env::var_os("BOLI_IMPORT_ROOT").filter(|v| !v.is_empty()) {
        let root = std::fs::canonicalize(&root_raw).map_err(|e| {
            BoliError::line(
                line,
                format!(
                    "import root '{}' nahi mila: {}",
                    root_raw.to_string_lossy(),
                    e
                ),
            )
        })?;
        if !root.is_dir() {
            return Err(BoliError::line(
                line,
                format!(
                    "import root '{}' directory nahi hai",
                    root_raw.to_string_lossy()
                ),
            ));
        }
        if !canonical.starts_with(&root) {
            return Err(BoliError::line(
                line,
                format!("import '{}' root se bahar hai (BOLI_IMPORT_ROOT)", raw_path),
            ));
        }
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn oversized_line_with_newline_in_same_chunk_is_drained() {
        let mut reader = BufReader::new(Cursor::new(b"abcdef\nnext\n".to_vec()));
        assert!(matches!(
            read_input_line_from(&mut reader, 5).unwrap(),
            InputLine::TooLarge
        ));
        match read_input_line_from(&mut reader, 5).unwrap() {
            InputLine::Line(line) => assert_eq!(line, "next"),
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn oversized_line_across_chunks_is_drained() {
        let mut reader = BufReader::with_capacity(4, Cursor::new(b"abcdefgh\nnext\n".to_vec()));
        assert!(matches!(
            read_input_line_from(&mut reader, 5).unwrap(),
            InputLine::TooLarge
        ));
        match read_input_line_from(&mut reader, 5).unwrap() {
            InputLine::Line(line) => assert_eq!(line, "next"),
            other => panic!("unexpected: {:?}", other),
        }
    }
}
