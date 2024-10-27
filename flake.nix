{
  description = "Display your currently playing track from Last.fm, ListenBrainz, and other services in your Revolt status.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, nixpkgs, flake-parts, rust-overlay, ... }: flake-parts.lib.mkFlake { inherit inputs; }
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
              toolchain = pkgs.rust-bin.stable.latest.minimal;
            in
            (pkgs.makeRustPlatform {
              cargo = toolchain;
              rustc = toolchain;
            }).buildRustPackage {
              pname = cargoTOML.package.name;
              version = cargoTOML.package.version;

              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;

              buildInputs = with pkgs; [ openssl ] ++ lib.optional stdenv.isDarwin
                (with darwin.apple_sdk.frameworks; [ CoreFoundation Security ]);
              nativeBuildInputs = with pkgs; [ pkg-config ];

              meta = {
                description = "Display your currently playing track from Last.fm, ListenBrainz, and other services in your Revolt status.";
                homepage = "https://github.com/catuhana/lure";
                license = lib.licenses.mpl20;
              };
            };
        };

      flake.nixosModules.default = import ./nix/modules/lure.nix self;
    };
}
