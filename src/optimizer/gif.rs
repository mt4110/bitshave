use super::{ImageFormat, Optimizer};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct GifOptimizer;

impl Optimizer for GifOptimizer {
    fn can_handle(&self, format: ImageFormat) -> bool {
        matches!(format, ImageFormat::Gif)
    }

    fn optimize(&self, path: &Path) -> Result<Option<PathBuf>> {
        let named_temp = tempfile::Builder::new()
            .suffix(".gif")
            .tempfile()?;
        let (_file, temp_path) = named_temp.keep()?;

        // gifsicle args:
        // -O3: Maximum optimization (reorders colormaps, etc. safe/lossless in context of pixel data preservation)
        // --no-comments: Strip comments
        // --no-names: Strip names
        // -o <output> <input>

        let args = [
            "-O3",
            "--no-comments",
            "--no-names",
            "-o", temp_path.to_str().unwrap(),
            path.to_str().unwrap()
        ];
        
        if crate::executor::run_external("gifsicle", &args)? {
             return Ok(Some(temp_path));
        } else {
             let _ = std::fs::remove_file(&temp_path);
        }

        Ok(None)
    }
}
