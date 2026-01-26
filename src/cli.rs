use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Target directory to scan (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub input: PathBuf,

    /// Output directory (default: in-place)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Dry run: detect reductions but do not modify files
    #[arg(long)]
    pub dry_run: bool,

    /// Show processing time
    #[arg(long)]
    pub time: bool,

    /// Scan recursively
    #[arg(short, long)]
    pub recursive: bool,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}
