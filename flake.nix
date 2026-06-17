{
  description = "flibrarian";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    ejectest = {
      url = "github:mlavrinenko/ejectest";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    linecop = {
      url = "github:mlavrinenko/linecop";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ self, ... }:
    let
      supportedSystems = [
        "aarch64-linux"
        "i686-linux"
        "x86_64-linux"
      ];
      forAllSystems = inputs.nixpkgs.lib.genAttrs supportedSystems;
      nixpkgsFor = forAllSystems (system: import inputs.nixpkgs { inherit system; });
      duckdbWithFts =
        system:
        let
          pkgs = nixpkgsFor.${system};
          ftsSrc = pkgs.fetchFromGitHub {
            owner = "duckdb";
            repo = "duckdb-fts";
            rev = "39376623630a968154bef4e6930d12ad0b59d7fb";
            hash = "sha256-fK5bMckI24Sz4UueB/pPlXKyKFwVgYhtC/PqAeLN5HQ=";
          };
          extensionsCmake = pkgs.writeText "duckdb-extensions.cmake" ''
            duckdb_extension_load(autocomplete)
            duckdb_extension_load(core_functions)
            duckdb_extension_load(fts
                SOURCE_DIR ${ftsSrc}
                INCLUDE_DIR ${ftsSrc}/extension/fts/include
            )
            duckdb_extension_load(icu)
            duckdb_extension_load(json)
            duckdb_extension_load(parquet)
          '';
        in
        pkgs.duckdb.overrideAttrs (old: {
          cmakeFlags = builtins.map (
            flag:
            if (builtins.match ".*DUCKDB_EXTENSION_CONFIGS.*" (builtins.toString flag)) != null then
              (pkgs.lib.cmakeFeature "DUCKDB_EXTENSION_CONFIGS" "${extensionsCmake}")
            else
              flag
          ) old.cmakeFlags;
        });
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};

          frontendNodeModules = pkgs.stdenvNoCC.mkDerivation {
            name = "flibrarian-frontend-node-modules";
            src = ./frontend;
            nativeBuildInputs = [ pkgs.bun ];
            impureEnvVars = pkgs.lib.fetchers.proxyImpureEnvVars;
            dontFixup = true;
            buildPhase = ''
              export HOME=$TMPDIR
              bun install --frozen-lockfile
            '';
            installPhase = ''
              cp -r node_modules $out
            '';
            outputHashMode = "recursive";
            outputHashAlgo = "sha256";
            outputHash = "sha256-3XTe7EjYUWh2aSGYmwLgmuGspj49/WFHQhU40dRKDWA=";
          };

          frontend = pkgs.stdenvNoCC.mkDerivation {
            pname = "flibrarian-frontend";
            version = "0.1.0";
            src = ./frontend;
            nativeBuildInputs = [ pkgs.bun pkgs.nodejs_22 ];
            configurePhase = ''
              export HOME=$TMPDIR
              cp -r ${frontendNodeModules} node_modules
              chmod -R u+w node_modules
              patchShebangs node_modules
            '';
            buildPhase = ''
              bun run build
            '';
            installPhase = ''
              cp -r dist $out
            '';
          };
        in {
        default =
          let duckdb = duckdbWithFts system;
          in pkgs.rustPlatform.buildRustPackage {
          pname = "flibrarian";
          version = "0.1.0";
          src = ./.;
          cargoHash = "sha256-ZMLvpzVHjUBZSj7xU34SFo3CsIkcCXmLCR0hUgdGGQM=";
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ duckdb.lib duckdb.dev ];
          DUCKDB_LIB_DIR = "${duckdb.lib}/lib";
          DUCKDB_INCLUDE_DIR = "${duckdb.dev}/include";
        };

        web =
          let duckdb = duckdbWithFts system;
          in pkgs.rustPlatform.buildRustPackage {
          pname = "flibrarian-web";
          version = "0.1.0";
          src = ./.;
          cargoHash = "sha256-ZMLvpzVHjUBZSj7xU34SFo3CsIkcCXmLCR0hUgdGGQM=";
          cargoBuildFlags = [ "-p" "flibrarian-web" ];
          cargoTestFlags = [ "-p" "flibrarian-web" ];
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ duckdb.lib duckdb.dev ];
          DUCKDB_LIB_DIR = "${duckdb.lib}/lib";
          DUCKDB_INCLUDE_DIR = "${duckdb.dev}/include";
          preBuild = ''
            rm -rf frontend/dist
            cp -r ${frontend} frontend/dist
          '';
        };
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
          duckdb = duckdbWithFts system;
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rustc
              cargo
              clippy
              rustfmt
              rust-analyzer
              pkg-config
              cargo-tauri
              wrapGAppsHook4
              bun
              nodejs_22
              mold
              clang
              sccache
            ];
            buildInputs = [
              pkgs.openssl
              duckdb.lib
              duckdb.dev
              pkgs.libiconv
              pkgs.webkitgtk_4_1
              pkgs.gtk3
              pkgs.cairo
              pkgs.gdk-pixbuf
              pkgs.glib
              pkgs.dbus
              pkgs.librsvg
              pkgs.libsoup_3
              pkgs.just
              pkgs.parallel
              pkgs.valgrind
              self.inputs.ejectest.packages.${system}.default
              self.inputs.linecop.packages.${system}.default
            ];
            shellHook = ''
              export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH"
              export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang"
              export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
              export RUSTC_WRAPPER="$(command -v sccache)"
              export DUCKDB_LIB_DIR="${duckdb.lib}/lib"
              export DUCKDB_INCLUDE_DIR="${duckdb.dev}/include"
            '';
          };
        }
      );
    };
}
