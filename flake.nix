{
  description = "relocator: directory swapper (swapcore + swapdirs)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};

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

      devShell = import ./shell.nix {inherit pkgs;};
    in {
      packages = {
        default = swapdirsPkg;
        swapdirs = swapdirsPkg;
      };

      devShells.default = devShell;
    });
}
