use super::Optimizer;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct PngOptimizer;

impl Optimizer for PngOptimizer {
    fn optimize(&self, path: &Path) -> Result<Option<PathBuf>> {
        let named_temp = tempfile::Builder::new().suffix(".png").tempfile()?;
        let (_file, temp_path) = named_temp.keep()?;

        let output_path = temp_path.clone();

        // oxipng args:
        let args = [
            "-o",
            "2",
            "--strip",
            "safe",
            "--out",
            output_path.to_str().unwrap(),
            path.to_str().unwrap(),
        ];

        let success = crate::executor::run_external("oxipng", &args)?;

        if success {
            return Ok(Some(temp_path));
        } else {
            let _ = std::fs::remove_file(&temp_path);
        }

        Ok(None)
    }
}
