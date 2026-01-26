pub mod png;
pub mod jpeg;
pub mod webp;
pub mod svg;
pub mod gif;

use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Svg,
    Gif,
    Unknown,
}

pub trait Optimizer: Sync + Send {
    fn can_handle(&self, format: ImageFormat) -> bool;
    fn optimize(&self, path: &Path) -> Result<Option<std::path::PathBuf>>;
}
