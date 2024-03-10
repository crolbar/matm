{
  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
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
      in with pkgs;
      {
        defaultPackage = (makeRustPlatform {
          inherit cargo rustc;
        }).buildRustPackage {
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkg-config perl ];
            buildInputs = [ openssl ];
            version = "0.1";
            pname = "matm";
            src = ./.;
        };

        devShells.default = mkShell {
          buildInputs = [
            pkg-config
            openssl
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
              targets = [ "x86_64-pc-windows-gnu" ];
            })
          ];
        };
      }
    );
}
