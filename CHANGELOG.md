# Changelog

## v0.1.1

- Accept a smaller PNG, JPEG, or static WebP only when decoded RGBA pixels match. GIF output must also preserve every frame, delay, and loop count.
- Reject candidates that cannot be decoded. Animated PNG/WebP and SVG are skipped until their output can be verified appropriately.
- Stage accepted output in the destination directory and report a write failure as skipped. Recheck the source hash before replacing it.
- Add real image fixtures, a Nix-based CI workflow, and accurate guarantee/limitation descriptions in both READMEs.

Metadata, color profile effects, and EXIF orientation are outside the pixel-equality guarantee. This release does not claim visual equivalence across all image viewers.
