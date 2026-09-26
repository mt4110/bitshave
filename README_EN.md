# bitshave

**A smaller file is accepted only when its decoded content still matches.**

bitshave is a Rust CLI that runs external optimizers for PNG, JPEG, WebP, and GIF. It accepts a result only when it is smaller, has the same format, and decodes to the same pixels. For GIF, every frame, its delay, and the loop count must also match. Otherwise the source is kept.

## Guarantees and limits

- PNG, JPEG, and static WebP are compared as decoded RGBA pixels. GIF frames, delays, and loop count are compared. Invalid output is rejected.
- Zero-byte, wrong-format, equal-size, and larger results are rejected. Output is staged in the destination directory before replacement; write failures are not reported as success.
- Animated PNG/WebP are skipped. SVG is recognized but skipped in v0.1.1 because visual equivalence is not verified.
- Metadata preservation, identical rendering in other decoders, and appearance affected by color profiles or EXIF orientation are **not guaranteed**. Back up images whose metadata matters.
- `cwebp -lossless` re-encodes decoded WebP pixels. It does not preserve the source WebP's compression representation or binary structure.

## Development and usage

The Nix shell provides `oxipng`, `jpegtran`, `cwebp`, and `gifsicle`. Outside Nix, the tools needed for each format must be on PATH; unavailable formats are skipped.

```bash
nix develop --command cargo test --all-targets
nix develop --command cargo clippy --all-targets -- -D warnings
nix develop --command cargo run --release -- --help
```

```bash
bitshave
bitshave --input ./assets --output ./dist --recursive
bitshave --input ./assets --dry-run
```

Only improved files are written with `--output`. Dry-run still runs the external optimizers and validates their candidates. Keep normal backups for in-place use.

## License

MIT License
