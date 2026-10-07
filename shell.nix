# Native GPUI, OCCT and GPU loader dependencies for desktop modeling.
{ pkgs ? import <nixpkgs> {} }:
let
  nativeLibraries = with pkgs; [
    libX11 libXi libXtst libXcursor libXrandr libXinerama
    libxkbcommon wayland fontconfig freetype vulkan-loader libGL opencascade-occt
  ];
in pkgs.mkShell {
  packages = with pkgs; [ pkg-config ] ++ nativeLibraries;
  OCCT_INCLUDE_DIR = "${pkgs.opencascade-occt}/include/opencascade";
  OCCT_LIB_DIR = "${pkgs.opencascade-occt}/lib";
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath nativeLibraries}:/run/opengl-driver/lib";
}
