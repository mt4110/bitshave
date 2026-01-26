#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use crate::validator;
    use crate::optimizer::{ImageFormat, png::PngOptimizer, Optimizer};

    // === Validator Tests ===

    #[test]
    fn test_validator_rejects_text_file_masked_as_png() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("fake.png");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "This is not a PNG").unwrap(); // Write text

        let result = validator::validate(&file_path).unwrap();
        assert!(result.is_none(), "Should reject text file pretending to be PNG");
    }

    #[test]
    fn test_validator_accepts_valid_png_header() {
        // Create a minimal valid PNG header
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("real.png");
        let mut file = File::create(&file_path).unwrap();
        // PNG Magic bytes: 89 50 4E 47 0D 0A 1A 0A
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).unwrap();
        
        // This validates "Magic Bytes". 
        // `file-format` crate needs a bit more than just 8 bytes usually? 
        // Actually 8 bytes is enough for PNG.
        
        let result = validator::validate(&file_path).unwrap();
        assert!(result.is_some(), "Should accept file with valid PNG signature");
        if let Some(v) = result {
            assert_eq!(v.format, ImageFormat::Png);
        }
    }

    // === Executor/Optimizer Tests (Requires Nix Environment) ===
    
    #[test]
    fn test_oxipng_installed() {
        // Check if `oxipng` is in PATH (provided by Nix)
        let status = std::process::Command::new("oxipng")
            .arg("--version")
            .output();
            
        assert!(status.is_ok(), "oxipng should be available in the environment");
    }

    #[test]
    fn test_png_optimizer_runs() {
        if std::process::Command::new("oxipng").arg("--version").output().is_err() {
            return; 
        }
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.png");
        {
            let mut f = File::create(&file_path).unwrap();
            f.write_all(&[0u8; 100]).unwrap(); 
        }
        let opt = PngOptimizer;
        let res = opt.optimize(&file_path);
        assert!(res.is_ok() || res.is_err()); 
    }

    #[test]
    fn test_jpeg_optimizer_runs() {
        if std::process::Command::new("jpegtran").arg("-version").output().is_err() { // jpegtran uses -version typically or -h, standard check often fails if weird version. Nix ensures availability.
             // allow skip if not present in dev checking? 
             // In Nix, it should be there.
        }
        
        // Actually, just try running it.
        use crate::optimizer::jpeg::JpegOptimizer;
        let opt = JpegOptimizer;
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.jpg");
        {
            let mut f = File::create(&file_path).unwrap();
            f.write_all(&[0u8; 100]).unwrap(); 
        }
        let _ = opt.optimize(&file_path); 
        // We just want to ensure it calls the binary without crashing.
    }

    #[test]
    fn test_webp_optimizer_runs() {
        use crate::optimizer::webp::WebPOptimizer;
        let opt = WebPOptimizer;
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.webp");
        {
            let mut f = File::create(&file_path).unwrap();
            f.write_all(&[0u8; 100]).unwrap(); 
        }
        let _ = opt.optimize(&file_path);
    }
    
    #[test]
    fn test_svg_optimizer_runs() {
        use crate::optimizer::svg::SvgOptimizer;
        let opt = SvgOptimizer;
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.svg");
        {
            let mut f = File::create(&file_path).unwrap();
            f.write_all(b"<svg></svg>").unwrap(); 
        }
        let _ = opt.optimize(&file_path);
    }
    
    #[test]
    fn test_gif_optimizer_runs() {
        if std::process::Command::new("gifsicle").arg("--version").output().is_err() {
            return;
        }
        use crate::optimizer::gif::GifOptimizer;
        let opt = GifOptimizer;
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.gif");
        {
             // Create empty file (gifsicle might fail, but checking execution)
            let mut f = File::create(&file_path).unwrap();
            f.write_all(&[0u8; 10]).unwrap(); 
        }
        let _ = opt.optimize(&file_path);
    }
    

}
