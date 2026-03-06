use assert_cmd::prelude::*;
use std::fs;

pub mod common;

#[test]
fn test_jsonc_file_is_processed() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();
    let file_path = temp_path.join("sample.jsonc");

    common::create_file(
        &file_path,
        r#"{
  // comment
  "b": 2,
  "a": 1,
}"#,
    );

    let mut cmd = common::run_cli("**/*.jsonc", &["--write"], temp_path);
    cmd.assert().success();

    let content = fs::read_to_string(&file_path)?;
    assert!(content.contains("\"a\": 1"));
    assert!(content.contains("\"b\": 2"));
    assert!(!content.contains("// comment"));

    Ok(())
}

#[test]
fn test_json_and_jsonc_patterns_work_together() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();
    let json_path = temp_path.join("sample.json");
    let jsonc_path = temp_path.join("sample.jsonc");

    common::create_file(
        &json_path,
        r#"{
  "z": 9,
  "a": 1
}"#,
    );
    common::create_file(
        &jsonc_path,
        r#"{
  "z": 9,
  "a": 1,
}"#,
    );

    let mut cmd = common::run_cli("**/*.json*", &["--write"], temp_path);
    let output = cmd.assert().success().get_output().stdout.clone();

    let processed_files = common::extract_processed_files(&output);
    common::assert_expected_processed_files_count(&processed_files, 2);

    let json_content = fs::read_to_string(&json_path)?;
    let jsonc_content = fs::read_to_string(&jsonc_path)?;

    assert!(json_content.find("\"a\"").expect("a") < json_content.find("\"z\"").expect("z"));
    assert!(jsonc_content.find("\"a\"").expect("a") < jsonc_content.find("\"z\"").expect("z"));

    Ok(())
}
