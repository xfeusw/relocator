{pkgs ? import <nixpkgs> {}}: let
  mkDevShell = pkgs:
    pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        rustc
        cargo
        rustfmt
        clippy

        cargo-nextest
        cargo-watch
        just

        pkg-config
        fd
      ];

      buildInputs = with pkgs; [
        openssl
      ];

      RUST_BACKTRACE = "1";
    };
in
  mkDevShell pkgs
