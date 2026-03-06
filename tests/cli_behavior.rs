use assert_cmd::prelude::*;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

pub mod common;

#[test]
fn test_check_mode_fails_when_file_needs_formatting() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();
    let file_path = temp_path.join("sample.json");
    common::create_file(&file_path, common::UNSORTED_JSON);

    let mut cmd = common::run_cli("**/*.json", &["--check"], temp_path);
    cmd.assert()
        .code(2)
        .stdout(predicate::str::contains("needs formatting"));

    Ok(())
}

#[test]
fn test_check_mode_passes_when_file_is_formatted() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();
    let file_path = temp_path.join("sample.json");
    common::create_file(
        &file_path,
        r#"{
 "a": 1,
 "b": 2,
 "c": 3
}"#,
    );

    let mut cmd = common::run_cli("**/*.json", &["--check"], temp_path);
    cmd.assert().success();

    Ok(())
}

#[test]
fn test_stdout_prints_sorted_json_without_writing() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();
    let file_path = temp_path.join("sample.json");
    let original = common::UNSORTED_JSON;
    common::create_file(&file_path, original);

    let mut cmd = common::run_cli("**/*.json", &["--stdout"], temp_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"a\": 1"));

    let content = fs::read_to_string(&file_path)?;
    assert_eq!(content, original);

    Ok(())
}

#[test]
fn test_indent_style_tabs_applies_without_indent_width() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();
    let file_path = temp_path.join("sample.json");
    common::create_file(
        &file_path,
        r#"{
  "b": 2,
  "a": 1
}"#,
    );

    let mut cmd = common::run_cli("**/*.json", &["--write", "--indent-style=tabs"], temp_path);
    cmd.assert().success();

    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("\n\t\"a\": 1"));

    Ok(())
}

#[test]
fn test_backup_creates_bak_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();
    let file_path = temp_path.join("sample.json");
    common::create_file(&file_path, common::UNSORTED_JSON);

    let mut cmd = common::run_cli("**/*.json", &["--write", "--backup"], temp_path);
    cmd.assert().success();

    let backup_path = temp_path.join("sample.bak");
    assert!(backup_path.exists());

    Ok(())
}

#[test]
fn test_no_matching_files_returns_error() {
    let temp_dir = common::setup_test_directory();
    let mut cmd = common::run_cli("**/*.json", &[], temp_dir.path());
    cmd.assert().failure().stderr(predicate::str::contains(
        "No JSON files found matching the provided patterns",
    ));
}

#[test]
fn test_log_level_quiet_suppresses_success_output() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();
    let file_path = temp_path.join("sample.json");
    common::create_file(&file_path, common::UNSORTED_JSON);

    let mut cmd = common::run_cli("**/*.json", &["--write", "--log-level=quiet"], temp_path);
    cmd.assert().success().stdout(predicate::str::is_empty());

    Ok(())
}

#[test]
fn test_stdin_mode_reads_and_writes_stdout() {
    let temp_dir = common::setup_test_directory();
    let mut cmd = Command::cargo_bin("tidy-json").expect("binary should build");
    cmd.current_dir(temp_dir.path()).arg("--stdin");
    cmd.write_stdin(common::UNSORTED_JSON)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"a\": 1"));
}

#[test]
fn test_config_file_defaults_are_applied() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();
    let file_path = temp_path.join("sample.json");
    common::create_file(&file_path, common::UNSORTED_JSON);

    common::create_file(
        &temp_path.join(".tidy-json.toml"),
        r#"write = true
order = "desc"
indent = 2
"#,
    );

    let mut cmd = common::run_cli("**/*.json", &[], temp_path);
    cmd.assert().success();

    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("\"c\": 3"));
    let c_pos = content.find("\"c\"").expect("c key should exist");
    let a_pos = content.find("\"a\"").expect("a key should exist");
    assert!(c_pos < a_pos);

    Ok(())
}

#[test]
fn test_trailing_comma_is_parsed_without_panic() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();
    let file_path = temp_path.join("sample.json");

    common::create_file(
        &file_path,
        r#"{
  "b": 2,
  "a": 1,
}"#,
    );

    let mut cmd = common::run_cli("**/*.json", &["--write"], temp_path);
    cmd.assert().success();

    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("\"a\": 1"));
    assert!(content.contains("\"b\": 2"));
    assert!(!content.contains(",\n}"));

    Ok(())
}
