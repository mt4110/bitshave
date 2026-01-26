use console::style;
use std::path::Path;
use crate::optimizer::ImageFormat;

pub fn print_success(
    path: &Path,
    format: ImageFormat,
    original_size: u64,
    new_size: u64,
    duration: std::time::Duration,
) {
    let saved_bytes = original_size - new_size;
    let percent = (saved_bytes as f64 / original_size as f64) * 100.0;

    println!("{}", style(path.file_name().unwrap_or_default().to_string_lossy()).bold());
    println!("  format : {:?}", format);
    println!("  before : {}", format_size(original_size));
    println!("  after  : {}", format_size(new_size));
    println!("  saved  : {} ({:.1}%)", 
        style(format_size(saved_bytes)).green(), 
        percent
    );
    println!("  time   : {:?}", duration);
    println!();
}

pub fn print_skipped(path: &Path, reason: &str) {
    println!("{}", style(path.file_name().unwrap_or_default().to_string_lossy()).dim());
    println!("  skipped ({})", reason);
    println!();
}

pub fn print_dry_run(path: &Path, original_size: u64, new_size: u64) {
    let saved_bytes = original_size - new_size;
    let percent = (saved_bytes as f64 / original_size as f64) * 100.0;
    
    println!("[DRY-RUN] {}", style(path.file_name().unwrap_or_default().to_string_lossy()).bold());
    println!("  estimated: {} -> {} (-{:.0}%)", 
        format_size(original_size), 
        format_size(new_size), 
        percent
    );
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
