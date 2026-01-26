mod cli;
mod scanner;
mod validator;
mod optimizer;
mod executor;
mod compare;
mod report;

#[cfg(test)]
mod tests;

use clap::Parser;
use cli::Cli;
use rayon::prelude::*;
use std::time::Instant;

fn main() {
    let args = Cli::parse();
    
    // 0. Scan
    let files = scanner::scan(&args);
    if files.is_empty() {
        println!("No image files found in {:?}", args.input);
        return;
    }

    // 1. Process in Parallel
    files.par_iter().for_each(|path| {
        // Validation
        let valid_file = match validator::validate(path) {
            Ok(Some(v)) => v,
            Ok(None) => return,
            Err(e) => {
                eprintln!("Error reading {}: {}", path.display(), e);
                return;
            }
        };

        // Select Optimizer
        let optimizer: Box<dyn optimizer::Optimizer> = match valid_file.format {
            optimizer::ImageFormat::Png => Box::new(optimizer::png::PngOptimizer),
            optimizer::ImageFormat::Jpeg => Box::new(optimizer::jpeg::JpegOptimizer),
            optimizer::ImageFormat::WebP => Box::new(optimizer::webp::WebPOptimizer),
            optimizer::ImageFormat::Svg => Box::new(optimizer::svg::SvgOptimizer),
            optimizer::ImageFormat::Gif => Box::new(optimizer::gif::GifOptimizer),
            optimizer::ImageFormat::Unknown => return,
        };

        let start_time = Instant::now();
        
        // Initial Stats
        let initial_stats = match compare::FileStats::from_file(&valid_file.path) {
            Ok(s) => s,
            Err(_) => return,
        };

        // Optimize
        let optimization_result = optimizer.optimize(&valid_file.path);
        
        match optimization_result {
            Ok(Some(temp_path)) => {
                // Compare
                let cmp = compare::compare(&initial_stats, &temp_path).unwrap_or(compare::ComparisonResult::WorseOrEqual);
                
                match cmp {
                    compare::ComparisonResult::Improved { saved_bytes: _, percent: _ } => {
                       let new_size = std::fs::metadata(&temp_path).map(|m| m.len()).unwrap_or(0);
                       
                       if args.dry_run {
                           report::print_dry_run(&valid_file.path, initial_stats.size, new_size);
                       } else {
                           // Apply
                           let target_path = if let Some(out_dir) = &args.output {
                               if !out_dir.exists() {
                                   let _ = std::fs::create_dir_all(out_dir);
                                }
                                out_dir.join(valid_file.path.file_name().unwrap())
                           } else {
                               valid_file.path.clone()
                           };
                           
                           if std::fs::rename(&temp_path, &target_path).is_err() {
                               if std::fs::copy(&temp_path, &target_path).is_ok() {
                                   let _ = std::fs::remove_file(&temp_path);
                               }
                           }
                           
                           report::print_success(
                               &valid_file.path,
                               valid_file.format,
                               initial_stats.size,
                               new_size,
                               start_time.elapsed()
                           );
                       }
                    },
                    compare::ComparisonResult::WorseOrEqual => {
                         report::print_skipped(&valid_file.path, "no size reduction");
                    },
                    compare::ComparisonResult::Unchanged => {
                         report::print_skipped(&valid_file.path, "unchanged");
                    },
                }
                
                // Cleanup temp if it still exists (Improvement logic might have moved it, others didn't)
                // If we moved it, rename succeeds. If dry run or failed, file exists.
                // Simple check: exists? delete.
                if temp_path.exists() {
                    let _ = std::fs::remove_file(&temp_path);
                }
            },
            Ok(None) => {
                report::print_skipped(&valid_file.path, "optimization failed / no change");
            },
            Err(_) => {
                report::print_skipped(&valid_file.path, "tool error");
            }
        }
    });
}
