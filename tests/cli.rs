use assert_cmd::prelude::*;
use insta::{assert_debug_snapshot, with_settings};
use predicates::prelude::*;
use serde_json::Value;
use std::fs::{self};
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn assert_file_content<P: AsRef<Path>>(path: P, expect_sorted: bool) {
    let str = &fs::read_to_string(path).expect("Should have been able to read test file.");
    let content: Value = serde_json::from_str(str).expect("Should have been able to load content");

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

    cmd.arg("sample.json")
        .arg("--write")
        .current_dir(temp_dir.as_ref());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processed 1 file(s) in"));

    assert_file_content(temp_path.join("sample.json"), true);

    Ok(())
}

#[test]
fn correctly_formats_single_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let temp_path = temp_dir.path();

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
    cmd.arg("./sample.json")
        .arg("--write")
        .current_dir(temp_dir.as_ref());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processed 1 file(s) in"));

    assert_file_content(temp_path.join("sample.json"), true);

    Ok(())
}

#[test]
fn correctly_applies_exludes() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let temp_path = temp_dir.path();

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
        .arg("--exclude=\"**/ignored.json\"")
        .current_dir(temp_dir.as_ref());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processed 1 file(s) in"));

    assert_file_content(temp_path.join("sample.json"), true);
    assert_file_content(temp_path.join("ignored.json"), false);

    Ok(())
}
