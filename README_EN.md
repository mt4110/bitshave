# bitshave - Safe & Deterministic Lossless Image Optimizer

**"Reduce size without touching a single pixel."**

`bitshave` is a CLI tool designed to remove redundant bits and metadata from image files without altering the visual content or pixel data.
It strictly avoids "AI enhancement" or "lossy compression". If no reduction is possible, it honestly reports "no change" and leaves the file untouched.

## 🚀 Features

- **Safety First**: Never corrupts the original file. Discards changes if size increases.
- **Strictly Lossless**: Only optimizes metadata and compression efficiency.
- **Fast & Parallel**: Built with Rust for high-performance multi-core processing.
- **Nix Powered**: Environment is strictly managed via Nix to ensure reproducible builds and dependency management.
- **Honest Reporting**: Shows exactly how much space was saved, or admits when no optimization was possible.

## 📦 Requirements

**Nix** is required to run and develop this tool.

```bash
# Enter the Nix environment (installs oxipng, jpegtran, cwebp, svgo automatically)
nix develop

# Build & Run
cargo run --release -- --help
```

## 🛠 Usage

```bash
# Optimize images in current directory (in-place)
bitshave

# Optimize a specific directory and output to a new location
bitshave --input ./assets --output ./dist

# Recursive scan
bitshave --input ./assets --recursive

# Dry-run (Predict size reduction without modifying files)
bitshave --dry-run
```

## 📄 License

MIT License. Free to use.
