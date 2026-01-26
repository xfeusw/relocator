{
  pkgs,
  toolchain ? pkgs.rustc,
}:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    toolchain

    cargo-nextest
    cargo-watch
    just
    fd

    # Windows GNU cross linker
    pkgsCross.mingwW64.stdenv.cc

    # Optional: run exe on Linux
    wineWowPackages.stable
  ];

  # nativeBuildInputs = nativeBuildInputs ++ [ pkgs.pkg-config ];
  # buildInputs = [ pkgs.openssl ];

  RUST_BACKTRACE = "1";
}
