use assert_cmd::prelude::*;
use insta::{assert_debug_snapshot, with_settings};
use predicates::prelude::*;
use serde_json::Value;
use std::fs::{self};
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn assert_file_content<P: AsRef<Path>>(path: P, expect_sorted: bool) {
    let content: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();

    with_settings!({
        description => if expect_sorted {"should be sorted"} else {"should not be sorted"},
    }, {
        assert_debug_snapshot!(content);
    });
}

#[test]
fn find_content_in_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp_path).unwrap();

    fs::write(
        temp_path.join("sample.json"),
        r#"
    {
        "c": 3,
        "b": 2,
        "a": 1
    }"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("tidy-json")?;
    cmd.arg("sample.json").arg("--write");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processed 1 file(s) in"));

    assert_file_content(temp_path.join("sample.json"), true);

    std::env::set_current_dir(original_dir).unwrap();

    Ok(())
}

#[test]
fn correctly_formats_single_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp_path).unwrap();

    fs::write(
        temp_path.join("sample.json"),
        r#"
    {
        "c": 3,
        "b": 2,
        "a": 1
    }"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("tidy-json")?;
    cmd.arg("./sample.json").arg("--write");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processed 1 file(s) in"));

    assert_file_content(temp_path.join("sample.json"), true);

    std::env::set_current_dir(original_dir).unwrap();

    Ok(())
}

#[test]
fn correctly_applies_exludes() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let original_dir = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp_path).unwrap();

    fs::write(
        temp_path.join("sample.json"),
        r#"
    {
        "c": 3,
        "b": 2,
        "a": 1
    }"#,
    )
    .unwrap();

    fs::write(
        temp_path.join("ignored.json"),
        r#"
    {
        "f": 3,
        "d": 2,
        "g": 1
    }"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("tidy-json")?;
    cmd.arg("**/*.json")
        .arg("--write")
        .arg("--exclude=\"**/ignored.json\"");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processed 1 file(s) in"));

    assert_file_content(temp_path.join("sample.json"), true);
    assert_file_content(temp_path.join("ignored.json"), false);

    std::env::set_current_dir(original_dir).unwrap();

    Ok(())
}
