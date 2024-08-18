use assert_cmd::prelude::*;
use insta::{assert_debug_snapshot, with_settings};
use rstest::rstest;
use serde_json::Value;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const UNSORTED_JSON_1: &str = r#"
{
    "c": 3,
    "b": 2,
    "a": 1
}"#;

const UNSORTED_JSON_2: &str = r#"
{
    "f": 3,
    "d": 2,
    "g": 1
}"#;

fn setup_test_directory() -> TempDir {
    TempDir::new().unwrap()
}

fn create_json_file(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
}

fn run_cli(include: &str, flags: &[&str], tmp_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("tidy-json").unwrap();
    cmd.arg(include).args(flags).current_dir(tmp_dir);
    cmd
}

fn setup_excludes_test() -> (TempDir, PathBuf, PathBuf) {
    let temp_dir = setup_test_directory();
    let temp_path = temp_dir.path();

    let sample_path = temp_path.join("sample.json");
    let ignored_path = temp_path.join("ignored.json");

    create_json_file(&sample_path, UNSORTED_JSON_1);
    create_json_file(&ignored_path, UNSORTED_JSON_2);

    (temp_dir, sample_path, ignored_path)
}

fn extract_processed_files(output: &[u8]) -> Vec<String> {
    let output_str = String::from_utf8(output.to_vec()).expect("Should be able to load output");

    output_str
        .lines()
        .filter(|line| line.contains(": Processed in"))
        .map(|line| line.split(':').next().unwrap_or("").to_string())
        .collect()
}

fn assert_expected_processed_files_count(processed_files: &[String], expected: usize) {
    assert_eq!(
        processed_files.len(),
        expected,
        "Expected only {} files to be processed",
        expected,
    );
}

fn assert_file_content<P: AsRef<Path>>(path: P, expect_sorted: bool) {
    let str = &fs::read_to_string(path).expect("Should have been able to read test file.");
    let content: Value = serde_json::from_str(str).expect("Should have been able to load content");

    with_settings!({
        description => if expect_sorted {"should be sorted"} else {"should not be sorted"},
    }, {
        assert_debug_snapshot!(content);
    });
}

fn assert_file_processed(processed_files: &[String], file_name: &str, should_be_processed: bool) {
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

#[test]
fn test_excludes_flag_ignores_specified_file() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, sample_path, ignored_path) = setup_excludes_test();

    let mut cmd = run_cli(
        "**/*.json",
        &["--write", "--exclude=**/ignored.json"],
        temp_dir.path(),
    );

    let output = cmd.assert().success().get_output().stdout.clone();

    let processed_files = extract_processed_files(&output);

    assert_file_processed(&processed_files, "./sample.json", true);
    assert_file_processed(&processed_files, "./ignored.json", false);

    assert_expected_processed_files_count(&processed_files, 1);

    assert_file_content(sample_path, true);
    assert_file_content(ignored_path, false);

    Ok(())
}

#[test]
fn test_excludes_flag_processes_non_excluded_files() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, sample_path, ignored_path) = setup_excludes_test();

    let mut cmd = run_cli(
        "**/*.json",
        &["--write", "--exclude=**/ignored.json"],
        temp_dir.path(),
    );

    let output = cmd.assert().success().get_output().stdout.clone();

    let processed_files = extract_processed_files(&output);

    assert_file_processed(&processed_files, "./sample.json", true);
    assert_file_processed(&processed_files, "./ignored.json", false);

    assert_expected_processed_files_count(&processed_files, 1);

    assert_file_content(sample_path, true);
    assert_file_content(ignored_path, false);

    Ok(())
}

#[test]
fn test_excludes_flag_does_not_modify_excluded_files() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, _, ignored_path) = setup_excludes_test();

    let original_content = fs::read_to_string(&ignored_path)?;

    run_cli(
        "**/*.json",
        &["--write", "--exclude=**/ignored.json"],
        temp_dir.path(),
    )
    .assert()
    .success();

    let post_run_content = fs::read_to_string(&ignored_path)?;
    assert_eq!(
        original_content, post_run_content,
        "Excluded file should not be modified"
    );

    Ok(())
}

#[rstest]
#[case("sample.json", "sample.json", &["--write"], true)]
#[case("./sample.json", "sample.json",&["--write"], true)]
fn test_write_flag_sorts_json(
    #[case] include_pattern: &str,
    #[case] test_file_name: &str,
    #[case] flags: &[&str],
    #[case] expect_sorted: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = setup_test_directory();
    let temp_path = temp_dir.path();

    create_json_file(temp_path.join(test_file_name).as_path(), UNSORTED_JSON_1);

    let mut cmd = run_cli(include_pattern, flags, temp_dir.as_ref());

    cmd.assert().success();

    assert_file_content(temp_path.join(include_pattern), expect_sorted);

    Ok(())
}
