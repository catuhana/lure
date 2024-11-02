{
  description = "Display your currently playing track from Last.fm, ListenBrainz, and other services in your Revolt status.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, nixpkgs, flake-parts, gitignore, rust-overlay, ... }: flake-parts.lib.mkFlake { inherit inputs; }
    {
      systems = nixpkgs.lib.systems.flakeExposed;

      perSystem = { self', lib, pkgs, system, ... }:
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };

          packages.default =
            let
              cargoTOML = lib.importTOML ./Cargo.toml;
              rustToolchain = pkgs.rust-bin.stable.latest.minimal;
              inherit (gitignore.lib) gitignoreSource;
            in
            (pkgs.makeRustPlatform {
              cargo = rustToolchain;
              rustc = rustToolchain;
            }).buildRustPackage {
              inherit (cargoTOML.package) version;

              pname = cargoTOML.package.name;

              src = gitignoreSource ./.;
              cargoLock.lockFile = ./Cargo.lock;

              buildInputs = with pkgs; [ openssl ] ++ lib.optional stdenv.isDarwin
                (with darwin.apple_sdk.frameworks; [ CoreFoundation Security ]);
              nativeBuildInputs = with pkgs; [ pkg-config ];

              meta = {
                inherit (cargoTOML.package) description license;

                homepage = cargoTOML.package.repository;

                platforms = lib.platforms.unix;
              };
            };
        };

      flake.nixosModules.default = import ./module.nix self;
    };
}
