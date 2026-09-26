mod cli;
mod compare;
mod executor;
mod optimizer;
mod report;
mod scanner;
mod validator;

#[cfg(test)]
mod tests;

use clap::Parser;
use cli::Cli;
use executor::ToolAvailability;
use optimizer::ImageFormat;
use rayon::prelude::*;
use report::{ProcessOutcome, Status};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn main() {
    let args = Cli::parse();

    // 0) Scan
    let files = scanner::scan(&args);
    if files.is_empty() {
        if !args.json {
            println!("No image files found in {:?}", args.input);
        }
        return;
    }

    // 1) Tool availability (only for formats that appear by extension)
    let tools = ToolAvailability::from_paths(&files);

    // 2) Process (parallel) -> collect outcomes, then print deterministically
    let mut outcomes: Vec<ProcessOutcome> = files
        .par_iter()
        .filter_map(|path| process_one(&args, &tools, path).ok().flatten())
        .collect();

    outcomes.sort_by(|a, b| a.path.cmp(&b.path));

    for o in outcomes {
        if args.json {
            report::print_json(&o);
            continue;
        }

        match o.status {
            Status::Optimized => {
                report::print_success(
                    &o.path,
                    o.format,
                    o.original_size,
                    o.new_size.unwrap_or(o.original_size),
                    o.duration,
                    args.time,
                );
            }
            Status::DryRun => {
                report::print_dry_run(
                    &o.path,
                    o.original_size,
                    o.new_size.unwrap_or(o.original_size),
                    o.duration,
                    args.time,
                );
            }
            Status::Skipped => {
                report::print_skipped(&o.path, o.reason.as_deref().unwrap_or("skipped"));
            }
        }
    }
}

fn process_one(
    args: &Cli,
    tools: &ToolAvailability,
    path: &Path,
) -> anyhow::Result<Option<ProcessOutcome>> {
    // Validation: only real images, consistent extension, non-zero.
    let valid_file = match validator::validate(path) {
        Ok(Some(v)) => v,
        Ok(None) => return Ok(None),
        Err(_) => {
            // Non-readable files: skip silently to avoid noisy logs in large trees.
            return Ok(None);
        }
    };

    if valid_file.format == ImageFormat::Svg {
        return Ok(Some(ProcessOutcome {
            path: valid_file.path,
            format: ImageFormat::Svg,
            status: Status::Skipped,
            original_size: valid_file.initial_size,
            new_size: None,
            duration: Duration::ZERO,
            reason: Some("SVG visual equivalence is not verified".to_string()),
            output_path: None,
        }));
    }

    // Tool availability per format.
    if let Some((tool, ok)) = tools.is_available_for(valid_file.format) {
        if !ok {
            return Ok(Some(ProcessOutcome {
                path: valid_file.path,
                format: valid_file.format,
                status: Status::Skipped,
                original_size: valid_file.initial_size,
                new_size: None,
                duration: Duration::from_millis(0),
                reason: Some(format!("tool missing: {}", tool)),
                output_path: None,
            }));
        }
    }

    // Select optimizer.
    let optimizer: Box<dyn optimizer::Optimizer> = match valid_file.format {
        ImageFormat::Png => Box::new(optimizer::png::PngOptimizer),
        ImageFormat::Jpeg => Box::new(optimizer::jpeg::JpegOptimizer),
        ImageFormat::WebP => Box::new(optimizer::webp::WebPOptimizer),
        ImageFormat::Svg => Box::new(optimizer::svg::SvgOptimizer),
        ImageFormat::Gif => Box::new(optimizer::gif::GifOptimizer),
    };

    // Initial stats
    let initial_stats = match compare::FileStats::from_file(&valid_file.path) {
        Ok(s) => s,
        Err(_) => {
            return Ok(Some(ProcessOutcome {
                path: valid_file.path,
                format: valid_file.format,
                status: Status::Skipped,
                original_size: valid_file.initial_size,
                new_size: None,
                duration: Duration::from_millis(0),
                reason: Some("read error".to_string()),
                output_path: None,
            }));
        }
    };

    let start_time = Instant::now();
    let optimization_result = optimizer.optimize(&valid_file.path);
    let duration = start_time.elapsed();

    match optimization_result {
        Ok(Some(temp_path)) => {
            // Reject empty output (mock bug / tool crash) immediately.
            let new_size = std::fs::metadata(&temp_path).map(|m| m.len()).unwrap_or(0);
            if new_size == 0 {
                let _ = std::fs::remove_file(&temp_path);
                return Ok(Some(ProcessOutcome {
                    path: valid_file.path,
                    format: valid_file.format,
                    status: Status::Skipped,
                    original_size: initial_stats.size,
                    new_size: None,
                    duration,
                    reason: Some("invalid output (0-byte)".to_string()),
                    output_path: None,
                }));
            }

            // Validate optimized output is still an image of the same format.
            let out_valid = validator::validate(&temp_path)
                .ok()
                .flatten()
                .filter(|v| v.format == valid_file.format);
            if out_valid.is_none() {
                let _ = std::fs::remove_file(&temp_path);
                return Ok(Some(ProcessOutcome {
                    path: valid_file.path,
                    format: valid_file.format,
                    status: Status::Skipped,
                    original_size: initial_stats.size,
                    new_size: None,
                    duration,
                    reason: Some("invalid output (format mismatch)".to_string()),
                    output_path: None,
                }));
            }

            match compare::same_pixels(&valid_file.path, &temp_path, valid_file.format) {
                Ok(true) => {}
                Ok(false) | Err(_) => {
                    let _ = std::fs::remove_file(&temp_path);
                    return Ok(Some(ProcessOutcome {
                        path: valid_file.path,
                        format: valid_file.format,
                        status: Status::Skipped,
                        original_size: initial_stats.size,
                        new_size: None,
                        duration,
                        reason: Some(
                            "decoded pixels or animation differ / cannot be verified".to_string(),
                        ),
                        output_path: None,
                    }));
                }
            }

            let cmp = compare::compare(&initial_stats, &temp_path)
                .unwrap_or(compare::ComparisonResult::WorseOrEqual);

            match cmp {
                compare::ComparisonResult::Improved { .. } => {
                    if args.dry_run {
                        let _ = std::fs::remove_file(&temp_path);
                        return Ok(Some(ProcessOutcome {
                            path: valid_file.path,
                            format: valid_file.format,
                            status: Status::DryRun,
                            original_size: initial_stats.size,
                            new_size: Some(new_size),
                            duration,
                            reason: None,
                            output_path: None,
                        }));
                    }

                    // Apply
                    let (target_path, output_path) = if let Some(out_dir) = &args.output {
                        let target = compute_output_path(&args.input, out_dir, &valid_file.path);
                        (target.clone(), Some(target))
                    } else {
                        (valid_file.path.clone(), None)
                    };

                    if compare::FileStats::from_file(&valid_file.path)
                        .map(|current| current.hash != initial_stats.hash)
                        .unwrap_or(true)
                    {
                        let _ = std::fs::remove_file(&temp_path);
                        return Ok(Some(ProcessOutcome {
                            path: valid_file.path,
                            format: valid_file.format,
                            status: Status::Skipped,
                            original_size: initial_stats.size,
                            new_size: None,
                            duration,
                            reason: Some("source changed during optimization".to_string()),
                            output_path: None,
                        }));
                    }

                    if apply_output(&temp_path, &target_path, &valid_file.path).is_err() {
                        let _ = std::fs::remove_file(&temp_path);
                        return Ok(Some(ProcessOutcome {
                            path: valid_file.path,
                            format: valid_file.format,
                            status: Status::Skipped,
                            original_size: initial_stats.size,
                            new_size: None,
                            duration,
                            reason: Some("could not write output".to_string()),
                            output_path: None,
                        }));
                    }
                    let _ = std::fs::remove_file(&temp_path);

                    Ok(Some(ProcessOutcome {
                        path: valid_file.path,
                        format: valid_file.format,
                        status: Status::Optimized,
                        original_size: initial_stats.size,
                        new_size: Some(new_size),
                        duration,
                        reason: None,
                        output_path,
                    }))
                }
                compare::ComparisonResult::WorseOrEqual => {
                    let _ = std::fs::remove_file(&temp_path);
                    Ok(Some(ProcessOutcome {
                        path: valid_file.path,
                        format: valid_file.format,
                        status: Status::Skipped,
                        original_size: initial_stats.size,
                        new_size: None,
                        duration,
                        reason: Some("no size reduction".to_string()),
                        output_path: None,
                    }))
                }
                compare::ComparisonResult::Unchanged => {
                    let _ = std::fs::remove_file(&temp_path);
                    Ok(Some(ProcessOutcome {
                        path: valid_file.path,
                        format: valid_file.format,
                        status: Status::Skipped,
                        original_size: initial_stats.size,
                        new_size: None,
                        duration,
                        reason: Some("unchanged".to_string()),
                        output_path: None,
                    }))
                }
            }
        }
        Ok(None) => Ok(Some(ProcessOutcome {
            path: valid_file.path,
            format: valid_file.format,
            status: Status::Skipped,
            original_size: initial_stats.size,
            new_size: None,
            duration,
            reason: Some("optimization failed / no change".to_string()),
            output_path: None,
        })),
        Err(_) => Ok(Some(ProcessOutcome {
            path: valid_file.path,
            format: valid_file.format,
            status: Status::Skipped,
            original_size: initial_stats.size,
            new_size: None,
            duration,
            reason: Some("tool error".to_string()),
            output_path: None,
        })),
    }
}

fn apply_output(optimized: &Path, target: &Path, original: &Path) -> anyhow::Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| anyhow::anyhow!("output has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let staged = tempfile::NamedTempFile::new_in(parent)?;
    std::fs::copy(optimized, staged.path())?;
    std::fs::set_permissions(staged.path(), std::fs::metadata(original)?.permissions())?;
    staged.persist(target)?;
    Ok(())
}

fn compute_output_path(input_root: &Path, out_root: &Path, source: &Path) -> PathBuf {
    // Keep directory structure: out_root/<relative path from input_root>
    match source.strip_prefix(input_root) {
        Ok(rel) => out_root.join(rel),
        Err(_) => out_root.join(source.file_name().unwrap_or_default()),
    }
}
