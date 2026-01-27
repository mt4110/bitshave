{
  description = "bitshave development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust Toolchain
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
            })
            
            # Tools required for runtime (checking PATH dependence)
            oxipng
            libjpeg
            libwebp
            nodePackages.svgo 
            gifsicle

            # Build tools
            pkg-config
          ];

          shellHook = ''
            echo "🪒  bitshave dev environment loaded via Nix"
            echo "Strict dependency: This tool relies on external binaries provided by this shell."
          '';
        };
      }
    );
}
