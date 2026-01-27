use console::style;
use std::path::Path;

use crate::optimizer::ImageFormat;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug)]
pub enum Status {
    Optimized,
    DryRun,
    Skipped,
}

#[derive(Debug)]
pub struct ProcessOutcome {
    pub path: PathBuf,
    pub format: ImageFormat,
    pub status: Status,
    pub original_size: u64,
    pub new_size: Option<u64>,
    pub duration: Duration,
    pub reason: Option<String>,
    pub output_path: Option<PathBuf>,
}

#[derive(Serialize)]
struct JsonOutcome {
    path: String,
    format: String,
    status: String,
    before_bytes: u64,
    after_bytes: Option<u64>,
    saved_bytes: Option<u64>,
    saved_percent: Option<f64>,
    duration_ms: Option<u128>,
    reason: Option<String>,
    output_path: Option<String>,
}

pub fn print_json(o: &ProcessOutcome) {
    let saved_bytes = match o.new_size {
        Some(new) if new < o.original_size => Some(o.original_size - new),
        _ => None,
    };

    let saved_percent = match (saved_bytes, o.original_size) {
        (Some(saved), before) if before > 0 => Some((saved as f64 / before as f64) * 100.0),
        _ => None,
    };

    let status = match o.status {
        Status::Optimized => "optimized",
        Status::DryRun => "dry-run",
        Status::Skipped => "skipped",
    };

    let record = JsonOutcome {
        path: o.path.to_string_lossy().to_string(),
        format: format!("{:?}", o.format),
        status: status.to_string(),
        before_bytes: o.original_size,
        after_bytes: o.new_size,
        saved_bytes,
        saved_percent,
        duration_ms: Some(o.duration.as_millis()),
        reason: o.reason.clone(),
        output_path: o
            .output_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
    };

    // JSONL: 1 file = 1 line
    if let Ok(line) = serde_json::to_string(&record) {
        println!("{}", line);
    }
}

pub fn print_success(
    path: &Path,
    format: ImageFormat,
    original_size: u64,
    new_size: u64,
    duration: std::time::Duration,
    show_time: bool,
) {
    let saved_bytes = original_size.saturating_sub(new_size);
    let percent = if original_size == 0 {
        0.0
    } else {
        (saved_bytes as f64 / original_size as f64) * 100.0
    };

    println!("{}", style(path.to_string_lossy().to_string()).bold());
    println!("  format : {:?}", format);
    println!("  before : {}", format_size(original_size));
    println!("  after  : {}", format_size(new_size));
    println!(
        "  saved  : {} ({:.1}%)",
        style(format_size(saved_bytes)).green(),
        percent
    );

    if show_time {
        println!("  time   : {} ms", duration.as_millis());
    }

    println!();
}

pub fn print_skipped(path: &Path, reason: &str) {
    println!("{}", style(path.to_string_lossy().to_string()).dim());
    println!("  skipped ({})", reason);
    println!();
}

pub fn print_dry_run(
    path: &Path,
    original_size: u64,
    new_size: u64,
    duration: std::time::Duration,
    show_time: bool,
) {
    let saved_bytes = original_size.saturating_sub(new_size);
    let percent = if original_size == 0 {
        0.0
    } else {
        (saved_bytes as f64 / original_size as f64) * 100.0
    };

    println!(
        "[DRY-RUN] {}",
        style(path.to_string_lossy().to_string()).bold()
    );
    println!(
        "  estimated: {} -> {} (-{:.0}%)",
        format_size(original_size),
        format_size(new_size),
        percent
    );

    if show_time {
        println!("  time   : {} ms", duration.as_millis());
    }

    println!();
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;

    if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
