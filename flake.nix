{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, utils, ... }:
  utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };

      nativeBuildInputs =  with pkgs; [
        pkg-config

        (rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
            targets = [ "x86_64-pc-windows-gnu" ];
        })
      ];

      buildInputs = with pkgs; [ openssl.dev ];
    in rec {
      devShell = pkgs.mkShell {
        inherit buildInputs nativeBuildInputs;
      };
    }
  );
}
