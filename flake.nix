{
  description = "Sync your Last.fm or ListenBrainz listening status to Stoat.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.treefmt-nix.flakeModule ];

      systems = inputs.nixpkgs.lib.systems.flakeExposed;

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
              (_: _: {
                rustToolchain = inputs.fenix.packages.${system}.fromToolchainFile {
                  file = ./rust-toolchain.toml;
                  sha256 = "sha256-zC8E38iDVJ1oPIzCqTk/Ujo9+9kx9dXq7wAwPMpkpg0=";
                };
              })
            ];
          };

          packages =
            let
              cargoTOML = builtins.fromTOML (builtins.readFile ./Cargo.toml);
              version = cargoTOML.workspace.package.version;

              rustPlatform = pkgs.makeRustPlatform {
                cargo = pkgs.rustToolchain;
                rustc = pkgs.rustToolchain;
              };
              rustPlatformMusl = pkgs.pkgsMusl.makeRustPlatform {
                cargo = pkgs.rustToolchain;
                rustc = pkgs.rustToolchain;
              };

              lureBuildArgs = {
                pname = "lure";
                inherit version;

                src = inputs.self;
                cargoLock.lockFile = ./Cargo.lock;

                meta = {
                  mainProgram = "lure";

                  description = "Sync your Last.fm or ListenBrainz listening status to Stoat.";
                  homepage = "https://github.com/catuhana/lure";
                  license = lib.licenses.bsd3;
                };
              };
            in
            rec {
              default = lure;

              lure = rustPlatform.buildRustPackage lureBuildArgs;
              lure-musl = rustPlatformMusl.buildRustPackage lureBuildArgs;

              docker = pkgs.dockerTools.streamLayeredImage {
                name = "lure";
                tag = version;

                config = {
                  Cmd = [
                    "${lib.getExe lure}"
                    "start"
                  ];
                  WorkingDir = "/data/lure";

                  # Env = [ "LURE_LOG=info" ];

                  Volumes = {
                    "/data/lure" = { };
                  };
                };
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
          };

          treefmt = {
            programs = {
              rustfmt = {
                enable = true;

                package = pkgs.rustToolchain;
              };

              yamlfmt = {
                enable = true;

                settings.formatter.retain_line_breaks = true;
              };

              mdformat.enable = true;
              nixfmt.enable = true;
            };
          };
        };
    };
}
