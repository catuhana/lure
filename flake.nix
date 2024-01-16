{
  description = "lure is a small _daemon_ that sets the currently playing track on Last.fm, ListenBrainz (and other platforms) as Revolt user status.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
    in
    with pkgs;
    {
      devShells.default = mkShell {
        nativeBuildInputs = [ rust-bin.stable.latest.default pkg-config ];
        buildInputs = [ openssl ];

        packages = [ nixpkgs-fmt ];
      };

      packages.default = (makeRustPlatform {
        cargo = rust-bin.stable.latest.minimal;
        rustc = rust-bin.stable.latest.minimal;
      }).buildRustPackage {
        pname = "lure";
        version = "0.3.0";

        src = ./.;

        cargoLock = { lockFile = ./Cargo.lock; allowBuiltinFetchGit = true; };

        nativeBuildInputs = [ pkg-config ];
        buildInputs = [ openssl ];

        meta = with lib; {
          description = "A small _daemon_ that sets the currently playing track on Last.fm, ListenBrainz (and other platforms) as Revolt user status.";
          homepage = "https://github.com/catuhana/lure";
          license = with licenses; [ mpl20 ];
        };
      };

      nixosModules = {
        lure = import ./nix/module.nix { inherit pkgs; };
        default = self.nixosModules.lure;
      };
    }
  );
}
