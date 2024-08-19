use assert_cmd::prelude::*;

pub mod common;

const DATA: &str = r#"
{
    "c": 3,
    "b": {
        "c": 1,
        "b": 5,
        "a": 2
    },
    "a": 1
}"#;

#[test]
fn test_sorts_correctly_without_depth_arg() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();

    common::create_file(temp_path.join("sample.json").as_path(), DATA);

    let mut cmd = common::run_cli("**/*.json", &["--write"], temp_dir.path());

    let _ = cmd.assert().success().get_output().stdout.clone();

    common::assert_file_content(temp_path.join("sample.json"), true);

    Ok(())
}

#[test]
fn test_sorts_correctly_with_depth_arg() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();

    common::create_file(temp_path.join("sample.json").as_path(), DATA);

    let mut cmd = common::run_cli("**/*.json", &["--write", "--depth=1"], temp_dir.path());

    let _ = cmd.assert().success().get_output().stdout.clone();

    common::assert_file_content(temp_path.join("sample.json"), true);

    Ok(())
}
