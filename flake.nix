{
  description = "Rust via Desugarings mdBook";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    mdbook-backlinks.url = "github:nadrieril/mdbook-backlinks";
  };

  outputs = inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import inputs.nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.mdbook
            pkgs.mdbook-linkcheck
            inputs.mdbook-backlinks.packages.${system}.default
          ];
        };
      });
}
