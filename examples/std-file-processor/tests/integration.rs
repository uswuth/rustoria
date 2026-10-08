//! Integration tests using real files in the system temp directory.
//! No `tempfile` crate: fixtures are created and cleaned up with `std::fs`.

use std::fs;
use std::path::PathBuf;

use std_file_processor::{collect_text_files, count_path, top_n, ProcessingError};

/// A temporary directory that deletes itself on drop.
struct TempFixture {
    dir: PathBuf,
}

impl TempFixture {
    fn new(test_name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "std-file-processor-test-{}-{}",
            test_name,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir); // clean leftovers from a crashed run
        fs::create_dir_all(&dir).unwrap();
        Self { dir }
    }

    fn write_file(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.dir.join(name);
        fs::write(&path, contents).unwrap();
        path
    }
}

impl Drop for TempFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn counts_words_in_single_file() {
    let fixture = TempFixture::new("single");
    let file = fixture.write_file("input.txt", "the quick brown fox\nthe lazy dog\n");

    let counts = count_path(&file).unwrap();

    assert_eq!(counts["the"], 2);
    assert_eq!(counts["quick"], 1);
    assert_eq!(counts["dog"], 1);
    assert_eq!(counts.len(), 6);
}

#[test]
fn counts_words_across_directory_of_txt_files() {
    let fixture = TempFixture::new("dir");
    fixture.write_file("a.txt", "shared apple");
    fixture.write_file("b.txt", "shared banana");
    // Non-txt files must be ignored.
    fixture.write_file("notes.md", "shared cherry");

    let counts = count_path(&fixture.dir).unwrap();

    assert_eq!(counts["shared"], 2);
    assert_eq!(counts["apple"], 1);
    assert_eq!(counts["banana"], 1);
    assert!(!counts.contains_key("cherry"));
}

#[test]
fn collect_text_files_is_sorted_and_filters_extensions() {
    let fixture = TempFixture::new("collect");
    fixture.write_file("b.txt", "");
    fixture.write_file("a.txt", "");
    fixture.write_file("c.md", "");

    let files = collect_text_files(&fixture.dir).unwrap();
    let names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
        .collect();

    assert_eq!(names, vec!["a.txt", "b.txt"]);
}

#[test]
fn missing_path_reports_not_found() {
    let fixture = TempFixture::new("missing");
    let missing = fixture.dir.join("does-not-exist.txt");

    let err = count_path(&missing).unwrap_err();

    assert!(matches!(err, ProcessingError::NotFound { .. }));
    // The Display impl must mention the path.
    assert!(err.to_string().contains("does-not-exist.txt"));
}

#[test]
fn directory_without_txt_files_reports_no_text_files() {
    let fixture = TempFixture::new("empty");
    fixture.write_file("data.csv", "a,b,c");

    let err = count_path(&fixture.dir).unwrap_err();

    assert!(matches!(err, ProcessingError::NoTextFiles { .. }));
}

#[test]
fn top_n_over_real_files() {
    let fixture = TempFixture::new("topn");
    fixture.write_file("one.txt", "red red blue");
    fixture.write_file("two.txt", "red green blue");

    let counts = count_path(&fixture.dir).unwrap();
    let top = top_n(&counts, 2);

    assert_eq!(top, vec![("red", 3), ("blue", 2)]);
}

// --- CLI contract: exit codes distinguish help, usage errors, and success ---

/// Run the actual binary and return (exit_code, stderr).
fn run_cli(args: &[&str]) -> (i32, String) {
    use std::process::Command;

    let output = Command::new(env!("CARGO_BIN_EXE_std-file-processor"))
        .args(args)
        .output()
        .expect("failed to launch binary");
    let code = output.status.code().expect("process killed by signal");
    (code, String::from_utf8_lossy(&output.stderr).into_owned())
}

#[test]
fn help_exits_zero_on_stdout() {
    use std::process::Command;

    let output = Command::new(env!("CARGO_BIN_EXE_std-file-processor"))
        .arg("--help")
        .output()
        .expect("failed to launch binary");

    assert_eq!(output.status.code(), Some(0), "--help must succeed");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Usage:"),
        "--help belongs on stdout"
    );
}

#[test]
fn usage_errors_exit_two() {
    let (code, stderr) = run_cli(&[]);
    assert_eq!(code, 2, "missing argument must be a usage error");
    assert!(stderr.contains("missing <PATH>"), "stderr: {stderr}");

    let (code, stderr) = run_cli(&["--top", "abc", "file.txt"]);
    assert_eq!(code, 2, "bad --top must be a usage error");
    assert!(stderr.contains("--top"), "stderr: {stderr}");

    let (code, _) = run_cli(&["--no-such-flag"]);
    assert_eq!(code, 2, "unknown flag must be a usage error");
}

#[test]
fn missing_file_exits_one() {
    let fixture = TempFixture::new("cli-missing");
    let missing = fixture.dir.join("nope.txt");

    let (code, stderr) = run_cli(&[missing.to_str().unwrap()]);
    assert_eq!(code, 1, "runtime errors use the generic failure code");
    assert!(stderr.contains("error:"), "stderr: {stderr}");
}
