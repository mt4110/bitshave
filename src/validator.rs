use crate::optimizer::ImageFormat;
use anyhow::Result;
use file_format::FileFormat;
use std::path::Path;

pub struct ValidatedFile {
    pub path: std::path::PathBuf,
    pub format: ImageFormat,
    pub initial_size: u64,
}

pub fn validate(path: &Path) -> Result<Option<ValidatedFile>> {
    if !path.is_file() {
        return Ok(None);
    }

    let metadata = path.metadata()?;
    if metadata.len() == 0 {
        return Ok(None);
    }

    let format = FileFormat::from_file(path)?;

    let image_format = match format {
        FileFormat::PortableNetworkGraphics => ImageFormat::Png,
        FileFormat::JointPhotographicExpertsGroup => ImageFormat::Jpeg,
        FileFormat::Webp => ImageFormat::WebP,
        FileFormat::ScalableVectorGraphics => ImageFormat::Svg,
        FileFormat::GraphicsInterchangeFormat => ImageFormat::Gif,
        _ => {
            return Ok(None);
        }
    };

    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        let consistent = matches!(
            (image_format, ext_str.as_str()),
            (ImageFormat::Png, "png")
                | (ImageFormat::Jpeg, "jpg" | "jpeg")
                | (ImageFormat::WebP, "webp")
                | (ImageFormat::Svg, "svg")
                | (ImageFormat::Gif, "gif")
        );

        if !consistent {
            return Ok(None);
        }
    } else {
        return Ok(None);
    }

    if image_format == ImageFormat::Svg && std::fs::read_to_string(path).is_err() {
        return Ok(None);
    }

    Ok(Some(ValidatedFile {
        path: path.to_path_buf(),
        format: image_format,
        initial_size: metadata.len(),
    }))
}
