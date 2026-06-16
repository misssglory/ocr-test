{
  description = "Rust screen capture to remote Tesseract OCR bridge";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          nativeInputs = with pkgs; [
            cargo
            rustc
            pkg-config
            clang
            cmake
          ];
          runtimeInputs = with pkgs; [
            tesseract
            dbus
            pipewire
            wayland
            libxkbcommon
            libGL
            libglvnd
            mesa
            libgbm  # Add this - provides GBM library
            wayland-protocols  # Add this - often needed for wayland capture
            xorg.libxcb
            xorg.libXrandr
          ];
        in {
          default = pkgs.mkShell {
            packages = nativeInputs ++ runtimeInputs;
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeInputs;
            # Add explicit library paths for pkg-config
            PKG_CONFIG_PATH = pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" runtimeInputs;
            RUST_LOG = "info";
          };
        });
    };
}