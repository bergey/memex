{
  description = "books, papers, reading lists, notes";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst";
  };

  outputs = {self, nixpkgs}: {
    packages = builtins.mapAttrs (system: pkgs: {
      memex-server = pkgs.rustPlatform.buildRustPackage rec {
        pname = "memex-server";
        version = "0.1";
        src = ./.;
        buildAndTestSubdir = "server";
        buildNoDefaultFeatures = true;
        nativeBuildInputs = [pkgs.pkg-config pkgs.mold];
        nativeCheckInputs = [
          pkgs.postgresql
          pkgs.postgresqlTestHook
        ];
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
    }) nixpkgs.legacyPackages;

    overlays.default = final: prev: {
      inherit (self.packages) memex-server;
    };
  };
}
