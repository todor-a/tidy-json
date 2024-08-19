use assert_cmd::prelude::*;
use insta::{assert_debug_snapshot, with_settings};

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

    let (content, description) = common::get_snapshot_info(
        temp_path.join("sample.json"),
        common::AssertFileContentOption::Description("should be sorted on all levels".to_string()),
    );

    with_settings!({
        description => description,
    }, {
        assert_debug_snapshot!(content);
    });

    Ok(())
}

#[test]
fn test_sorts_correctly_with_depth_arg() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();

    common::create_file(temp_path.join("sample.json").as_path(), DATA);

    let mut cmd = common::run_cli("**/*.json", &["--write", "--depth=1"], temp_dir.path());

    let _ = cmd.assert().success().get_output().stdout.clone();

    let (content, description) = common::get_snapshot_info(
        temp_path.join("sample.json"),
        common::AssertFileContentOption::Description(
            "should be sorted only on level 1".to_string(),
        ),
    );

    with_settings!({
        description => description,
    }, {
        assert_debug_snapshot!(content);
    });

    Ok(())
}
