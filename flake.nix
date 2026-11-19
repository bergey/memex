{
  description = "books, papers, reading lists, notes";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {self, nixpkgs, rust-overlay}: {
    packages = builtins.mapAttrs (system: _:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        systems = {"x86_64-linux" = "";} ; # extend if I test on other systems
        # rust-bin = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rust-bin = pkgs.rust-bin.stable.latest.minimal;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rust-bin;
          rustc = rust-bin;
        };
      in {
      memex-server = rustPlatform.buildRustPackage rec {
        pname = "memex-server";
        version = "0.1";
        src = ./.;
        buildAndTestSubdir = "server";
        buildNoDefaultFeatures = true;
        nativeBuildInputs = [pkgs.pkg-config pkgs.mold];
        doCheck = false;
        # nativeCheckInputs = [
        #   pkgs.postgresql
        #   pkgs.postgresqlTestHook
        # ];
        # DATABASE_URL = "postgres:///$PGDATABASE?user=$PGUSER&host=$PGHOST";
        # DATABASE_URL = "postgres:///test_db?user=test_user";
        cargoLock = {
          lockFile = ./Cargo.lock;

          outputHashes = {
            "webauthn-attestation-ca-0.6.2" = "sha256-V+jnyLMH+FpoJ7ICzkxczlu0YHdk7QOn/5Ehxmxq5c0=";
          };
        };
      };

      default = self.packages.${system}.memex-server;
      }) systems;

    overlays.default = final: prev: {
      inherit (self.packages) memex-server;
    };
  };
}
