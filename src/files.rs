use std::path::{Path, PathBuf};

use glob::Pattern;
use ignore::WalkBuilder;

pub fn list_files<P: AsRef<Path>>(
    root: P,
    include_patterns: Vec<String>,
    exclude_patterns: Option<Vec<String>>,
    ignore_git_ignore: bool,
) -> Vec<PathBuf> {
    include_patterns
        .iter()
        .flat_map(|pattern| {
            WalkBuilder::new(root.as_ref())
                .hidden(false)
                .git_ignore(!ignore_git_ignore)
                .build()
                .filter_map(std::result::Result::ok)
                .filter(|entry| {
                    let path = entry.path();
                    let relative_path = path.strip_prefix(root.as_ref()).unwrap();
                    is_json_file(&path.to_path_buf())
                        && Pattern::new(pattern).unwrap().matches_path(relative_path)
                        && !is_excluded(relative_path, &exclude_patterns)
                })
                .map(|entry| entry.path().to_path_buf())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn is_excluded(path: &Path, exclude_patterns: &Option<Vec<String>>) -> bool {
    match exclude_patterns {
        Some(patterns) => patterns
            .iter()
            .any(|pattern| Pattern::new(pattern).unwrap().matches_path(path)),
        None => false,
    }
}

pub fn is_json_file(path: &PathBuf) -> bool {
    path.extension()
        .map(|ext| ext == "json" || ext == "jsonc")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    #[test]
    fn test_is_json_file() {
        assert!(is_json_file(&PathBuf::from("test.json")));
        assert!(is_json_file(&PathBuf::from("test.jsonc")));
        assert!(!is_json_file(&PathBuf::from("test.txt")));
        assert!(!is_json_file(&PathBuf::from("test")));
    }

    #[test]
    fn test_list_files() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        std::process::Command::new("git")
            .args(&["init"])
            .current_dir(temp_path)
            .output()
            .expect("Failed to initialize git repo");

        fs::write(temp_path.join(".gitignore"), "ignored.json\n").unwrap();

        File::create(temp_path.join("test1.json")).unwrap();
        File::create(temp_path.join("test2.jsonc")).unwrap();
        File::create(temp_path.join("test3.txt")).unwrap();
        File::create(temp_path.join("ignored.json")).unwrap();

        let subdir = temp_path.join("subdir");
        fs::create_dir(&subdir).unwrap();
        File::create(subdir.join("test4.json")).unwrap();
        File::create(subdir.join("test5.jsonc")).unwrap();

        let files = list_files(
            temp_path,
            vec!["**/*.json".to_string(), "**/*.jsonc".to_string()],
            None,
            false,
        );

        assert_eq!(files.len(), 4);
        assert!(files.contains(&temp_path.join("test1.json")));
        assert!(files.contains(&temp_path.join("test2.jsonc")));
        assert!(files.contains(&temp_path.join("subdir/test4.json")));
        assert!(files.contains(&temp_path.join("subdir/test5.jsonc")));
        assert!(!files.contains(&temp_path.join("ignored.json")));

        let files = list_files(
            temp_path,
            vec!["**/*.json".to_string(), "**/*.jsonc".to_string()],
            None,
            true,
        );

        assert_eq!(files.len(), 5);
        assert!(files.contains(&temp_path.join("ignored.json")));

        let files = list_files(temp_path, vec!["**/test*.json".to_string()], None, false);
        assert_eq!(files.len(), 2);
        assert!(files.contains(&temp_path.join("test1.json")));
        assert!(files.contains(&temp_path.join("subdir/test4.json")));

        let files = list_files(temp_path, vec!["**/ignored.json".to_string()], None, false);
        assert_eq!(files.len(), 0);

        let files = list_files(
            temp_path,
            vec!["**/*.json".to_string(), "**/*.jsonc".to_string()],
            Some(vec!["**/test2*".to_string()]),
            false,
        );
        assert_eq!(files.len(), 3);
        assert!(files.contains(&temp_path.join("test1.json")));
        assert!(!files.contains(&temp_path.join("test2.jsonc")));
        assert!(files.contains(&temp_path.join("subdir/test4.json")));
        assert!(files.contains(&temp_path.join("subdir/test5.jsonc")));
        assert!(!files.contains(&temp_path.join("ignored.json")));

        let files = list_files(
            temp_path,
            vec!["**/*.json".to_string(), "**/*.jsonc".to_string()],
            Some(vec!["**/test2*".to_string()]),
            true,
        );
        assert_eq!(files.len(), 4);
        assert!(files.contains(&temp_path.join("test1.json")));
        assert!(!files.contains(&temp_path.join("test2.jsonc")));
        assert!(files.contains(&temp_path.join("subdir/test4.json")));
        assert!(files.contains(&temp_path.join("subdir/test5.jsonc")));
        assert!(files.contains(&temp_path.join("ignored.json")));
    }
}
