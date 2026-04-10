{
  description = "Polyblade development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        cargo-binstall = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cargo-binstall";
          version = "1.16.0";
          src = pkgs.fetchCrate {
            inherit pname version;
            hash = "sha256-YvNoAFI8Sx34Gl1+V0dUGsYglqGIUp0HSwi9cdlW7FU=";
          };
          cargoHash = "sha256-9w17tZ8vkqyebtPK4LzaGELeb15pLGMwWzr7DFyYVms=";
          nativeBuildInputs = with pkgs; [ pkg-config openssl.dev ];
          buildInputs = with pkgs; [
            openssl
            glib
            gtk3
            libsoup_3
            webkitgtk_4_1
            xdotool
          ];
        };
        runtimeLibs = with pkgs; [
          wayland
          wayland-protocols
          libxkbcommon
          vulkan-loader
          libGL
          libx11
          libxcursor
          libxi
          libxrandr
          libxxf86vm
        ];
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            cargo
            rust-analyzer
            rustfmt
            clippy
            rustc
            lld
            graphviz
            cargo-binstall
          ];

          buildInputs = with pkgs;
            [ vulkan-headers vulkan-validation-layers ] ++ runtimeLibs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
          VK_LAYER_PATH =
            "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
        };
      });
}
