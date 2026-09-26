use crate::optimizer::ImageFormat;
use anyhow::Result;
use image::{codecs::gif::GifDecoder, metadata::LoopCount, AnimationDecoder, ImageReader};
use std::fs;
use std::io::BufReader;
use std::path::Path;

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

#[derive(Debug)]
pub enum ComparisonResult {
    Improved {
        #[allow(dead_code)]
        saved_bytes: u64,
        #[allow(dead_code)]
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

/// Compare decoded raster content. GIF timing and loop count are part of the
/// displayed result, so they must also match.
pub fn same_pixels(original: &Path, optimized: &Path, format: ImageFormat) -> Result<bool> {
    if format == ImageFormat::Gif {
        let before = GifDecoder::new(BufReader::new(fs::File::open(original)?))?;
        let after = GifDecoder::new(BufReader::new(fs::File::open(optimized)?))?;
        let same_loop = match (before.loop_count(), after.loop_count()) {
            (LoopCount::Infinite, LoopCount::Infinite) => true,
            (LoopCount::Finite(a), LoopCount::Finite(b)) => a == b,
            _ => false,
        };
        if !same_loop {
            return Ok(false);
        }
        let mut before = before.into_frames();
        let mut after = after.into_frames();
        loop {
            match (before.next(), after.next()) {
                (None, None) => return Ok(true),
                (Some(Ok(a)), Some(Ok(b))) => {
                    if a.delay() != b.delay() || a.buffer() != b.buffer() {
                        return Ok(false);
                    }
                }
                (Some(Err(e)), _) | (_, Some(Err(e))) => return Err(e.into()),
                _ => return Ok(false),
            }
        }
    }

    // APNG and animated WebP require frame-aware checks; do not claim a
    // single-frame comparison covers them.
    if format == ImageFormat::Png && fs::read(original)?.windows(4).any(|w| w == b"acTL") {
        return Ok(false);
    }
    if format == ImageFormat::WebP && is_animated_webp(original)? {
        return Ok(false);
    }

    let before = ImageReader::open(original)?.decode()?.to_rgba16();
    let after = ImageReader::open(optimized)?.decode()?.to_rgba16();
    Ok(before == after)
}

fn is_animated_webp(path: &Path) -> Result<bool> {
    let bytes = fs::read(path)?;
    Ok(bytes.windows(4).any(|w| w == b"ANIM"))
}
