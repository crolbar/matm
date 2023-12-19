{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
    shellHook = "export SHELL=${pkgs.zsh}/bin/zsh";
    nativeBuildInputs = with pkgs; [
        pkg-config
        openssl
    ];
}
