use clap::{CommandFactory, Parser, ValueEnum};
use colored::*;
use glob::GlobError;
use log::{error, info, LevelFilter};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use thiserror::Error;

mod files;
mod sort;

#[derive(Error, Debug)]
enum CustomError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Glob error: {0}")]
    Glob(#[from] GlobError),
    #[error("{0}")]
    Custom(String),
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SortOrder {
    #[clap(name = "asc", alias = "alphabetical-asc", alias = "a")]
    AlphabeticalAsc,
    #[clap(name = "desc", alias = "alphabetical-desc", alias = "d")]
    AlphabeticalDesc,
    #[clap(name = "rand", alias = "random", alias = "r")]
    Random,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum IndentStyle {
    #[clap()]
    Tabs,
    #[clap()]
    Spaces,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum LogLevel {
    #[clap()]
    Quiet,
    #[clap()]
    Default,
    #[clap()]
    Verbose,
}

impl LogLevel {
    pub fn to_level_filter(&self) -> LevelFilter {
        match self {
            LogLevel::Quiet => LevelFilter::Error,
            LogLevel::Default => LevelFilter::Error,
            LogLevel::Verbose => LevelFilter::Debug,
        }
    }
}

type Result<T> = std::result::Result<T, CustomError>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File patterns to process
    #[arg(required = true, help = "File patterns to process (e.g., *.json)")]
    include: Vec<PathBuf>,

    /// File patterns to exclude
    #[arg(short, long, help = "File patterns to exclude (e.g., *.json)")]
    exclude: Option<Vec<PathBuf>>,

    /// Write the sorted JSON back to the input files
    #[arg(short, long, default_value = "false")]
    write: bool,

    /// Create backups before modifying files
    #[arg(short, long, default_value = "false")]
    backup: bool,

    /// Specify how deep the sorting should go
    #[arg(short, long)]
    depth: Option<u32>,

    /// Specify the sort order
    #[arg(short = 'o', long, value_enum, default_value = "asc")]
    order: SortOrder,

    /// Specify the desired indent
    #[arg(short, long)]
    indent: Option<usize>,

    /// Specify the desired indent style
    #[arg(long)]
    indent_style: Option<IndentStyle>,

    /// Specify the log level
    #[arg(value_enum, long, default_value_t=LogLevel::Default)]
    log_level: LogLevel,
}

#[derive(Debug)]
struct Configuration {
    include: Vec<PathBuf>,
    exclude: Option<Vec<PathBuf>>,
    write: bool,
    backup: bool,
    order: SortOrder,
    depth: Option<u32>,
    indent: Option<usize>,
    indent_style: Option<IndentStyle>,
}

fn print_error(err: &CustomError) {
    error!("{} {}", "Error:".red().bold(), err);
    match err {
        CustomError::Custom(msg) if msg == "No JSON files found matching the provided patterns" => {
            error!(
                "{}",
                "Make sure the file patterns are correct and the files exist.".yellow()
            );
            if let Err(err) = Args::command().print_help() {
                error!("Failed to print help message: {}", err);
            }
        }
        _ => {
            if let Err(err) = Args::command().print_help() {
                error!("Failed to print help message: {}", err);
            }
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    env_logger::builder()
        .filter_level(args.log_level.to_level_filter())
        .init();
    if let Err(e) = run(args) {
        print_error(&e);
        std::process::exit(1);
    } else {
        Ok(())
    }
}
fn run(args: Args) -> Result<()> {
    let start_time = Instant::now();

    if args.include.is_empty() {
        return Err(CustomError::Custom(
            "No include file patterns provided".to_string(),
        ));
    }

    let cfg = Configuration {
        backup: args.backup,
        depth: args.depth,
        exclude: args.exclude,
        include: args.include,
        indent: args.indent,
        order: args.order,
        write: args.write,
        indent_style: args.indent_style,
    };

    let files =
        files::list_files(&cfg.include, &cfg.exclude, vec![files::Extension::Json]).unwrap();

    let results: Vec<_> = files
        .par_iter()
        .map(|path| {
            let file_start_time = Instant::now();
            let result = process_file(path, &cfg);
            let duration = file_start_time.elapsed();
            (path, result, duration)
        })
        .collect();

    let processed_files: usize = results
        .iter()
        .filter(|(_, result, _)| result.is_ok())
        .count();

    for (path, result, duration) in results {
        match result {
            Ok(_) => println!(
                "{}: Processed in {:.2?}",
                path.display().to_string().green(),
                duration
            ),
            Err(e) => error!(
                "{}: {} (in {:.2?})",
                path.display().to_string().red(),
                e,
                duration
            ),
        }
    }

    let total_duration = start_time.elapsed();

    info!(
        "{}",
        format!(
            "Processed {} file(s) in {:.2?}",
            processed_files, total_duration
        )
        .green()
    );

    Ok(())
}

fn process_file(path: &PathBuf, cfg: &Configuration) -> Result<()> {
    let data = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&data)?;

    let sorted_json = sort::sort(&json, &cfg.order, 0, cfg.depth);

    if cfg.write {
        if cfg.backup {
            let backup_path = path.with_extension("bak");
            fs::copy(path, &backup_path)?;
            info!("Backup created: {:?}", backup_path);
        }

        let indent = get_indent(cfg, &data);
        let formatted_json = format_json(&sorted_json, &indent);
        fs::write(path, formatted_json)?;

        info!("Sorted JSON written back to {:?}", path);
    } else {
        serde_json::to_string_pretty(&sorted_json)?;
        info!("Sorted {:?}.", path);
    }

    Ok(())
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

fn get_indent(cfg: &Configuration, data: &str) -> String {
    cfg.indent
        .map(|indent| match cfg.indent_style {
            Some(IndentStyle::Tabs) => "\t".repeat(indent),
            _ => " ".repeat(indent),
        })
        .unwrap_or_else(|| detect_indent(data).unwrap_or_else(|| " ".to_string()))
}

fn format_json(value: &serde_json::Value, indent: &str) -> String {
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    value.serialize(&mut ser).unwrap();
    String::from_utf8(buf).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
