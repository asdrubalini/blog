{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    rust-flake.url = "github:juspay/rust-flake";
    rust-flake.inputs.nixpkgs.follows = "nixpkgs";

    # Dev tools
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs = inputs@{ nixpkgs, flake-parts, systems, rust-flake, treefmt-nix, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import systems;
      imports = [
        treefmt-nix.flakeModule
        rust-flake.flakeModules.default
        rust-flake.flakeModules.nixpkgs
      ];

      perSystem = { config, self', pkgs, lib, system, ... }: {
        rust-project.crane.args = {
          buildInputs = lib.optionals pkgs.stdenv.isDarwin (
            with pkgs.darwin.apple_sdk.frameworks; [
              IOKit
            ]
          );
        };

        # Add your auto-formatters here.
        # cf. https://numtide.github.io/treefmt/
        treefmt.config = {
          projectRootFile = "flake.nix";
          programs = {
            nixpkgs-fmt.enable = true;
            rustfmt.enable = true;
          };
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self'.devShells.blog ];
          packages = with pkgs; [
            cargo-watch
            just
            treefmt
            flyctl
          ];
        };
        packages.default = self'.packages.blog;

        packages.container = pkgs.dockerTools.buildImage {
          name = "blog";
          tag = self'.packages.blog.version;

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [ self'.packages.blog ];
            pathsToLink = [ "/bin" ];
          };

          config = {
            Cmd = [ "/bin/blog" ];
          };
        };
      };
    };
}
