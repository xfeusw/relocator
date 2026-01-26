{
  description = "relocator: directory swapper (swapcore + swapdirs)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      fenixPkgs = fenix.packages.${system};

      toolchain = fenixPkgs.complete.withComponents [
        "cargo"
        "rustc"
        "rustfmt"
        "clippy"
        "rust-src"
      ];

      windowsStd = fenixPkgs.targets.x86_64-pc-windows-gnu.latest.rust-std;

      toolchainWithTargets = fenixPkgs.combine [
        toolchain
        windowsStd
      ];

      rust = pkgs.rustPlatform;

      swapdirsPkg = rust.buildRustPackage {
        pname = "swapdirs";
        version = "0.1.0";
        src = self;

        cargoLock = {lockFile = ./Cargo.lock;};

        cargoBuildFlags = ["-p" "swapdirs"];
        cargoTestFlags = ["-p" "swapcore" "-p" "swapdirs"];

        nativeBuildInputs = with pkgs; [pkg-config];
        buildInputs = with pkgs; [openssl];

        doCheck = true;
      };

      devShell = import ./shell.nix {
        inherit pkgs;
        toolchain = toolchainWithTargets;
      };
    in {
      packages = {
        default = swapdirsPkg;
        swapdirs = swapdirsPkg;
      };

      devShells.default = devShell;
    });
}
