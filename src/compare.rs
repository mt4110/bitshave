use anyhow::Result;
use std::path::Path;
use std::fs;

pub struct FileStats {
    pub size: u64,
    pub hash: blake3::Hash,
}

impl FileStats {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read(path)?;
        let size = content.len() as u64;
        let hash = blake3::hash(&content);
        Ok(Self { size, hash })
    }
}

pub struct ComparisonRequest {
    pub original_path: std::path::PathBuf,
    pub optimized_path: std::path::PathBuf,
}

pub enum ComparisonResult {
    Improved {
        saved_bytes: u64,
        percent: f64,
    },
    WorseOrEqual,
    Unchanged, // Exact same file content (hash match)
}

pub fn compare(original: &FileStats, optimized_path: &Path) -> Result<ComparisonResult> {
    let optimized_stats = FileStats::from_file(optimized_path)?;

    if original.hash == optimized_stats.hash {
        return Ok(ComparisonResult::Unchanged);
    }

    if optimized_stats.size >= original.size {
        return Ok(ComparisonResult::WorseOrEqual);
    }

    let saved = original.size - optimized_stats.size;
    let percent = (saved as f64 / original.size as f64) * 100.0;

    Ok(ComparisonResult::Improved {
        saved_bytes: saved,
        percent,
    })
}
