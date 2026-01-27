use super::Optimizer;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct JpegOptimizer;

impl Optimizer for JpegOptimizer {
    fn optimize(&self, path: &Path) -> Result<Option<PathBuf>> {
        let named_temp = tempfile::Builder::new().suffix(".jpg").tempfile()?;
        let (_file, temp_path) = named_temp.keep()?;

        // jpegtran args:
        let args = [
            "-copy",
            "none",
            "-optimize",
            "-outfile",
            temp_path.to_str().unwrap(),
            path.to_str().unwrap(),
        ];

        if crate::executor::run_external("jpegtran", &args)? {
            return Ok(Some(temp_path));
        } else {
            let _ = std::fs::remove_file(&temp_path);
        }

        Ok(None)
    }
}
