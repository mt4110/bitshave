use walkdir::WalkDir;
use std::path::{Path, PathBuf};
use crate::cli::Cli;

pub fn scan(cli: &Cli) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    let max_depth = if cli.recursive { usize::MAX } else { 1 };

    for entry in WalkDir::new(&cli.input)
        .max_depth(max_depth)
        .follow_links(false) // Security: don't follow symlinks to weird places
        .into_iter()
        .filter_map(|e| e.ok()) 
    {
        if entry.file_type().is_file() {
            // Preliminary extension filter to avoid identifying every single file in the disk
            if has_image_extension(entry.path()) {
                files.push(entry.path().to_path_buf());
            }
        }
    }
    files
}

fn has_image_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let e = e.to_lowercase();
            matches!(e.as_str(), "png" | "jpg" | "jpeg" | "webp" | "svg" | "gif")
        })
        .unwrap_or(false)
}
