{
  description = "Bevy Chess";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, flake-utils, nixpkgs, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk { };
        wasm-bindgen-cli-105 = pkgs.wasm-bindgen-cli.overrideAttrs (final: prev: rec {
          src = pkgs.fetchCrate {
            pname = "wasm-bindgen-cli";
            version = "0.2.105";
            hash = "sha256-zLPFFgnqAWq5R2KkaTGAYqVQswfBEYm9x3OPjx8DJRY=";
          };

          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            inherit src;
            inherit (src) pname version;
            hash = "sha256-a2X9bzwnMWNt0fTf30qAiJ4noal/ET1jEtf5fBFj5OU=";
          };
        });

        nativeBuildInputs = with pkgs; [
          pkg-config
          rustc
          rustfmt
          cargo
          llvmPackages.lld
          wasm-bindgen-cli-105
          binaryen
        ];

        buildInputs = with pkgs; [
          udev
          alsa-lib
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          libxkbcommon
          wayland
        ];

        libraryPath = pkgs.lib.makeLibraryPath buildInputs;
      in
      rec {
        inherit wasm-bindgen-cli-105;
        # For `nix build` & `nix run`:
        chess_binary = naersk'.buildPackage {
          src = ./.;
          inherit nativeBuildInputs;
          inherit buildInputs;
          cargoBuildOptions = x: x ++ [ "--no-default-features" ];

          postInstall = ''
            cp -r assets $out/bin
          '';
        };
        chess = pkgs.stdenv.mkDerivation {
          pname = "chess";
          version = "0.1.0";

          buildInputs = [ chess_binary ];
          src = ./.;

          installPhase = ''
            mkdir -p $out/bin
            cp -r assets $out/bin
            echo '#!${pkgs.stdenv.shell}' > $out/bin/chess
            echo 'export LD_LIBRARY_PATH=${libraryPath}:$LD_LIBRARY_PATH' >> $out/bin/chess
            echo '${chess_binary}/bin/chess' >> $out/bin/chess
            chmod +x $out/bin/chess
          '';
        };
        defaultPackage = chess;

        # For `nix develop`:
        devShell = pkgs.mkShell {
          inherit nativeBuildInputs;
          inherit buildInputs;

          LD_LIBRARY_PATH = libraryPath;
        };

        # WASM
        chess-wasm_binary = naersk'.buildPackage {
          src = ./.;
          inherit nativeBuildInputs;
          inherit buildInputs;
          cargoBuildOptions = x: x ++ [
            "--no-default-features"
            "--target wasm32-unknown-unknown"
          ];
        };
        chess-wasm = pkgs.stdenv.mkDerivation {
          pname = "chess-wasm";
          version = "0.1.0";

          inherit nativeBuildInputs buildInputs;

          src = ./.;

          buildPhase = ''
            wasm-bindgen --out-dir wasm-app --target web ${chess-wasm_binary}/bin/chess.wasm
            wasm-opt -Oz --strip-debug wasm-app/chess_bg.wasm -o wasm-app/chess_bg.wasm
          '';

          installPhase = ''
            mkdir $out
            cp -r wasm-app $out/wasm
            cp -r assets $out/wasm
            cp wasm/index.html $out/wasm
            mkdir $out/bin
            cat > $out/bin/chess-wasm <<EOF
            #!/usr/bin/env bash
            ${pkgs.python3}/bin/python -m http.server -d $out/wasm 8000
            EOF
            chmod +x $out/bin/chess-wasm
          '';
        };
      }
    );
}
