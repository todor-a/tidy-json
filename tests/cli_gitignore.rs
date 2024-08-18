use std::path::PathBuf;

use assert_cmd::prelude::*;
use tempfile::TempDir;

pub mod common;

pub fn setup_gitnore_test_dir() -> (TempDir, PathBuf, PathBuf) {
    let temp_dir = common::setup_test_directory();
    let temp_path = temp_dir.path();

    common::init_git_repo(temp_path);

    common::create_file(temp_path.join(".gitignore").as_path(), "ignored.json");

    let sample_path = temp_path.join("sample.json");
    let ignored_path = temp_path.join("ignored.json");

    common::create_file(&sample_path, common::UNSORTED_JSON);
    common::create_file(&ignored_path, common::UNSORTED_JSON);

    (temp_dir, sample_path, ignored_path)
}

#[test]
fn test_gitignored_files_successfuly_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, sample_path, ignored_path) = setup_gitnore_test_dir();

    let mut cmd = common::run_cli("**/*.json", &["--write"], temp_dir.path());

    let output = cmd.assert().success().get_output().stdout.clone();

    let processed_files = common::extract_processed_files(&output);

    common::assert_file_processed(&processed_files, "./sample.json", true);
    common::assert_file_processed(&processed_files, "./ignored.json", false);

    common::assert_expected_processed_files_count(&processed_files, 1);

    common::assert_file_content(sample_path, true);
    common::assert_file_content(ignored_path, false);

    Ok(())
}
