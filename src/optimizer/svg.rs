use super::Optimizer;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct SvgOptimizer;

impl Optimizer for SvgOptimizer {
    fn optimize(&self, path: &Path) -> Result<Option<PathBuf>> {
        let named_temp = tempfile::Builder::new().suffix(".svg").tempfile()?;
        let (_file, temp_path) = named_temp.keep()?;

        // svgo args:
        let args = [
            "-i",
            path.to_str().unwrap(),
            "-o",
            temp_path.to_str().unwrap(),
        ];

        if crate::executor::run_external("svgo", &args)? {
            return Ok(Some(temp_path));
        } else {
            let _ = std::fs::remove_file(&temp_path);
        }

        Ok(None)
    }
}
