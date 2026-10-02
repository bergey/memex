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
        buildInputs = [pkgs.openssl];
        nativeBuildInputs = [pkgs.pkg-config];
        cargoLock = {
          lockFile = ./Cargo.lock;

          outputHashes = {
            "base64urlsafedata-0.5.5" = "sha256-I16UpVkclt2SdzFA0/5dPY9uIdvTs7XFF0Ll5iKxoXE=";
          };
        };
      };

      default = self.packages.${system}.memex-server;
    }) nixpkgs.legacyPackages;
  };
}
