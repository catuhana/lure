{
  description = "Sync your Last.fm or ListenBrainz listening status to Stoat.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix.url = "github:nix-community/fenix";

    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      perSystem =
        {
          pkgs,
          lib,
          system,
          ...
        }:
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              (final: prev: {
                rustToolchain = inputs.fenix.packages.${system}.fromToolchainFile {
                  file = ./rust-toolchain.toml;
                  sha256 = "sha256-zC8E38iDVJ1oPIzCqTk/Ujo9+9kx9dXq7wAwPMpkpg0=";
                };
              })
            ];
          };

          packages = rec {
            default = lure;

            lure =
              let
                cargoTOML = builtins.fromTOML (builtins.readFile ./Cargo.toml);
              in
              (pkgs.makeRustPlatform {
                cargo = pkgs.rustToolchain;
                rustc = pkgs.rustToolchain;
              }).buildRustPackage
                {
                  pname = "lure";
                  version = cargoTOML.workspace.package.version;

                  src = inputs.self;
                  cargoLock.lockFile = ./Cargo.lock;

                  # TODO: Deduplicate this with the devShell.
                  # Or may need to drop if I switch over to rustls.
                  buildInputs = [ pkgs.openssl ];
                  nativeBuildInputs = [ pkgs.pkg-config ];
                };
          };

          devShells.default = pkgs.mkShell {
            packages = builtins.attrValues {
              inherit (pkgs)
                rustToolchain

                nixd
                nixfmt
                ;
            };

            env = {
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
            };

            buildInputs = [ pkgs.openssl ];
            nativeBuildInputs = [ pkgs.pkg-config ];
          };

          # TODO: Use treefmt-nix.
          formatter = pkgs.nixfmt-tree;
        };
    };
}
