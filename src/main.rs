use clap::{CommandFactory, Parser, ValueEnum};
use colored::*;
use log::{error, info, LevelFilter};
use rayon::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::Instant;
use thiserror::Error;

use tidy_json::sort;
use tidy_json::SortOrder;

mod files;

#[derive(Error, Debug)]
enum CustomError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to read config: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("{0} file(s) need formatting")]
    CheckFailed(usize),
    #[error("{0}")]
    Custom(String),
}

#[derive(Debug, Clone, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndentStyle {
    #[clap()]
    Tabs,
    #[clap()]
    Spaces,
}

#[derive(Debug, Clone, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
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
            LogLevel::Verbose => LevelFilter::Info,
        }
    }
}

type Result<T> = std::result::Result<T, CustomError>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File patterns to process
    #[arg(help = "File patterns to process (e.g., *.json)")]
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

    /// Check if files would change without writing them
    #[arg(long, default_value = "false")]
    check: bool,

    /// Specify how deep the sorting should go
    #[arg(short, long)]
    depth: Option<u32>,

    /// Specify the sort order
    #[arg(short = 'o', long, value_enum)]
    order: Option<SortOrder>,

    /// Specify the desired indent
    #[arg(short, long)]
    indent: Option<usize>,

    /// Specify the desired indent style
    #[arg(long)]
    indent_style: Option<IndentStyle>,

    /// Specify the log level
    #[arg(value_enum, long)]
    log_level: Option<LogLevel>,

    /// Read input from stdin instead of files
    #[arg(long, default_value = "false")]
    stdin: bool,

    /// Print sorted output to stdout
    #[arg(long, default_value = "false")]
    stdout: bool,

    /// Path to a TOML config file
    #[arg(long)]
    config: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    include: Option<Vec<PathBuf>>,
    exclude: Option<Vec<PathBuf>>,
    write: Option<bool>,
    backup: Option<bool>,
    check: Option<bool>,
    order: Option<String>,
    depth: Option<u32>,
    indent: Option<usize>,
    indent_style: Option<String>,
    log_level: Option<String>,
    stdin: Option<bool>,
    stdout: Option<bool>,
}

#[derive(Debug)]
struct Configuration {
    include: Vec<PathBuf>,
    exclude: Option<Vec<PathBuf>>,
    write: bool,
    backup: bool,
    check: bool,
    order: SortOrder,
    depth: Option<u32>,
    indent: Option<usize>,
    indent_style: Option<IndentStyle>,
    log_level: LogLevel,
    stdin: bool,
    stdout: bool,
}

#[derive(Debug)]
struct ProcessOutcome {
    changed: bool,
    output: Option<String>,
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
        CustomError::Custom(msg) if msg == "No include file patterns provided" => {
            if let Err(err) = Args::command().print_help() {
                error!("Failed to print help message: {}", err);
            }
        }
        CustomError::CheckFailed(_) => {}
        _ => {
            error!("Run with --help for usage information.");
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let cfg = resolve_configuration(args)?;
    env_logger::builder()
        .filter_level(cfg.log_level.to_level_filter())
        .init();
    if let Err(e) = run(&cfg) {
        print_error(&e);
        match e {
            CustomError::CheckFailed(_) => std::process::exit(2),
            _ => std::process::exit(1),
        }
    } else {
        Ok(())
    }
}

fn resolve_configuration(args: Args) -> Result<Configuration> {
    let file_cfg = load_config(args.config.as_ref())?;

    let include = if args.include.is_empty() {
        file_cfg.include.unwrap_or_default()
    } else {
        args.include
    };
    let exclude = args.exclude.or(file_cfg.exclude);

    let order = match args.order {
        Some(order) => order,
        None => parse_sort_order(file_cfg.order.as_deref())?.unwrap_or(SortOrder::AlphabeticalAsc),
    };
    let indent_style = match args.indent_style {
        Some(indent_style) => Some(indent_style),
        None => parse_indent_style(file_cfg.indent_style.as_deref())?,
    };
    let log_level = match args.log_level {
        Some(log_level) => log_level,
        None => parse_log_level(file_cfg.log_level.as_deref())?.unwrap_or(LogLevel::Default),
    };

    let cfg = Configuration {
        backup: args.backup || file_cfg.backup.unwrap_or(false),
        check: args.check || file_cfg.check.unwrap_or(false),
        depth: args.depth.or(file_cfg.depth),
        exclude,
        include,
        indent: args.indent.or(file_cfg.indent),
        order,
        write: args.write || file_cfg.write.unwrap_or(false),
        indent_style,
        log_level,
        stdin: args.stdin || file_cfg.stdin.unwrap_or(false),
        stdout: args.stdout || file_cfg.stdout.unwrap_or(false),
    };

    validate_configuration(&cfg)?;

    Ok(cfg)
}

fn load_config(path: Option<&PathBuf>) -> Result<FileConfig> {
    let config_path = match path {
        Some(path) => Some(path.clone()),
        None => {
            let default_path = PathBuf::from(".tidy-json.toml");
            if default_path.exists() {
                Some(default_path)
            } else {
                None
            }
        }
    };

    match config_path {
        Some(path) => {
            let content = fs::read_to_string(path)?;
            Ok(toml::from_str(&content)?)
        }
        None => Ok(FileConfig::default()),
    }
}

fn parse_sort_order(value: Option<&str>) -> Result<Option<SortOrder>> {
    value
        .map(|v| {
            SortOrder::from_str(v, true)
                .map_err(|_| CustomError::Custom(format!("Invalid sort order in config: {v}")))
        })
        .transpose()
}

fn parse_indent_style(value: Option<&str>) -> Result<Option<IndentStyle>> {
    value
        .map(|v| {
            IndentStyle::from_str(v, true)
                .map_err(|_| CustomError::Custom(format!("Invalid indent style in config: {v}")))
        })
        .transpose()
}

fn parse_log_level(value: Option<&str>) -> Result<Option<LogLevel>> {
    value
        .map(|v| {
            LogLevel::from_str(v, true)
                .map_err(|_| CustomError::Custom(format!("Invalid log level in config: {v}")))
        })
        .transpose()
}

fn validate_configuration(cfg: &Configuration) -> Result<()> {
    if cfg.stdin {
        if !cfg.include.is_empty() {
            return Err(CustomError::Custom(
                "Do not pass include patterns when using --stdin".to_string(),
            ));
        }
        if cfg.write {
            return Err(CustomError::Custom(
                "--write is not supported with --stdin".to_string(),
            ));
        }
        if cfg.backup {
            return Err(CustomError::Custom(
                "--backup is not supported with --stdin".to_string(),
            ));
        }
    } else if cfg.include.is_empty() {
        return Err(CustomError::Custom(
            "No include file patterns provided".to_string(),
        ));
    }

    Ok(())
}

fn run(cfg: &Configuration) -> Result<()> {
    if cfg.stdin {
        return run_stdin(cfg);
    }

    let start_time = Instant::now();
    let files = files::list_files(
        &cfg.include,
        &cfg.exclude,
        vec![files::Extension::Json, files::Extension::Jsonc],
    )?;

    if files.is_empty() {
        return Err(CustomError::Custom(
            "No JSON files found matching the provided patterns".to_string(),
        ));
    }

    let results: Vec<_> = files
        .par_iter()
        .map(|path| {
            let file_start_time = Instant::now();
            let result = process_file(path, &cfg);
            let duration = file_start_time.elapsed();
            (path, result, duration)
        })
        .collect();

    let successful_files = results
        .iter()
        .filter(|(_, result, _)| result.is_ok())
        .count();
    let changed_files = results
        .iter()
        .filter_map(|(_, result, _)| result.as_ref().ok())
        .filter(|result| result.changed)
        .count();

    let total_files = files.len();

    for (path, result, duration) in results {
        match result {
            Ok(outcome) => {
                if cfg.stdout {
                    if let Some(output) = outcome.output {
                        print_output(&path, &output, total_files);
                    }
                }

                if cfg.check && outcome.changed && !is_quiet(cfg) {
                    println!("{} needs formatting", path.display());
                }

                if !cfg.write && !cfg.check && !cfg.stdout && !is_quiet(cfg) {
                    let status = if outcome.changed {
                        "Needs formatting"
                    } else {
                        "Already formatted"
                    };
                    println!("{}: {status}", path.display().to_string().green());
                }

                if cfg.write && !is_quiet(cfg) {
                    println!(
                        "{}: Processed in {:.2?}",
                        path.display().to_string().green(),
                        duration
                    );
                }
            }
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
            successful_files, total_duration
        )
        .green()
    );

    if cfg.check && changed_files > 0 {
        return Err(CustomError::CheckFailed(changed_files));
    }

    Ok(())
}

fn run_stdin(cfg: &Configuration) -> Result<()> {
    let mut data = String::new();
    io::stdin().read_to_string(&mut data)?;
    if data.trim().is_empty() {
        return Err(CustomError::Custom(
            "No input received on stdin".to_string(),
        ));
    }

    let json: Value = parse_json_value(&data)?;
    let sorted_json = sort::sort(&json, &cfg.order, 0, cfg.depth);
    let indent = get_indent(cfg, &data);
    let formatted_json = format_json(&sorted_json, &indent)?;
    let changed = formatted_json != data;

    if cfg.stdout || !cfg.check {
        println!("{formatted_json}");
    }

    if cfg.check && changed {
        return Err(CustomError::CheckFailed(1));
    }

    Ok(())
}

fn process_file(path: &PathBuf, cfg: &Configuration) -> Result<ProcessOutcome> {
    let data = fs::read_to_string(path)?;
    let json: Value = parse_json_value(&data)?;

    let sorted_json = sort::sort(&json, &cfg.order, 0, cfg.depth);
    let indent = get_indent(cfg, &data);
    let formatted_json = format_json(&sorted_json, &indent)?;
    let changed = formatted_json != data;

    if cfg.write {
        if cfg.backup && changed {
            let backup_path = path.with_extension("bak");
            fs::copy(path, &backup_path)?;
            info!("Backup created: {:?}", backup_path);
        }

        if changed {
            let formatted_json = if is_jsonc(path) {
                restore_jsonc_leading_comments(&data, &formatted_json)
            } else {
                formatted_json
            };
            fs::write(path, &formatted_json)?;
            info!("Sorted JSON written back to {:?}", path);
        }
    }

    let output = if cfg.stdout && !cfg.write {
        Some(formatted_json)
    } else {
        None
    };

    Ok(ProcessOutcome { changed, output })
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
    let detected_indent = detect_indent(data);

    if let Some(indent) = cfg.indent {
        return match cfg.indent_style {
            Some(IndentStyle::Tabs) => "\t".repeat(indent),
            _ => " ".repeat(indent),
        };
    }

    if let Some(indent_style) = &cfg.indent_style {
        return match indent_style {
            IndentStyle::Tabs => "\t".to_string(),
            IndentStyle::Spaces => {
                let width = detected_indent
                    .as_ref()
                    .map(|value| {
                        if value.starts_with('\t') {
                            2
                        } else {
                            value.len()
                        }
                    })
                    .unwrap_or(2);
                " ".repeat(width)
            }
        };
    }

    detected_indent.unwrap_or_else(|| " ".to_string())
}

fn format_json(value: &serde_json::Value, indent: &str) -> Result<String> {
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    value.serialize(&mut ser).map_err(CustomError::Json)?;
    String::from_utf8(buf).map_err(|err| CustomError::Custom(err.to_string()))
}

fn parse_json_value(data: &str) -> Result<Value> {
    match serde_json::from_str::<Value>(data) {
        Ok(value) => Ok(value),
        Err(_) => json5::from_str::<Value>(data)
            .map_err(|err| CustomError::Custom(format!("Failed to parse JSON content: {err}"))),
    }
}

fn is_quiet(cfg: &Configuration) -> bool {
    matches!(cfg.log_level, LogLevel::Quiet)
}

fn print_output(path: &PathBuf, output: &str, total_files: usize) {
    if total_files > 1 {
        println!("--- {} ---", path.display());
    }
    println!("{output}");
}

fn is_jsonc(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("jsonc"))
        .unwrap_or(false)
}

fn restore_jsonc_leading_comments(original: &str, formatted: &str) -> String {
    let mut comments_by_key = collect_leading_comments_by_key(original);
    let mut result = Vec::new();

    for line in formatted.lines() {
        let trimmed = line.trim_start();
        if let Some(key) = extract_object_key(trimmed) {
            if let Some(queue) = comments_by_key.get_mut(&key) {
                if let Some(comments) = queue.pop_front() {
                    result.extend(comments);
                }
            }
        }
        result.push(line.to_string());
    }

    format!("{}\n", result.join("\n"))
}

fn collect_leading_comments_by_key(original: &str) -> HashMap<String, VecDeque<Vec<String>>> {
    let mut by_key: HashMap<String, VecDeque<Vec<String>>> = HashMap::new();
    let mut pending_comments: Vec<String> = Vec::new();
    let mut in_block_comment = false;

    for line in original.lines() {
        let trimmed = line.trim_start();

        if in_block_comment {
            pending_comments.push(line.to_string());
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("//") {
            pending_comments.push(line.to_string());
            continue;
        }

        if trimmed.starts_with("/*") {
            pending_comments.push(line.to_string());
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }

        if let Some(key) = extract_object_key(trimmed) {
            if !pending_comments.is_empty() {
                by_key
                    .entry(key)
                    .or_default()
                    .push_back(std::mem::take(&mut pending_comments));
            }
            continue;
        }

        if !trimmed.is_empty() {
            pending_comments.clear();
        }
    }

    by_key
}

fn extract_object_key(trimmed: &str) -> Option<String> {
    if !trimmed.starts_with('"') {
        return None;
    }

    let mut escaped = false;
    let mut end_index = None;
    for (index, ch) in trimmed.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            end_index = Some(index);
            break;
        }
    }

    let end_index = end_index?;
    let remainder = trimmed[end_index + 1..].trim_start();
    if !remainder.starts_with(':') {
        return None;
    }

    Some(trimmed[1..end_index].to_string())
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
