use super::Optimizer;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct WebPOptimizer;

impl Optimizer for WebPOptimizer {
    fn optimize(&self, path: &Path) -> Result<Option<PathBuf>> {
        let named_temp = tempfile::Builder::new().suffix(".webp").tempfile()?;
        let (_file, temp_path) = named_temp.keep()?;

        // cwebp args:
        let args = [
            "-lossless",
            "-metadata",
            "none",
            "-o",
            temp_path.to_str().unwrap(),
            path.to_str().unwrap(),
        ];

        if crate::executor::run_external("cwebp", &args)? {
            return Ok(Some(temp_path));
        } else {
            let _ = std::fs::remove_file(&temp_path);
        }

        Ok(None)
    }
}
