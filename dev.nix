# Nix development environment for the Omni project
# See https://firebase.google.com/docs/studio/customize-workspace for details
{ pkgs, ... }:
{
  # Use the latest stable Nixpkgs channel
  channel = "stable-24.05";

  # Packages required to build the Rust CLI/GUI
  packages = [
    pkgs.rustup
    pkgs.pkg-config
    pkgs.openssl
    pkgs.openssl.dev
    pkgs.clang
    pkgs.cmake
    pkgs.alsa-lib
    pkgs.vulkan-loader
    pkgs.libxkbcommon
    pkgs.wayland
    pkgs.python3
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
    pkgs.nodejs # Added Node.js
  ];

  env = {
    # Environment variables for development
  };

  idx = {
    # Useful editor extensions
    extensions = [
      "rust-lang.rust-analyzer"
    ];

    previews = {
      enable = true;
      previews = {};
    };

    workspace = {
      onCreate = {};
      onStart = {};
    };
  };
}
