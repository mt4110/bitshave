use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Runs an external command.
/// Returns Ok(true) if the command succeeded (exit code == 0).
pub fn run_external(program: &str, args: &[&str]) -> Result<bool> {
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("Failed to find or execute tool: {}", program))?;

    Ok(status.success())
}

/// Checks if a tool is available in the system PATH without executing it.
///
/// NOTE:
/// Many CLI tools do *not* support a universal "--version" flag (e.g. jpegtran).
/// So we detect availability by searching PATH.
pub fn check_tool_availability(program: &str) -> bool {
    find_in_path(program).is_some()
}

fn find_in_path(program: &str) -> Option<PathBuf> {
    // If program is an explicit path, just check it.
    if program.contains(std::path::MAIN_SEPARATOR) {
        let p = Path::new(program);
        return if is_executable_file(p) {
            Some(p.to_path_buf())
        } else {
            None
        };
    }

    let path_var = env::var_os("PATH")?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(program);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }

        // Windows: honor PATHEXT if present.
        #[cfg(windows)]
        {
            if let Some(pathext) = env::var_os("PATHEXT") {
                for ext in pathext.to_string_lossy().split(';') {
                    let ext = ext.trim();
                    if ext.is_empty() {
                        continue;
                    }
                    let candidate = dir.join(format!("{}{}", program, ext));
                    if is_executable_file(&candidate) {
                        return Some(candidate);
                    }
                }
            }
        }
    }

    None
}

fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = path.metadata() {
            return (meta.permissions().mode() & 0o111) != 0;
        }
        false
    }

    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_echo() {
        let res = run_external("echo", &["hello"]).unwrap();
        assert!(res);
    }
}

use crate::optimizer::ImageFormat;

pub struct ToolAvailability {
    pub oxipng: bool,
    pub jpegtran: bool,
    pub cwebp: bool,
    pub svgo: bool,
    pub gifsicle: bool,
}

impl ToolAvailability {
    pub fn from_paths(paths: &[PathBuf]) -> Self {
        let mut needs_png = false;
        let mut needs_jpeg = false;
        let mut needs_webp = false;
        let mut needs_svg = false;
        let mut needs_gif = false;

        for p in paths {
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            match ext.as_str() {
                "png" => needs_png = true,
                "jpg" | "jpeg" => needs_jpeg = true,
                "webp" => needs_webp = true,
                "svg" => needs_svg = true,
                "gif" => needs_gif = true,
                _ => {}
            }
        }

        Self {
            oxipng: if needs_png {
                check_tool_availability("oxipng")
            } else {
                false
            },
            jpegtran: if needs_jpeg {
                check_tool_availability("jpegtran")
            } else {
                false
            },
            cwebp: if needs_webp {
                check_tool_availability("cwebp")
            } else {
                false
            },
            svgo: if needs_svg {
                check_tool_availability("svgo")
            } else {
                false
            },
            gifsicle: if needs_gif {
                check_tool_availability("gifsicle")
            } else {
                false
            },
        }
    }

    pub fn is_available_for(&self, format: ImageFormat) -> Option<(&'static str, bool)> {
        match format {
            ImageFormat::Png => Some(("oxipng", self.oxipng)),
            ImageFormat::Jpeg => Some(("jpegtran", self.jpegtran)),
            ImageFormat::WebP => Some(("cwebp", self.cwebp)),
            ImageFormat::Svg => Some(("svgo", self.svgo)),
            ImageFormat::Gif => Some(("gifsicle", self.gifsicle)),
        }
    }
}
