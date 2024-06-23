{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, treefmt-nix, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        toolchain =
          pkgs.rust-bin.stable.latest.default.override
            {
              extensions = [
                "rust-src"
                "rust-analyzer"
                "clippy"
              ];
            };

        craneLib = crane.mkLib pkgs;

        # Only keeps markdown files
        htmlFilter = path: _type: builtins.match ".*html$" path != null;
        orgFilter = path: _type: builtins.match ".*org$" path != null;
        markdownOrCargo = path: type:
          (htmlFilter path type) || (orgFilter path type) || (craneLib.filterCargoSources path type);

        blog = craneLib.buildPackage {
          src = pkgs.lib.cleanSourceWith {
            src = craneLib.path ./.; # The original, unfiltered source
            filter = markdownOrCargo;
          };

          buildInputs = [
            # Add additional build inputs here
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
          ];

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };
      in
      {
        packages.default = blog;

        treefmt.config = {
          projectRootFile = "flake.nix";
          programs = {
            nixpkgs-fmt.enable = true;
            rustfmt.enable = true;
          };
        };

        packages.container = pkgs.dockerTools.buildImage {
          name = "blog";
          tag = blog.version;

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [ blog ./templates ];
            pathsToLink = [ "/bin" "/bin/templates" ]; # TODO: make sure that this works
          };

          config = {
            Cmd = [ "/bin/blog" ];
          };
        };

        devShells.default = craneLib.devShell {
          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEV_URL = "http://localhost:3000";

          shellHook = ''
            # For rust-analyzer 'hover' tooltips to work.
            export RUST_SRC_PATH="${toolchain}/lib/rustlib/src/rust/library";
          '';

          # Automatically inherit any build inputs from `blog`
          inputsFrom = [ blog ];

          # Extra inputs (only used for interactive development)
          # can be added here; cargo and rustc are provided by default.
          packages = with pkgs; [
            cargo-audit
            cargo-watch
            just
            treefmt
            flyctl
          ];
        };
      });
}
