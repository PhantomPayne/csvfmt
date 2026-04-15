//! End-to-end tests that exercise the `csvfmt` binary.

use std::io::Write;
use std::process::{Command, Stdio};

/// Helper: pipe `input` into `csvfmt <args…>` and return (stdout, stderr, success).
fn run(input: &str, args: &[&str]) -> (String, String, bool) {
    let bin = env!("CARGO_BIN_EXE_csvfmt");

    let mut child = Command::new(bin)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn csvfmt");

    child
        .stdin
        .as_mut()
        .expect("failed to open stdin")
        .write_all(input.as_bytes())
        .expect("failed to write to stdin");

    let output = child.wait_with_output().expect("failed to wait on csvfmt");
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.success(),
    )
}

/// Helper that asserts success and returns trimmed stdout.
fn run_ok(input: &str, args: &[&str]) -> String {
    let (stdout, stderr, ok) = run(input, args);
    assert!(ok, "csvfmt failed.\nstderr: {stderr}");
    stdout
}

// ── Basic substitution ──────────────────────────────────────────────────────

#[test]
fn basic_substitution() {
    let out = run_ok("Alice,30\n", &["Hello {1}, you are {2} years old."]);
    assert_eq!(out.trim(), "Hello Alice, you are 30 years old.");
}

#[test]
fn multiple_rows() {
    let out = run_ok("Alice,30\nBob,25\n", &["{1}: {2}"]);
    assert_eq!(out.trim(), "Alice: 30\nBob: 25");
}

// ── Named fields with headers ───────────────────────────────────────────────

#[test]
fn named_fields_with_header() {
    let input = "first,last,email\nAlice,Smith,alice@example.com\n";
    let out = run_ok(input, &["-H", "Dear {first} {last} <{email}>,"]);
    assert_eq!(out.trim(), "Dear Alice Smith <alice@example.com>,");
}

#[test]
fn named_fields_multiple_rows() {
    let input = "name,age\nAlice,30\nBob,25\n";
    let out = run_ok(input, &["-H", "{name} is {age}"]);
    assert_eq!(out.trim(), "Alice is 30\nBob is 25");
}

// ── Conditional blocks ──────────────────────────────────────────────────────

#[test]
fn conditional_present() {
    let out = run_ok("Alice,30,Engineer\n", &["Hello {1}{?3:, job: {3}}"]);
    assert_eq!(out.trim(), "Hello Alice, job: Engineer");
}

#[test]
fn conditional_absent() {
    let out = run_ok("Alice,30,\n", &["Hello {1}{?3:, job: {3}}"]);
    assert_eq!(out.trim(), "Hello Alice");
}

// ── Default values ──────────────────────────────────────────────────────────

#[test]
fn default_value_used() {
    let out = run_ok("Alice,,\n", &["{1} is {2:unknown} years old"]);
    assert_eq!(out.trim(), "Alice is unknown years old");
}

#[test]
fn default_value_not_used() {
    let out = run_ok("Alice,30,\n", &["{1} is {2:unknown} years old"]);
    assert_eq!(out.trim(), "Alice is 30 years old");
}

// ── TSV input ───────────────────────────────────────────────────────────────

#[test]
fn tsv_flag() {
    let out = run_ok("Alice\t30\n", &["--tsv", "{1}: {2}"]);
    assert_eq!(out.trim(), "Alice: 30");
}

#[test]
fn delimiter_flag_tab() {
    let out = run_ok("Alice\t30\n", &["-d", "\\t", "{1}: {2}"]);
    assert_eq!(out.trim(), "Alice: 30");
}

#[test]
fn delimiter_flag_semicolon() {
    let out = run_ok("Alice;30;Berlin\n", &["-d", ";", "{1} lives in {3}"]);
    assert_eq!(out.trim(), "Alice lives in Berlin");
}

// ── Trim ────────────────────────────────────────────────────────────────────

#[test]
fn trim_whitespace() {
    let out = run_ok(" Alice , 30 \n", &["--trim", "Name={1}, Age={2}"]);
    assert_eq!(out.trim(), "Name=Alice, Age=30");
}

// ── Skip empty ──────────────────────────────────────────────────────────────

#[test]
fn skip_empty_rows() {
    let out = run_ok("Alice,30\n\n,\nBob,25\n", &["--skip-empty", "{1}: {2}"]);
    assert_eq!(out.trim(), "Alice: 30\nBob: 25");
}

// ── Escaped braces ──────────────────────────────────────────────────────────

#[test]
fn escaped_braces() {
    let out = run_ok("Alice\n", &["{{{1}}}"]);
    assert_eq!(out.trim(), "{Alice}");
}

// ── File input ──────────────────────────────────────────────────────────────

#[test]
fn input_from_file() {
    let dir = std::env::temp_dir().join("csvfmt_test");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("test.csv");
    std::fs::write(&file, "Alice,30\nBob,25\n").unwrap();

    let bin = env!("CARGO_BIN_EXE_csvfmt");
    let output = Command::new(bin)
        .args(["-i", file.to_str().unwrap(), "{1}: {2}"])
        .output()
        .expect("failed to run csvfmt");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "Alice: 30\nBob: 25");

    std::fs::remove_dir_all(&dir).ok();
}

// ── Error cases ─────────────────────────────────────────────────────────────

#[test]
fn error_on_invalid_template() {
    let (_, stderr, ok) = run("Alice\n", &["{1"]);
    assert!(!ok, "should fail on unclosed brace");
    assert!(stderr.contains("invalid template"), "stderr: {stderr}");
}

#[test]
fn error_on_missing_file() {
    let bin = env!("CARGO_BIN_EXE_csvfmt");
    let output = Command::new(bin)
        .args(["-i", "/nonexistent/path.csv", "{1}"])
        .output()
        .expect("failed to run csvfmt");
    assert!(!output.status.success());
}

// ── SQL generation (practical use case) ─────────────────────────────────────

#[test]
fn sql_insert_generation() {
    let out = run_ok(
        "Alice,30\nBob,25\n",
        &["INSERT INTO users (name, age) VALUES ('{1}', {2});"],
    );
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(
        lines[0],
        "INSERT INTO users (name, age) VALUES ('Alice', 30);"
    );
    assert_eq!(
        lines[1],
        "INSERT INTO users (name, age) VALUES ('Bob', 25);"
    );
}

// ── Markdown links (practical use case) ─────────────────────────────────────

#[test]
fn markdown_link_generation() {
    let out = run_ok("Google,https://google.com\n", &["[{1}]({2})"]);
    assert_eq!(out.trim(), "[Google](https://google.com)");
}
