{
  description = "Display your currently playing track from Last.fm, ListenBrainz, and other services in your Revolt status.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, nixpkgs, flake-parts, fenix, ... }: flake-parts.lib.mkFlake { inherit inputs; }
    {
      systems = nixpkgs.lib.systems.flakeExposed;

      perSystem = { self', lib, pkgs, system, ... }:
        let
          lurePackage =
            let
              cargoTOML = lib.importTOML ./Cargo.toml;
              toolchain = fenix.packages.${system}.minimal.toolchain;
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
        in
        {
          packages.default = lurePackage;
        };

      flake.nixosModules.default = import ./nix/modules/lure.nix self.packages.${nixpkgs.system}.default;
    };
}

# {
#   description = "Display your currently playing track from Last.fm, ListenBrainz, and other services in your Revolt status.";

#   inputs = {
#     nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
#     rust-overlay.url = "github:oxalica/rust-overlay";
#     flake-parts.url = "github:hercules-ci/flake-parts";
#   };

#   outputs = inputs@{ rust-overlay, flake-parts, ... }: flake-parts.lib.mkFlake { inherit inputs; } {
#     systems = inputs.nixpkgs.lib.systems.flakeExposed;

#     perSystem = { self', lib, pkgs, system, ... }:
#       let
#         cargoTOML = lib.importTOML ./Cargo.toml;
#       in
#       {
#         _module.args.pkgs = import inputs.nixpkgs {
#           inherit system;
#           overlays = [ (import rust-overlay) ];
#         };

#         packages.default = pkgs.callPackage ./nix/package.nix {
#           inherit cargoTOML;
#           rust = pkgs.rust-bin.stable.latest.minimal;
#         };
#       };

#     # nixosModules.default = import ./nix/modules/lure.nix;
#   };
# }
