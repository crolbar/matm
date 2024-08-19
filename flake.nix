{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    nixpkgs,
    flake-parts,
    rust-overlay,
    ...
  } @ inputs: let
    overlays = [(import rust-overlay)];
  in
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux"];

      perSystem = {system, ...}: let
        pkgs = import nixpkgs {inherit system overlays;};
      in {
        _module.args.pkgs = pkgs;

        devShells.default = let
          rust = pkgs.rust-bin.stable.latest.default.override {
            extensions = ["rust-src" "rust-analyzer"];
            targets = ["x86_64-pc-windows-gnu"];
          };
        in
          with pkgs;
            mkShell {
              nativeBuildInputs = [
                pkg-config
                openssl
                perl
                rust
              ];
            };

        packages = rec {
          matm = pkgs.callPackage ./package.nix {};
          default = matm;
        };
      };
    };
}
