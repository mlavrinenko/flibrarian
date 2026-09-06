{
  description = "flibrarian";

  nixConfig = {
    extra-substituters = [ "https://mlavrinenko.cachix.org" ];
    extra-trusted-public-keys = [
      "mlavrinenko.cachix.org-1:vNcY3Nf5Y1J0D30uNAwrw44CBHbHDd1tGiA18ANz4XY="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    qahq.url = "github:mlavrinenko/qahq";
  };

  outputs =
    inputs@{ self, ... }:
    let
      # i686 is absent on purpose: DuckDB publishes prebuilt extensions only for
      # linux_amd64 and linux_arm64, and we no longer compile them from source.
      supportedSystems = [
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = inputs.nixpkgs.lib.genAttrs supportedSystems;
      nixpkgsFor = forAllSystems (system: import inputs.nixpkgs { inherit system; });

      duckdbPlatforms = {
        x86_64-linux = "linux_amd64";
        aarch64-linux = "linux_arm64";
      };

      # `fts` left the DuckDB tree in 1.3, so nixpkgs' duckdb no longer links it.
      # Rebuilding duckdb to link it back cost ~50 min of C++ per nixpkgs bump and
      # could never be substituted from any binary cache. Instead we keep the
      # cached stock duckdb and drop DuckDB's own signed extension build into an
      # `extension_directory` tree, which `LOAD fts;` resolves at runtime.
      #
      # An extension binary only loads into the exact duckdb release it was built
      # for, so the table is keyed by version: consumers that override our nixpkgs
      # (e.g. a NixOS config with `inputs.nixpkgs.follows`) resolve a different
      # duckdb than our own lock does, and both must be listed. An unknown version
      # is a hard eval error naming the fix, never a silent fallback.
      ftsHashes = {
        "1.4.4" = {
          linux_amd64 = "sha256-gR/HsDZ/H8+pE21u5swUvghRXd/LAa3zuu50dKs8HI4=";
          linux_arm64 = "sha256-Ql18Haj2sfGaSwE4KTdoSnuNUZPNuf8KCcFkkfUS1Uk=";
        };
        "1.5.4" = {
          linux_amd64 = "sha256-vsLUQR/YJHBNY5P38znH48pAOhx1BkMlvOxmSMi0+Ec=";
          linux_arm64 = "sha256-cqhnM9KNrRpNbM5fY9fSQ9Ymumrb7z3ttl5uAD83nnQ=";
        };
        "1.5.5" = {
          linux_amd64 = "sha256-kNbwSeWbWSVmz80ijeMAHrZ5xk6fFEwTjcLNVdqxLNY=";
          linux_arm64 = "sha256-h6jC3d9B05fGF69B5HnU42XdZqnxFc7H54N0BX6AR48=";
        };
      };

      duckdbExtensionsFor =
        system:
        let
          pkgs = nixpkgsFor.${system};
          inherit (pkgs.duckdb) version;
          platform = duckdbPlatforms.${system};
          hashes =
            ftsHashes.${version} or (throw ''
              No pinned duckdb fts extension hash for duckdb ${version}.
              Add it to `ftsHashes` in flake.nix, e.g.:
                nix-prefetch-url https://extensions.duckdb.org/v${version}/${platform}/fts.duckdb_extension.gz
            '');
          fts = pkgs.fetchurl {
            url = "https://extensions.duckdb.org/v${version}/${platform}/fts.duckdb_extension.gz";
            hash = hashes.${platform};
          };
        in
        pkgs.runCommand "duckdb-extensions-${version}-${platform}" { } ''
          mkdir -p $out/v${version}/${platform}
          gzip -dc ${fts} > $out/v${version}/${platform}/fts.duckdb_extension
        '';
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};
          inherit (pkgs) duckdb;
          duckdbExtensions = duckdbExtensionsFor system;

          # Shared by every Rust package here: link the cached stock duckdb, and
          # point both `cargo test` and the installed binaries at the extension
          # tree. --set-default leaves an operator override possible.
          duckdbEnv = {
            nativeBuildInputs = [ pkgs.pkg-config pkgs.makeWrapper ];
            buildInputs = [ duckdb.lib duckdb.dev ];
            DUCKDB_LIB_DIR = "${duckdb.lib}/lib";
            DUCKDB_INCLUDE_DIR = "${duckdb.dev}/include";
            FLIBRARIAN_DUCKDB_EXTENSION_DIR = "${duckdbExtensions}";
            postInstall = ''
              for bin in "$out"/bin/*; do
                wrapProgram "$bin" \
                  --set-default FLIBRARIAN_DUCKDB_EXTENSION_DIR ${duckdbExtensions}
              done
            '';
          };

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
            version = "0.1.1";
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
        default = pkgs.rustPlatform.buildRustPackage (
          duckdbEnv
          // {
            pname = "flibrarian";
            version = "0.1.1";
            src = ./.;
            cargoHash = "sha256-ZMLvpzVHjUBZSj7xU34SFo3CsIkcCXmLCR0hUgdGGQM=";
            # CLI only. A workspace-wide build drags in flibrarian-gui, whose
            # gio-sys/webkitgtk stack is not packaged here; the GUI ships through
            # GitHub Releases instead.
            cargoBuildFlags = [ "-p" "flibrarian-cli" ];
            cargoTestFlags = [ "-p" "flibrarian-core" "-p" "flibrarian-cli" ];
          }
        );

        web = pkgs.rustPlatform.buildRustPackage (
          duckdbEnv
          // {
            pname = "flibrarian-web";
            version = "0.1.1";
            src = ./.;
            cargoHash = "sha256-ZMLvpzVHjUBZSj7xU34SFo3CsIkcCXmLCR0hUgdGGQM=";
            cargoBuildFlags = [ "-p" "flibrarian-web" ];
            cargoTestFlags = [ "-p" "flibrarian-web" ];
            preBuild = ''
              rm -rf frontend/dist
              cp -r ${frontend} frontend/dist
            '';
          }
        );
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
          inherit (pkgs) duckdb;
          duckdbExtensions = duckdbExtensionsFor system;
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
              self.inputs.qahq.packages.${system}.ejectest
              self.inputs.qahq.packages.${system}.linecop
            ];
            shellHook = ''
              export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH"
              export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang"
              export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
              # RUSTC_WRAPPER comes from the host session (global kache); inherited
              # here, and it never reaches the `nix build` sandbox.
              export DUCKDB_LIB_DIR="${duckdb.lib}/lib"
              export DUCKDB_INCLUDE_DIR="${duckdb.dev}/include"
              export FLIBRARIAN_DUCKDB_EXTENSION_DIR="${duckdbExtensions}"
            '';
          };
        }
      );
    };
}
