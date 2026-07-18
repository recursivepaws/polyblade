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

        # wasm-bindgen's CLI version must match the `wasm-bindgen` crate version
        # resolved in Cargo.lock exactly, or `dx` fails at build time with a
        # "schema version mismatch" error.
        wasmBindgenCli = pkgs.wasm-bindgen-cli_0_2_126;

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        # dioxus-cli itself links against tao/wry (the desktop webview crates) for its bundler/preview tooling,
        # even when you only ever target `--platform web`.
        # Per https://dioxuslabs.com/learn/0.7/getting_started/#linux these are required to build (and run) `dx` on Linux at all.
        webviewLibs = with pkgs; [
          webkitgtk_4_1
          glib
          gtk3
          libsoup_3
          xdotool
          librsvg
          libayatana-appindicator
        ];

        # Runtime libraries for the winit/wgpu native renderer (dx serve --native).
        runtimeLibs = with pkgs; [
          wayland
          wayland-protocols
          libxkbcommon
          vulkan-loader
          libGL
          fontconfig
          libx11
          libxcursor
          libxi
          libxrandr
          libxxf86vm
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs =
            with pkgs;
            [
              rustToolchain
              pkg-config
              dioxus-cli
              wasmBindgenCli
              binaryen
              tailwindcss
              mold
              lld
              nasm
            ]
            ++ webviewLibs;

          buildInputs = with pkgs; [ openssl ] ++ webviewLibs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (webviewLibs ++ runtimeLibs);

          shellHook = ''
            export CARGO_TARGET_DIR="$PWD/target"

            echo ""
            echo "polyblade dev shell ready:"
            echo "  dx serve --platform web                                          # run in browser"
            echo "  dx serve --platform linux --renderer native                      # run as native window"
            echo "  tailwindcss -i ./tailwind.css -o ./assets/tailwind.css --watch  # css"
          '';
        };
      }
    );
}
