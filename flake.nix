{
  description = "Zotero database FUSE filesystem";

  inputs = {
    nixpkgs = {
      type = "indirect";
      id = "nixpkgs";
    };
    flake-utils = {
      type = "indirect";
      id = "flake-utils";
    };
  };

  outputs = { self, nixpkgs, flake-utils }:
  flake-utils.lib.eachDefaultSystem (system:
    let pkgs = nixpkgs.legacyPackages."${system}";
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          cargo
          rustc
          libiconv
        ];
      };

      packages.default = pkgs.rustPlatform.buildRustPackage {
        pname = "zoterofs";
        version = "0.0.1";
        buildInputs = with pkgs; [
          macfuse-stubs
        ];
        src = ./.;
        cargoHash = "sha256-zDkARXz0nTSnwykdGShpLh3Sz7N8CRhMf1baiklhnRM=";
        meta = {
          description = "Zotero database FUSE filesystem";
          homepage = "https://github.com/vladidobro/zoterofs";
          license = pkgs.lib.licenses.gpl3;
          maintainers = [];
        };
      };
    }
  );
}
