{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      supportedSystems =
        [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      pkgsFor = system:
        import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

      commonBuildInputs = pkgs:
        with pkgs; {
          nativeBuildInputs = [ pkg-config ];

          buildInputs = [ openssl ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin
            (with pkgs.darwin.apple_sdk.frameworks; [
              Security
              SystemConfiguration
              CoreFoundation
            ]);
        };

      buildPackageFor = system:
        let
          pkgs = pkgsFor system;
          cargoTOML = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          buildInputs = commonBuildInputs pkgs;
        in pkgs.rustPlatform.buildRustPackage ({
          pname = cargoTOML.package.name;
          version = cargoTOML.package.version;
          src = ./.;

          cargoLock = { lockFile = ./Cargo.lock; };
        } // buildInputs);
    in {
      packages = forAllSystems (system: { default = buildPackageFor system; });

      nixosModules.default = import ./nix/modules/lure.nix;
    };
}
