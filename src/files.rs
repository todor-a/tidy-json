use anyhow::*;
use glob::{MatchOptions, Pattern};
use ignore::WalkBuilder;
#[cfg(not(test))]
use log::debug;
use std::path::{Path, PathBuf};

#[cfg(test)]
use std::println as debug;

#[derive(Debug, PartialEq)]
pub enum Extension {
    Json,
}

impl Extension {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "json",
        }
    }
}

fn create_patterns(patterns: Vec<PathBuf>) -> Result<Vec<Pattern>> {
    patterns
        .into_iter()
        .map(|path| {
            let pattern_str = path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid pattern path"))?
                .trim_matches('"'); // Remove surrounding quotes if present
            Pattern::new(pattern_str).map_err(|e| e.into())
        })
        .collect()
}

fn matches_patterns(patterns: &[Pattern], path: &Path) -> bool {
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    patterns
        .iter()
        .any(|pattern| pattern.matches_path_with(path, options))
}

fn matches_exclude_patterns(patterns: &[Pattern], path: &Path) -> bool {
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    patterns.iter().any(|pattern| {
        let path_str = path.to_str().unwrap_or("");
        pattern.matches_with(path_str, options)
    })
}

fn has_allowed_extension(path: &Path, allowed_extensions: &[Extension]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| allowed_extensions.iter().any(|e| e.as_str() == ext))
        .unwrap_or(false)
}

pub fn list_files(
    include_patterns: Vec<PathBuf>,
    exclude_patterns: Option<Vec<PathBuf>>,
    allowed_extensions: Vec<Extension>,
) -> Result<Vec<PathBuf>> {
    let mut walk = WalkBuilder::new(".");
    walk.hidden(true).ignore(true).git_global(true);

    let include_patterns: Vec<Pattern> = create_patterns(include_patterns)?;
    let exclude_patterns: Vec<Pattern> = exclude_patterns
        .map(create_patterns)
        .transpose()?
        .unwrap_or_default();

    let mut matching_files = Vec::new();

    for entry in walk.build() {
        let entry = entry.context("Failed to read directory entry")?;
        if entry.file_type().map_or(false, |ft| ft.is_file()) {
            let path = entry.path();
            let relative_path = path.strip_prefix(".").unwrap_or(path);

            // Create both versions of the path for matching
            let relative_path_with_dot = PathBuf::from(".").join(relative_path);

            if (matches_patterns(&include_patterns, &relative_path_with_dot)
                || matches_patterns(&include_patterns, relative_path))
                && !(matches_exclude_patterns(&exclude_patterns, &relative_path_with_dot)
                    || matches_exclude_patterns(&exclude_patterns, relative_path))
                && has_allowed_extension(relative_path, &allowed_extensions)
            {
                matching_files.push(path.to_path_buf());
            }
        }
    }

    debug!("Found {:?} files.", matching_files);

    Ok(matching_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    #[test]
    fn test_matches_patterns() {
        let patterns = vec![Pattern::new("./test1.json").unwrap()];

        let matching_path = Path::new("test1.json");
        assert!(!matches_patterns(&patterns, matching_path));

        let matching_path = Path::new("./test1.json");
        assert!(matches_patterns(&patterns, matching_path));

        let non_matching_path = Path::new("test2.json");
        assert!(!matches_patterns(&patterns, non_matching_path));

        let subdirectory_path = Path::new("subdir/test1.json");
        assert!(!matches_patterns(&patterns, subdirectory_path));
    }

    #[test]
    fn test_list_files() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_path).unwrap();

        std::process::Command::new("git")
            .args(["init"])
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
            vec![PathBuf::from("**/*.json")],
            None,
            vec![Extension::Json],
        )
        .unwrap();

        debug!("result {:?}, a {:?}", files, &temp_path.join("test1.json"));

        assert_eq!(files.len(), 2);
        assert!(files.contains(&PathBuf::from("./test1.json")));
        assert!(files.contains(&PathBuf::from("./subdir/test4.json")));

        assert!(!files.contains(&PathBuf::from("./test2.jsonc")));
        assert!(!files.contains(&PathBuf::from("./test3.txt")));
        assert!(!files.contains(&PathBuf::from("./ignored.json")));
        assert!(!files.contains(&PathBuf::from("./subdir/test5.jsonc")));

        // test single file
        File::create(temp_path.join("foo.json")).unwrap();
        File::create(temp_path.join("bar.json")).unwrap();

        let files = list_files(
            vec![PathBuf::from("./foo.json")],
            Some(vec![PathBuf::from("./test2bar.json")]),
            vec![Extension::Json],
        )
        .unwrap();

        debug!("result {:?}, a {:?}", files, &temp_path.join("foo.json"));

        assert_eq!(files.len(), 1);
        assert!(files.contains(&PathBuf::from("./foo.json")));
        assert!(!files.contains(&PathBuf::from("./bar.json")));

        std::env::set_current_dir(original_dir).unwrap();
    }
}
