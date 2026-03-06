use assert_cmd::prelude::*;
use serde_json::Value;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[derive(serde::Serialize)]
pub struct Info {
    pub used_flags: Vec<&'static str>,
}

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
        .filter_map(|line| {
            line.split_once(": Processed in")
                .map(|(path, _)| normalize_path_for_match(path))
        })
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

pub enum AssertFileContentOption {
    Sorted(bool),
    Description(String),
}

pub fn get_snapshot_info<P: AsRef<Path>>(
    path: P,
    option: AssertFileContentOption,
) -> (Value, String) {
    let str = &fs::read_to_string(path).expect("Should have been able to read test file.");
    let content: Value = serde_json::from_str(str).expect("Should have been able to load content");

    let description = match option {
        AssertFileContentOption::Description(desc) => desc,
        AssertFileContentOption::Sorted(true) => "should be sorted".to_string(),
        AssertFileContentOption::Sorted(false) => "should not be sorted".to_string(),
    };

    (content, description)
}

pub fn assert_file_processed(
    processed_files: &[String],
    file_name: &str,
    should_be_processed: bool,
) {
    let normalized_file_name = normalize_path_for_match(file_name);
    let file_processed = processed_files
        .iter()
        .any(|s| s.contains(&normalized_file_name));
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

fn normalize_path_for_match(path: &str) -> String {
    path.replace('\\', "/")
}
