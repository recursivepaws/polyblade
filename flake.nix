{
  description = "Polyblade (dioxus) development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        dioxusCliVersion = "0.7.0-alpha.2";

        # wasm-bindgen's CLI version must match the `wasm-bindgen` crate version
        # resolved in Cargo.lock exactly, or `dx` fails at build time with a
        # "schema version mismatch" error.
        wasmBindgenCli = pkgs.wasm-bindgen-cli_0_2_100;

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # dioxus-cli itself links against tao/wry (the desktop webview crates)
        # for its bundler/preview tooling, even when you only ever target
        # `--platform web`. Per https://dioxuslabs.com/learn/0.7/getting_started/#linux
        # these are required to build (and run) `dx` on Linux at all.
        webviewLibs = with pkgs; [
          webkitgtk_4_1
          glib
          gtk3
          libsoup_3
          xdotool
          librsvg
          libayatana-appindicator
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs =
            with pkgs;
            [
              rustToolchain
              pkg-config
              cargo-binstall
              wasmBindgenCli
              binaryen # provides wasm-opt, used by `dx bundle --release`
              tailwindcss # v3 standalone CLI, matches tailwind.config.js
              mold
              lld
              # Asset pipeline (manganis) transitively optimizes images and
              # occasionally needs a codec assembler at build time.
              nasm
            ]
            ++ webviewLibs;

          buildInputs = with pkgs; [ openssl ] ++ webviewLibs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath webviewLibs;

          shellHook = ''
            export CARGO_TARGET_DIR="$PWD/target"

            # Installed project-locally (not ~/.cargo/bin): mkShell's PATH
            # doesn't reliably inherit the user's global cargo bin dir
            # (especially under nix-direnv), and this also keeps the pinned
            # alpha `dx` from clashing with other dioxus projects on disk.
            export DX_BIN_DIR="$PWD/.bin"
            mkdir -p "$DX_BIN_DIR"
            export PATH="$DX_BIN_DIR:$PATH"

            if ! command -v dx >/dev/null 2>&1 || \
                [ "$(dx --version 2>/dev/null | awk '{print $2}')" != "${dioxusCliVersion}" ]; then
              echo "installing dioxus-cli ${dioxusCliVersion} (pinned to match Cargo.toml) ..."
              echo "(this will build from source if no prebuilt binary exists for this alpha tag - can take a while)"
              cargo binstall -y --locked --install-path "$DX_BIN_DIR" "dioxus-cli@${dioxusCliVersion}"
            fi

            echo ""
            echo "polyblade dev shell ready:"
            echo "  dx serve --platform web                                          # run the app"
            echo "  tailwindcss -i ./tailwind.css -o ./assets/tailwind.css --watch  # css"
          '';
        };
      }
    );
}
