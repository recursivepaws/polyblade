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
        # macOS uses its native WebKit.framework instead, so these are Linux-only (webkitgtk is marked broken on Darwin in nixpkgs).
        webviewLibs = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
          webkitgtk_4_1
          glib
          gtk3
          libsoup_3
          xdotool
          librsvg
          libayatana-appindicator
        ]);

        # Runtime libraries for the winit/wgpu native renderer (dx serve --native). Linux-only (X11/Wayland/Vulkan).
        runtimeLibs = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
          wayland
          wayland-protocols
          libxkbcommon
          vulkan-loader
          libGL
          fontconfig
          libx11
          libxcb
          libxcursor
          libxi
          libxrandr
          libxxf86vm
        ]);

        # `dx bundle --package-types appimage` needs `linuxdeploy`, but nixpkgs' `dioxus-cli` is
        # built with the `no-downloads` feature (Nix's model forbids silent runtime network
        # fetches), so it never auto-downloads it. `dx` checks `~/.dx/tools/linuxdeploy-{arch}.AppImage`
        # *before* ever considering a download, so pre-seeding that path from a pinned Nix fetch
        # (see shellHook below) satisfies it without needing that restriction lifted.
        linuxdeployAppImage = pkgs.fetchurl {
          url = "https://github.com/tauri-apps/binary-releases/releases/download/linuxdeploy/linuxdeploy-x86_64.AppImage";
          sha256 = "sha256-52K+qFyOsNSzUI1G5cHwN/cX0PkwOuO0qvyLBJkfoe8=";
        };
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
              lld
              nasm
              cargo-deny
            ]
            ++ webviewLibs;

          buildInputs = with pkgs; [ openssl ] ++ webviewLibs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (webviewLibs ++ runtimeLibs);

          shellHook = ''
            export CARGO_TARGET_DIR="$PWD/target"

            ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
              # dx resolves its data dir per-platform (XDG data dir on Linux, unless
              # DX_HOME is set) - pin it explicitly so the linuxdeploy seed below
              # always lands where `dx bundle` will actually look for it.
              export DX_HOME="$HOME/.dx"
              mkdir -p "$DX_HOME/tools"
              if [ ! -e "$DX_HOME/tools/linuxdeploy-x86_64.AppImage" ]; then
                install -m 0755 ${linuxdeployAppImage} "$DX_HOME/tools/linuxdeploy-x86_64.AppImage"
              fi
            ''}

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
