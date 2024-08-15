use clap::{CommandFactory, Parser, ValueEnum};
use colored::*;
use glob::{glob, GlobError};
use log::{debug, error, info};
use rand::seq::SliceRandom;
use rayon::prelude::*;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use thiserror::Error;

#[derive(Error, Debug)]
enum CustomError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Glob error: {0}")]
    GlobError(#[from] GlobError),
    #[error("{0}")]
    CustomError(String),
}

#[derive(Debug, Clone, ValueEnum)]
enum SortOrder {
    #[clap(name = "asc", alias = "alphabetical-asc", alias = "a")]
    AlphabeticalAsc,
    #[clap(name = "desc", alias = "alphabetical-desc", alias = "d")]
    AlphabeticalDesc,
    #[clap(name = "rand", alias = "random", alias = "r")]
    Random,
}

type Result<T> = std::result::Result<T, CustomError>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File patterns to process
    #[arg(required = true, help = "File patterns to process (e.g., *.json)")]
    patterns: Vec<String>,

    /// Write the sorted JSON back to the input files
    #[arg(short, long, default_value = "false")]
    write: bool,

    /// Create backups before modifying files
    #[arg(short, long, default_value = "false")]
    backup: bool,

    /// Specify the sort order
    #[arg(short = 'o', long, value_enum, default_value = "asc")]
    order: SortOrder,
}

fn print_error(err: &CustomError) {
    eprintln!("{} {}", "Error:".red().bold(), err);
    match err {
        CustomError::CustomError(msg)
            if msg == "No JSON files found matching the provided patterns" =>
        {
            eprintln!(
                "{}",
                "Make sure the file patterns are correct and the files exist.".yellow()
            );
            if let Err(err) = Args::command().print_help() {
                eprintln!("Failed to print help message: {}", err);
            }
        }
        _ => {
            if let Err(err) = Args::command().print_help() {
                eprintln!("Failed to print help message: {}", err);
            }
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();
    if let Err(e) = run() {
        print_error(&e);
        std::process::exit(1);
    } else {
        Ok(())
    }
}
fn run() -> Result<()> {
    let args = Args::parse();
    let start_time = Instant::now();

    if args.patterns.is_empty() {
        return Err(CustomError::CustomError(
            "No file patterns provided".to_string(),
        ));
    }

    let files: Vec<PathBuf> = args
        .patterns
        .iter()
        .flat_map(|pattern| {
            glob(pattern)
                .expect("Failed to read glob pattern")
                .filter_map(|entry| match entry {
                    Ok(path) if is_json_file(&path) => Some(path),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let results: Vec<_> = files
        .par_iter()
        .map(|path| {
            let file_start_time = Instant::now();
            let result = process_file(path, args.write, args.backup, &args.order);
            let duration = file_start_time.elapsed();
            (path, result, duration)
        })
        .collect();

    let processed_files: usize = results
        .iter()
        .filter(|(_, result, _)| result.is_ok())
        .count();
    let total_duration = start_time.elapsed();

    for (path, result, duration) in results {
        match result {
            Ok(_) => println!(
                "{}: Processed in {:.2?}",
                path.display().to_string().green(),
                duration
            ),
            Err(e) => eprintln!(
                "{}: {} (in {:.2?})",
                path.display().to_string().red(),
                e,
                duration
            ),
        }
    }

    println!(
        "{}",
        format!(
            "Processed {} file(s) in {:.2?}",
            processed_files, total_duration
        )
        .green()
    );

    Ok(())
}

fn is_json_file(path: &PathBuf) -> bool {
    path.extension()
        .map(|ext| ext == "json" || ext == "jsonc")
        .unwrap_or(false)
}

fn process_file(path: &PathBuf, write: bool, backup: bool, order: &SortOrder) -> Result<()> {
    let data = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&data)?;

    debug!("json {:?}", json);
    debug!("Usign sort order {:?}", order);

    let sorted_json = sort(&json, order);

    if write {
        if backup {
            let backup_path = path.with_extension("bak");
            fs::copy(path, &backup_path)?;
            info!("Backup created: {:?}", backup_path);
        }

        let indent = detect_indent(&data);

        debug!("Using indent {:?}", indent);

        let sorted_data = match indent {
            Some(indent_str) => {
                serde_json::to_string_pretty(&sorted_json)?.replace("  ", &indent_str)
            }
            None => serde_json::to_string_pretty(&sorted_json)?,
        };

        fs::write(path, sorted_data)?;
        info!("Sorted JSON written back to {:?}", path);
    } else {
        let sorted_data = serde_json::to_string_pretty(&sorted_json)?;
        info!("Sorted JSON for {:?}:\n{}", path, sorted_data);
    }

    Ok(())
}

fn sort(value: &Value, order: &SortOrder) -> Value {
    match value {
        Value::Object(map) => {
            let sorted_map: BTreeMap<_, _> = map.iter().collect();
            let mut entries: Vec<_> = sorted_map.into_iter().collect();

            debug!("hm? {:?}", entries);

            match order {
                SortOrder::AlphabeticalAsc => entries.sort_by(|(a, _), (b, _)| a.cmp(b)),
                SortOrder::AlphabeticalDesc => entries.sort_by(|(a, _), (b, _)| b.cmp(a)),
                SortOrder::Random => {
                    let mut rng = rand::thread_rng();
                    entries.shuffle(&mut rng);
                }
            }

            Value::Object(
                entries
                    .into_iter()
                    .map(|(k, v)| (k.clone(), sort(v, order)))
                    .collect(),
            )
        }
        Value::Array(arr) => Value::Array(arr.iter().map(|v| sort(v, order)).collect()),
        _ => value.clone(),
    }
}

fn detect_indent(json: &str) -> Option<String> {
    json.lines()
        .skip_while(|line| line.trim().is_empty())
        .find_map(|line| {
            let trimmed = line.trim_start();
            if !trimmed.is_empty() && line.len() > trimmed.len() {
                Some(line[..(line.len() - trimmed.len())].to_string())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn should_sort_descending() {
        let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "address": {
                "street": "123 Main St",
                "city": "Anytown"
            },
            "hobbies": ["reading", "cycling"]
        }"#;

        let json: Value = serde_json::from_str(data).unwrap();
        let sorted_json = sort(&json, &SortOrder::AlphabeticalDesc);
        assert_debug_snapshot!(sorted_json);
    }

    #[test]
    fn should_sort_correctly() {
        let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "address": {
                "street": "123 Main St",
                "city": "Anytown"
            },
            "hobbies": ["reading", "cycling"]
        }"#;

        let json: Value = serde_json::from_str(data).unwrap();
        let sorted_json = sort(&json, &SortOrder::AlphabeticalAsc);
        assert_debug_snapshot!(sorted_json);
    }

    #[test]
    fn should_apply_uniform_indent() {
        let data = r#"
        {
            "name": "John Doe",
          "age": 43,
            "address": {
             "street": "123 Main St",
                "city": "Anytown"
         },
            "hobbies": ["reading", "cycling"]
        }"#;

        let json: Value = serde_json::from_str(data).unwrap();
        let sorted_json = sort(&json, &SortOrder::AlphabeticalAsc);
        assert_debug_snapshot!(sorted_json);
    }

    #[test]
    fn test_detect_indent() {
        let json = r#"
{
    "key": "value",
    "nested": {
        "inner": "value"
    }
}"#;
        assert_eq!(detect_indent(json), Some("    ".to_string()));

        let json_tabs = r#"
{
	"key": "value",
	"nested": {
		"inner": "value"
	}
}"#;
        assert_eq!(detect_indent(json_tabs), Some("\t".to_string()));

        let json_no_indent = r#"{"key": "value"}"#;
        assert_eq!(detect_indent(json_no_indent), None);
    }

    #[test]
    fn test_is_json_file() {
        assert!(is_json_file(&PathBuf::from("test.json")));
        assert!(is_json_file(&PathBuf::from("test.jsonc")));
        assert!(!is_json_file(&PathBuf::from("test.txt")));
        assert!(!is_json_file(&PathBuf::from("test")));
    }
}
