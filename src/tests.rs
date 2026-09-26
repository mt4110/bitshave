use crate::compare;
use crate::executor::check_tool_availability;
use crate::optimizer::{
    gif::GifOptimizer, jpeg::JpegOptimizer, png::PngOptimizer, webp::WebPOptimizer, ImageFormat,
    Optimizer,
};
use crate::{process_one, validator, Cli, ToolAvailability};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn validates_real_fixtures_and_rejects_fake_header() {
    for (name, expected) in [
        ("checker.png", ImageFormat::Png),
        ("checker.jpg", ImageFormat::Jpeg),
        ("checker.webp", ImageFormat::WebP),
        ("two-frame.gif", ImageFormat::Gif),
        ("simple.svg", ImageFormat::Svg),
    ] {
        assert_eq!(
            validator::validate(&fixture(name)).unwrap().unwrap().format,
            expected
        );
    }
    let dir = tempdir().unwrap();
    let fake = dir.path().join("fake.png");
    fs::write(&fake, b"not an image").unwrap();
    assert!(validator::validate(&fake).unwrap().is_none());
}

#[test]
fn real_raster_fixtures_have_decodable_pixels() {
    for (name, format) in [
        ("checker.png", ImageFormat::Png),
        ("checker.jpg", ImageFormat::Jpeg),
        ("checker.webp", ImageFormat::WebP),
        ("two-frame.gif", ImageFormat::Gif),
    ] {
        let path = fixture(name);
        assert!(
            compare::same_pixels(&path, &path, format).unwrap(),
            "{name}"
        );
    }
}

#[test]
fn rejects_changed_pixels_and_corrupt_decode() {
    let dir = tempdir().unwrap();
    let original = fixture("checker.png");
    let changed = dir.path().join("changed.png");
    let mut image = image::open(&original).unwrap().to_rgba8();
    image.put_pixel(0, 0, image::Rgba([0, 255, 0, 255]));
    image.save(&changed).unwrap();
    assert!(!compare::same_pixels(&original, &changed, ImageFormat::Png).unwrap());
    fs::write(&changed, b"not an image").unwrap();
    assert!(compare::same_pixels(&original, &changed, ImageFormat::Png).is_err());
}

#[test]
fn rejects_changed_gif_timing() {
    let dir = tempdir().unwrap();
    let original = fixture("two-frame.gif");
    let changed = dir.path().join("changed.gif");
    let mut bytes = fs::read(&original).unwrap();
    let marker = bytes
        .windows(3)
        .position(|w| w == [0x21, 0xf9, 0x04])
        .unwrap();
    bytes[marker + 4] = bytes[marker + 4].wrapping_add(1);
    fs::write(&changed, bytes).unwrap();
    assert!(!compare::same_pixels(&original, &changed, ImageFormat::Gif).unwrap());
}

#[test]
fn svg_is_explicitly_skipped() {
    let path = fixture("simple.svg");
    let args = Cli {
        input: path.parent().unwrap().to_path_buf(),
        output: None,
        dry_run: false,
        time: false,
        recursive: false,
        json: false,
    };
    let tools = ToolAvailability::from_paths(std::slice::from_ref(&path));
    let outcome = process_one(&args, &tools, &path).unwrap().unwrap();
    assert!(matches!(outcome.status, crate::report::Status::Skipped));
    assert!(outcome.reason.unwrap().contains("SVG visual equivalence"));
}

#[test]
fn available_tools_are_checked_with_real_images() {
    for (name, tool, optimizer) in [
        (
            "checker.png",
            "oxipng",
            Box::new(PngOptimizer) as Box<dyn Optimizer>,
        ),
        ("checker.jpg", "jpegtran", Box::new(JpegOptimizer)),
        ("checker.webp", "cwebp", Box::new(WebPOptimizer)),
        ("two-frame.gif", "gifsicle", Box::new(GifOptimizer)),
    ] {
        if !check_tool_availability(tool) {
            continue;
        }
        let path = fixture(name);
        if let Some(output) = optimizer.optimize(&path).unwrap() {
            assert!(validator::validate(&output).unwrap().is_some(), "{name}");
            let format = validator::validate(&path).unwrap().unwrap().format;
            assert!(
                compare::same_pixels(&path, &output, format).unwrap(),
                "{name}"
            );
            fs::remove_file(output).unwrap();
        }
    }
}
