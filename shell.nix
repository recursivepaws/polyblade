{ pkgs ? import <nixpkgs> { } }:

let
  dioxus-cli = pkgs.rustPlatform.buildRustPackage rec {
    pname = "dioxus-cli";
    version = "0.7.1";

    /* src = pkgs.fetchFromGitHub {
         owner = "DioxusLabs";
         repo = "dioxus";
         rev = "v${version}";
         sha256 = "sha256-EzfuD3rWVuomyzqSv4b3SVA6MmQiWAeePbdfXEvkiRk=";
       };
    */
    src = pkgs.fetchCrate {
      inherit pname version;
      hash = "sha256-YvNoAFI8Sx34Gl1+V0dUGsYglqGIUp0HSwi9cdlW7FU=";
    };

    cargoHash = "sha256-YvNoAFI8Sx34Gl1+V0dUGsYglqGIUp0HSwi9cdlW7FU=";
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
  cargo-binstall = pkgs.rustPlatform.buildRustPackage rec {
    pname = "cargo-binstall";
    version = "1.16.0";

    /* src = pkgs.fetchFromGitHub {
         owner = "DioxusLabs";
         repo = "dioxus";
         rev = "v${version}";
         sha256 = "sha256-EzfuD3rWVuomyzqSv4b3SVA6MmQiWAeePbdfXEvkiRk=";
       };
    */
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
in pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    cargo
    rust-analyzer
    rustfmt
    rustc
    lld
    graphviz
    cargo-binstall
  ];

  buildInputs = with pkgs; [
    # Wayland
    wayland
    libxkbcommon

    # Graphics
    vulkan-loader
    vulkan-headers
    vulkan-validation-layers
    libGL

    # X11
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    xorg.libXxf86vm
  ];

  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [
    pkgs.wayland
    pkgs.libxkbcommon
    pkgs.vulkan-loader
    pkgs.libGL
    pkgs.xorg.libX11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
  ]}";

  VK_LAYER_PATH =
    "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
}
