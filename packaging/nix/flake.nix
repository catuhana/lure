{
  # FIXME: Fill here.
  description = "";
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable;

    flake-parts.url = github:hercules-ci/flake-parts;

    crane.url = github:ipetkov/crane;
    rust-overlay = {
      url = github:oxalica/rust-overlay;
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    crane,
    rust-overlay,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;}
    {
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem = {
        self,
        lib,
        pkgs,
        system,
        ...
      }: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        projectRoot = ../..;
        crateDirs = [
          "lure"
          "lure-config"
          "lure-lastfm-models"
          "lure-lastfm-service"
          "lure-lastfm-service-config"
          "lure-listenbrainz-models"
          "lure-listenbrainz-service"
          "lure-listenbrainz-service-config"
          "lure-resources"
          "lure-revolt-api"
          "lure-revolt-models"
          "lure-service-common"
        ];

        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);

        src = craneLib.cleanCargoSource projectRoot;

        commonArgs = {
          inherit src;

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          strictDeps = true;

          buildInputs = with pkgs;
            [
              openssl
            ]
            ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
              CoreFoundation
              Security
            ]);
          nativeBuildInputs = with pkgs; [pkg-config];
        };

        individualCrateArgs = cratePath:
          commonArgs
          // {
            inherit (craneLib.crateNameFromCargoToml {cargoToml = cratePath + /Cargo.toml;}) pname;
            inherit (craneLib.crateNameFromCargoToml {cargoToml = ../../Cargo.toml;}) version;

            src = fileSetForCrate cratePath;
          };

        getCratePath = crate: projectRoot + /${crate};
        fileSetForCrate = crate:
          lib.fileset.toSource {
            root = projectRoot;
            fileset = lib.fileset.unions ([
                (projectRoot + /Cargo.lock)
                (projectRoot + /Cargo.toml)
              ]
              ++ (map (dir: craneLib.fileset.commonCargoSources (getCratePath dir)) crateDirs)
              ++ [
                (getCratePath "lure-resources" + "/resources/config.sample.yaml")
              ]);
          };

        buildCrate = cratePath:
          craneLib.buildPackage (
            individualCrateArgs (getCratePath cratePath)
            // {
              cargoExtraArgs = "--bin ${cratePath}";
            }
          );

        lure = buildCrate "lure";
      in {
        checks = {
          inherit lure;

          fmt = craneLib.cargoFmt {
            inherit src;
          };

          clippy = craneLib.cargoClippy (commonArgs
            // {
              cargoClippyExtraArgs = "--all-targets -- -D warnings";
            });

          test = craneLib.cargoTest commonArgs;
        };

        packages.default = lure;

        apps.default = {
          type = "app";
          program = "${lure}/bin/lure";
        };
      };
    };
}
