{
  description = "A Rust roguelike game using SDL2 and TCOD";
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
        rustVersion = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustVersion
            rust-analyzer
            rustfmt
            clippy
            pkg-config
            SDL2
            SDL2_image
            SDL2_ttf
            SDL2_mixer
            # libtcod
          ];
          shellHook = ''
            echo "Rust roguelike development environment"
            echo "-------------------------------------"
            rustc --version
            cargo --version
            echo "-------------------------------------"
            export RUST_SRC_PATH="${rustVersion}/lib/rustlib/src/rust/library"
            exec zsh
          '';
        };
      }
    );
}
