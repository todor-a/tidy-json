use rstest::rstest;
use std::path::PathBuf;

use assert_cmd::prelude::*;
use insta::{assert_debug_snapshot, assert_json_snapshot, with_settings};
use tempfile::TempDir;

pub mod common;

pub fn setup_test_dir(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();

    let test_file_path = temp_path.join("sample.json");

    common::create_file(&test_file_path, content);

    (temp_dir, test_file_path)
}

#[test]
fn test_does_not_affect_spaces_in_values() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, test_file_path) = setup_test_dir(
        r#"
    {
        "b": "foo",
        "a": "1 2  3",
        "c": 3
    }
    "#,
    );

    let mut cmd = common::run_cli("**/*.json", &["--write"], temp_dir.path());

    let _ = cmd.assert().success().get_output().stdout.clone();

    let content = common::get_snapshot_info(
        test_file_path,
        common::AssertFileContentOption::Sorted(true),
    );

    assert_debug_snapshot!(content);

    Ok(())
}

#[rstest]
#[case(&["--write"])]
#[case(&["--write", "-i=5"])]
#[case(&["--write", "-i=3", "--indent-style=tabs"])]
#[case(&["--write", "-i=6", "--indent-style=spaces"])]
fn test_applies_correct_indent(#[case] flags: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, test_file_path) = setup_test_dir(
        r#"
    {
        "b": "foo",
        "a": "1 2  3",
        "c": 3
    }
    "#,
    );

    let mut cmd = common::run_cli("**/*.json", flags, temp_dir.path());

    let _ = cmd.assert().success().get_output().stdout.clone();

    let content = common::get_snapshot_info(
        test_file_path,
        common::AssertFileContentOption::Sorted(true),
    );

    with_settings!({
        description => flags.join(", "),
    }, {
        // todo: this is not resolving the indentation, find a way to do it
        assert_json_snapshot!(content);
    });

    Ok(())
}
