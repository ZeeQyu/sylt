{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell rec {
  nativeBuildInputs = with pkgs;[
    pkg-config
    rustup
    # To use lld linker
    clang
    llvmPackages.bintools
  ];
  buildInputs = with pkgs; [
    udev
    alsaLib
    vulkan-loader
    # To use x11 feature
    xlibsWrapper
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    # To use wayland feature
    libxkbcommon
    wayland
  ];
  RUSTC_VERSION =
    builtins.elemAt
      (builtins.match
        ".*channel *= *\"([^\"]*)\".*"
        (pkgs.lib.readFile ./rust-toolchain.toml)
      )
      0;
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
  shellHook = ''
    rustup toolchain install ''${RUSTC_VERSION}
  '';
}
