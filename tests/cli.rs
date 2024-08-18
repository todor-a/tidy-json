use assert_cmd::prelude::*;
use rstest::rstest;
use std::fs::{self};

mod common;

#[test]
fn test_excludes_flag_ignores_specified_file() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, sample_path, ignored_path) = common::setup_excludes_test();

    let mut cmd = common::run_cli(
        "**/*.json",
        &["--write", "--exclude=**/ignored.json"],
        temp_dir.path(),
    );

    let output = cmd.assert().success().get_output().stdout.clone();

    let processed_files = common::extract_processed_files(&output);

    common::assert_file_processed(&processed_files, "./sample.json", true);
    common::assert_file_processed(&processed_files, "./ignored.json", false);

    common::assert_expected_processed_files_count(&processed_files, 1);

    common::assert_file_content(sample_path, true);
    common::assert_file_content(ignored_path, false);

    Ok(())
}

#[test]
fn test_excludes_flag_processes_non_excluded_files() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, sample_path, ignored_path) = common::setup_excludes_test();

    let mut cmd = common::run_cli(
        "**/*.json",
        &["--write", "--exclude=**/ignored.json"],
        temp_dir.path(),
    );

    let output = cmd.assert().success().get_output().stdout.clone();

    let processed_files = common::extract_processed_files(&output);

    common::assert_file_processed(&processed_files, "./sample.json", true);
    common::assert_file_processed(&processed_files, "./ignored.json", false);

    common::assert_expected_processed_files_count(&processed_files, 1);

    common::assert_file_content(sample_path, true);
    common::assert_file_content(ignored_path, false);

    Ok(())
}

#[test]
fn test_excludes_flag_does_not_modify_excluded_files() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, _, ignored_path) = common::setup_excludes_test();

    let original_content = fs::read_to_string(&ignored_path)?;

    common::run_cli(
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
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();

    common::create_json_file(
        temp_path.join(test_file_name).as_path(),
        common::UNSORTED_JSON_1,
    );

    let mut cmd = common::run_cli(include_pattern, flags, temp_dir.as_ref());

    cmd.assert().success();

    common::assert_file_content(temp_path.join(include_pattern), expect_sorted);

    Ok(())
}
