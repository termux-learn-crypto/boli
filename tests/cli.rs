// BOLI CLI end-to-end tests (binary level)

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn boli_bin() -> &'static str {
    env!("CARGO_BIN_EXE_boli")
}

// temp .hl program likho aur path wapas do
fn write_program(name: &str, contents: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, contents).unwrap();
    path
}

// program chalao, stdin feed karo, exit code / stdout / stderr do
// har test ka apna file naam (tests parallel chalte hain)
fn run_with_stdin(
    name: &str,
    program: &str,
    stdin_text: &str,
) -> (std::process::ExitStatus, String, String) {
    let path = write_program(name, program);
    let mut child = Command::new(boli_bin())
        .arg("run")
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("boli spawn nahi hua");
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(stdin_text.as_bytes()).unwrap();
    }
    let output = child.wait_with_output().unwrap();
    (
        output.status,
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn cli_runs_loop_example() {
    let output = Command::new(boli_bin())
        .args(["run", "examples/loop.hl"])
        .output()
        .expect("boli binary chala nahi");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Adult ho"));
    assert!(stdout.contains("Namaste"));
}

#[test]
fn cli_missing_file_gives_hinglish_error() {
    let output = Command::new(boli_bin())
        .args(["run", "no_such_file_xyz.hl"])
        .output()
        .expect("boli binary chala nahi");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Galti:"));
}

#[test]
fn cli_repl_extra_khatam_does_not_latch() {
    // pehle extra khatam se REPL tut-na chahiye, phir bhi program chalna chahiye
    let mut child = Command::new(boli_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("REPL spawn nahi hua");
    {
        let stdin = child.stdin.as_mut().unwrap();
        write!(stdin, "khatam\nlikho \"after-khatam\"\nnikalo\n").unwrap();
    }
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("after-khatam"),
        "REPL ne text nahi print kiya: {}",
        stdout
    );
    assert!(output.status.success());
}

#[test]
fn cli_repl_multiblock() {
    let mut child = Command::new(boli_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("REPL spawn nahi hua");
    {
        let stdin = child.stdin.as_mut().unwrap();
        // multi-line block REPL me abhi bhi kaam karta hai
        write!(
            stdin,
            "agar 5 3 se bada hai to\nlikho \"inside\"\nkhatam\nnikalo\n"
        )
        .unwrap();
    }
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("inside"),
        "multi-block REPL nahi chala: {}",
        stdout
    );
}

#[test]
fn cli_calculator2_example_works() {
    let mut child = Command::new(boli_bin())
        .args(["run", "examples/calculator2.hl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("calculator2 spawn nahi hua");
    {
        let stdin = child.stdin.as_mut().unwrap();
        write!(stdin, "6\n7\nguna\n").unwrap();
    }
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("42"), "guna ka jawab nahi aaya: {}", stdout);
}

// ---------- BUG 5: pucho input with non-finite values ----------

#[test]
fn cli_pucho_nan_stays_text() {
    let (status, stdout, _) = run_with_stdin(
        "nan.hl",
        "pucho x \"X: \"\njodo y x aur 1\nlikho y",
        "nan\n",
    );
    assert!(status.success());
    assert!(
        stdout.contains("nan1"),
        "nan text banana chahiye tha, got: {}",
        stdout
    );
}

#[test]
fn cli_pucho_inf_stays_text() {
    let (status, stdout, _) = run_with_stdin(
        "inf.hl",
        "pucho x \"X: \"\njodo y x aur 1\nlikho y",
        "inf\n",
    );
    assert!(status.success());
    assert!(stdout.contains("inf1"));
}

#[test]
fn cli_pucho_negative_infinity_stays_text() {
    let (status, stdout, _) = run_with_stdin(
        "neg_inf.hl",
        "pucho x \"X: \"\njodo y x aur 1\nlikho y",
        "-Infinity\n",
    );
    assert!(status.success());
    assert!(stdout.contains("-Infinity1"));
}

#[test]
fn cli_pucho_infinity_word_stays_text() {
    let (status, stdout, _) = run_with_stdin(
        "inf_word.hl",
        "pucho x \"X: \"\njodo y x aur 1\nlikho y",
        "Infinity\n",
    );
    assert!(status.success());
    assert!(stdout.contains("Infinity1"));
}

#[test]
fn cli_pucho_number_works_as_number() {
    let (status, stdout, _) = run_with_stdin(
        "decimal.hl",
        "pucho x \"X: \"\njodo y x aur 1\nlikho y",
        "99.50\n",
    );
    assert!(status.success());
    assert!(stdout.contains("100.5"));
}

#[test]
fn cli_pucho_huge_finite_number_stays_number() {
    let (status, stdout, _) = run_with_stdin(
        "huge.hl",
        "pucho x \"X: \"\njodo y x aur 0\nlikho y",
        "1.7976931348623157e308\n",
    );
    assert!(status.success());
    // finite max f64 + 0 hi jawab hai
    assert!(stdout.contains("1.7976931348623157e308") || stdout.contains("179769313486231570"));
}

#[test]
fn cli_reken_overflow_is_hinglish_error_not_inf() {
    let (status, stdout, stderr) = run_with_stdin(
        "overflow.hl",
        "y rakho 1000000000000000\nguna y y ko y se\nguna y y ko y se\nguna y y ko y se\nguna y y ko y se\nguna y y ko y se\nlikho y",
        "",
    );
    assert!(!status.success());
    assert!(
        stderr.contains("finite nahi raha"),
        "overflow error expect kiya tha, got: {}",
        stderr
    );
    let _ = stdout;
}

// ---------- misc regression ----------

#[test]
fn cli_broken_pipe_exits_cleanly() {
    // stdout consume na karke seedha close karo: boli panic nahi karna chahiye
    let stdout_pipe = Command::new(boli_bin())
        .args(["run", "examples/loop.hl"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("boli spawn nahi hua");
    let output = stdout_pipe.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn cli_bogus_subcommand_errors() {
    let output = Command::new(boli_bin())
        .arg("boguscommand")
        .output()
        .expect("boli binary chala nahi");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Galti"));
}

#[test]
fn cli_run_with_too_many_args_errors() {
    let output = Command::new(boli_bin())
        .args(["run", "a.hl", "extra"])
        .output()
        .expect("boli binary chala nahi");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Galti"));
}

#[test]
fn cli_source_bytes_boundary() {
    // exactly at limit (8 bytes, valid program) -> runs
    let ok_program = "likho 1\n"; // 8 bytes
    let path_ok = write_program("source_bytes_ok.hl", ok_program);
    let output = Command::new(boli_bin())
        .arg("run")
        .arg(&path_ok)
        .env("BOLI_MAX_SOURCE_BYTES", "8")
        .output()
        .expect("boli binary chala nahi");
    assert!(
        output.status.success(),
        "8-byte program with limit 8 should run"
    );

    // one byte under -> Galti
    let output = Command::new(boli_bin())
        .arg("run")
        .arg(&path_ok)
        .env("BOLI_MAX_SOURCE_BYTES", "7")
        .output()
        .expect("boli binary chala nahi");
    assert!(
        !output.status.success(),
        "8-byte program with limit 7 must fail"
    );
}

#[test]
fn cli_env_eval_depth_is_respected() {
    // deep nesting jodo - limit 10 par fail, 500 par pass
    let depth = 20;
    let program = format!(
        "likho {}{}1{}",
        "jodo ".repeat(depth),
        "",
        " aur 1".repeat(depth)
    );
    let path = write_program("eval_depth_env.hl", &program);

    let output = Command::new(boli_bin())
        .arg("run")
        .arg(&path)
        .env("BOLI_MAX_EVAL_DEPTH", "10")
        .output()
        .expect("boli binary chala nahi");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("expression bahut zyada gehra hai"));

    let output = Command::new(boli_bin())
        .arg("run")
        .arg(&path)
        .env("BOLI_MAX_EVAL_DEPTH", "500")
        .output()
        .expect("boli binary chala nahi");
    assert!(output.status.success());
}

#[test]
fn cli_repl_oversized_line_is_rejected_and_repl_survives() {
    let long_line = "x".repeat(200);
    let input = format!("{}\nlikho 42\nnikalo\n", long_line);
    let output = Command::new(boli_bin())
        .env("BOLI_MAX_REPL_LINE_BYTES", "50")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write as _;
            child.stdin.as_mut().unwrap().write_all(input.as_bytes())?;
            child.wait_with_output()
        })
        .expect("REPL spawn/write nahi hua");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // oversized line reject + REPL zinda raha + agli command chal gayi
    assert!(stderr.contains("line bahut badi hai"), "stderr: {}", stderr);
    assert!(stdout.contains("42"), "stdout: {}", stdout);
    assert!(stdout.contains("Alvida"), "stdout: {}", stdout);
}
