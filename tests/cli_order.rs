use assert_cmd::prelude::*;
use std::fs;

pub mod common;

#[test]
fn test_line_length_order_sorts_keys_by_key_length() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();
    let file_path = temp_path.join("sample.json");

    common::create_file(
        &file_path,
        r#"{
  "zebra": 3,
  "bb": 2,
  "a": 1,
  "cat": 4
}"#,
    );

    let mut cmd = common::run_cli("**/*.json", &["--write", "--order=line-length"], temp_path);
    cmd.assert().success();

    let content = fs::read_to_string(&file_path)?;
    let a = content.find("\"a\"").expect("a key missing");
    let bb = content.find("\"bb\"").expect("bb key missing");
    let cat = content.find("\"cat\"").expect("cat key missing");
    let zebra = content.find("\"zebra\"").expect("zebra key missing");

    assert!(a < bb);
    assert!(bb < cat);
    assert!(cat < zebra);

    Ok(())
}
