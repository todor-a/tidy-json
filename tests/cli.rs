use assert_cmd::prelude::*;
use insta::{assert_debug_snapshot, with_settings};
use rstest::rstest;
use std::fs::{self};

pub mod common;

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

    let (content, description) =
        common::get_snapshot_info(sample_path, common::AssertFileContentOption::Sorted(true));
    with_settings!({
        description => description,
    }, {
        assert_debug_snapshot!(content);
    });

    let (content, description) =
        common::get_snapshot_info(ignored_path, common::AssertFileContentOption::Sorted(false));

    with_settings!({
        description => description,
    }, {
        assert_debug_snapshot!(content);
    });

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

    let (content, description) =
        common::get_snapshot_info(sample_path, common::AssertFileContentOption::Sorted(true));
    with_settings!({
        description => description,
    }, {
        assert_debug_snapshot!(content);
    });

    let (content, description) =
        common::get_snapshot_info(ignored_path, common::AssertFileContentOption::Sorted(false));
    with_settings!({
        description => description,
    }, {
        assert_debug_snapshot!(content);
    });

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

    common::create_file(
        temp_path.join(test_file_name).as_path(),
        common::UNSORTED_JSON,
    );

    let mut cmd = common::run_cli(include_pattern, flags, temp_dir.as_ref());

    cmd.assert().success();

    let (content, description) = common::get_snapshot_info(
        temp_path.join(include_pattern),
        common::AssertFileContentOption::Sorted(expect_sorted),
    );

    with_settings!({
        description => description,
    }, {
        assert_debug_snapshot!(content);
    });

    Ok(())
}
