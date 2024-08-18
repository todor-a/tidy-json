use assert_cmd::prelude::*;
use insta::{assert_debug_snapshot, with_settings};
use serde_json::Value;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

pub const UNSORTED_JSON: &str = r#"
{
    "c": 3,
    "b": 2,
    "a": 1
}"#;

pub fn init_git_repo(path: &Path) -> &std::path::Path {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .expect("Failed to initialize git repo");

    path
}

pub fn setup_test_directory() -> TempDir {
    TempDir::new().unwrap()
}

pub fn create_file(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
}

pub fn run_cli(include: &str, flags: &[&str], tmp_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("tidy-json").unwrap();
    cmd.arg(include).args(flags).current_dir(tmp_dir);
    cmd
}

pub fn setup_excludes_test() -> (TempDir, PathBuf, PathBuf) {
    let temp_dir = setup_test_directory();
    let temp_path = temp_dir.path();

    let sample_path = temp_path.join("sample.json");
    let ignored_path = temp_path.join("ignored.json");

    create_file(&sample_path, UNSORTED_JSON);
    create_file(&ignored_path, UNSORTED_JSON);

    (temp_dir, sample_path, ignored_path)
}

pub fn extract_processed_files(output: &[u8]) -> Vec<String> {
    let output_str = String::from_utf8(output.to_vec()).expect("Should be able to load output");

    output_str
        .lines()
        .filter(|line| line.contains(": Processed in"))
        .map(|line| line.split(':').next().unwrap_or("").to_string())
        .collect()
}

pub fn assert_expected_processed_files_count(processed_files: &[String], expected: usize) {
    assert_eq!(
        processed_files.len(),
        expected,
        "Expected only {} files to be processed",
        expected,
    );
}

pub fn assert_file_content<P: AsRef<Path>>(path: P, expect_sorted: bool) {
    let str = &fs::read_to_string(path).expect("Should have been able to read test file.");
    let content: Value = serde_json::from_str(str).expect("Should have been able to load content");

    with_settings!({
        description => if expect_sorted {"should be sorted"} else {"should not be sorted"},
    }, {
        assert_debug_snapshot!(content);
    });
}

pub fn assert_file_processed(
    processed_files: &[String],
    file_name: &str,
    should_be_processed: bool,
) {
    let file_processed = processed_files.iter().any(|s| s.contains(file_name));
    if should_be_processed {
        assert!(
            file_processed,
            "Expected file '{}' to be processed, but it was not.",
            file_name
        );
    } else {
        assert!(
            !file_processed,
            "Expected file '{}' not to be processed, but it was.",
            file_name
        );
    }
}
