pub mod gif;
pub mod jpeg;
pub mod png;
pub mod svg;
pub mod webp;

use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Svg,
    Gif,
}

pub trait Optimizer: Sync + Send {
    fn optimize(&self, path: &Path) -> Result<Option<std::path::PathBuf>>;
}
