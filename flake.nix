{
  description = "Rust via Desugarings";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    mdbook-backlinks.url = "github:nadrieril/mdbook-backlinks";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = inputs @ { self, ... }:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ (import inputs.rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;
        craneArgs = {
          src = craneLib.cleanCargoSource ./.;
        };
        crate = pkgs.callPackage
          ({ bintools
           , craneLib
           , lib
           , makeWrapper
           , rustToolchain
           , stdenv
           , zlib
           }:
            craneLib.buildPackage (
              craneArgs
              // {
                buildInputs = [
                  makeWrapper
                  zlib
                ];
                cargoArtifacts = craneLib.buildDepsOnly craneArgs;
                passthru.check-fmt = craneLib.cargoFmt craneArgs;
              }
            ))
          { inherit craneLib rustToolchain; };
      in
      {
        packages = {
          default = crate;
          inherit rustToolchain;
        };
        checks = {
          default = craneLib.cargoFmt craneArgs;
        };
        devShells.default = pkgs.mkShell {
          # # To run `cargo outdated` and `cargo udeps`
          # LD_LIBRARY_PATH =
          #   pkgs.lib.makeLibraryPath [ pkgs.stdenv.cc.cc.lib pkgs.openssl pkgs.curl pkgs.zlib ];

          packages = [
            pkgs.mdbook
            pkgs.mdbook-linkcheck
            inputs.mdbook-backlinks.packages.${system}.default
          ];
          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.rlwrap
          ];
          # To compile some rust crates that need system dependencies.
          buildInputs = [
            pkgs.openssl
            pkgs.glibc.out
            pkgs.glibc.static
          ];

          inputsFrom = [
            self.packages.${system}.default
          ];
        };
      });
}
