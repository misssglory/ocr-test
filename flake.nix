{
  description = "Full-screen capture with client-side Tesseract OCR and remote text receiver";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          runtimeInputs = with pkgs; [
            tesseract
            dbus
            pipewire
            wayland
            libxkbcommon
            libGL
            libglvnd
            mesa
            xorg.libxcb
            xorg.libXrandr
          ];
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              pkg-config
              clang
              cmake
              curl
              jq
            ] ++ runtimeInputs;
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeInputs;
            RUST_LOG = "info";
          };
        });
    };
}
