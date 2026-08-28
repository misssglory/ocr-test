{
  description = "Screen capture + OCR bridge with NixOS/Hyprland integration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };

          tesseractWithLanguages =
            pkgs.tesseract.override {
              enableLanguages = [
                "eng"
                "rus"
              ];
            };

      screenOcr = pkgs.rustPlatform.buildRustPackage {
        pname = "screen-ocr";
        version = "0.2.0";

        src = ./.;

        cargoLock.lockFile = ./Cargo.lock;

        cargoBuildFlags = [
          "--workspace"
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
          clang
          cmake
          findutils
        ];

        buildInputs = with pkgs; [
          dbus
          pipewire
          wayland
          wayland-protocols
          libxkbcommon

          libGL
          libglvnd
          mesa

          xorg.libxcb
          xorg.libXrandr
        ];

        LIBCLANG_PATH =
          "${pkgs.libclang.lib}/lib";

        installPhase = ''
          runHook preInstall

          mkdir -p "$out/bin"

          sender="$(
            find target \
              -type f \
              -path '*/release/screen-ocr-sender' \
              -print \
              -quit
          )"

          receiver="$(
            find target \
              -type f \
              -path '*/release/screen-ocr-receiver' \
              -print \
              -quit
          )"

          if [ -z "$sender" ]; then
            echo "ERROR: screen-ocr-sender binary not found"
            echo
            echo "Contents of target/:"
            find target -maxdepth 4 -type f -print
            exit 1
          fi

          if [ -z "$receiver" ]; then
            echo "ERROR: screen-ocr-receiver binary not found"
            echo
            echo "Contents of target/:"
            find target -maxdepth 4 -type f -print
            exit 1
          fi

          echo "Installing sender from: $sender"
          echo "Installing receiver from: $receiver"

          install \
            -Dm755 \
            "$sender" \
            "$out/bin/screen-ocr-sender"

          install \
            -Dm755 \
            "$receiver" \
            "$out/bin/screen-ocr-receiver"

          runHook postInstall
        '';
      };

          screenOcrWorker =
            pkgs.writeShellApplication {
              name = "screen-ocr-worker";

              runtimeInputs = with pkgs; [
                coreutils
                gawk
                gnused
                libnotify
                tesseractWithLanguages
              ];

              text = ''
                set -euo pipefail

                image="''${1:?usage: screen-ocr-worker IMAGE.png}"

                if [ ! -f "$image" ]; then
                  notify-send \
                    --app-name="Screen OCR" \
                    --urgency=critical \
                    "OCR failed" \
                    "Image does not exist: $image"

                  exit 1
                fi

                text_file="''${image%.*}.txt"

                if ! text="$(
                  tesseract \
                    "$image" \
                    stdout \
                    -l eng+rus \
                    --psm 6 \
                    2>/dev/null
                )"; then

                  notify-send \
                    --app-name="Screen OCR" \
                    --urgency=critical \
                    "OCR failed" \
                    "$(basename "$image")"

                  exit 1
                fi

                printf '%s\n' "$text" > "$text_file"

                lines="$(
                  printf '%s\n' "$text" |
                    awk '
                      NF {
                        count++
                      }

                      END {
                        print count + 0
                      }
                    '
                )"

                words="$(
                  printf '%s' "$text" |
                    wc -w |
                    tr -d '[:space:]'
                )"

                chars="$(
                  printf '%s' "$text" |
                    wc -m |
                    tr -d '[:space:]'
                )"

                preview="$(
                  printf '%s' "$text" |
                    tr '\n' ' ' |
                    cut -c1-180
                )"

                if [ -z "$preview" ]; then
                  preview="No text recognized"
                fi

                notify-send \
                  --app-name="Screen OCR" \
                  --icon="$image" \
                  "OCR complete" \
                  "Lines: $lines
Words: $words
Characters: $chars

$preview"
              '';
            };

          screenshotOcr =
            pkgs.writeShellApplication {
              name = "screenshot-ocr";

              runtimeInputs = with pkgs; [
                coreutils
                grim
                libnotify
                slurp
                systemd
                wl-clipboard
              ];

              text = ''
                set -euo pipefail

                output_dir="''${SCREENSHOT_DIR:-$HOME/Pictures/Screenshots}"

                mkdir -p "$output_dir"

                geometry="$(
                  slurp \
                    -d \
                    -f '%x,%y %wx%h'
                )" || exit 0

                if [ -z "$geometry" ]; then
                  exit 0
                fi

                stamp="$(
                  date +'%Y-%m-%d_%H-%M-%S'
                )"

                image="$output_dir/$stamp.png"

                #
                # One capture only.
                #
                grim \
                  -g "$geometry" \
                  "$image"

                #
                # Immediately copy PNG to Wayland clipboard.
                #
                wl-copy \
                  --type image/png \
                  < "$image"

                #
                # Screenshot is already available before OCR begins.
                #
                notify-send \
                  --app-name="Screenshot" \
                  --icon="$image" \
                  "Screenshot copied" \
                  "$image"

                #
                # Run OCR independently of the Hyprland keybind.
                #
                # systemd-run returns immediately.
                #
                systemd-run \
                  --user \
                  --collect \
                  --quiet \
                  --unit="screen-ocr-$(date +%s%N)" \
                  ${screenOcrWorker}/bin/screen-ocr-worker \
                  "$image"
              '';
            };
        in
        {
          default = screenOcr;

          screen-ocr = screenOcr;

          screen-ocr-worker =
            screenOcrWorker;

          screenshot-ocr =
            screenshotOcr;
        }
      );

      nixosModules.default =
        import ./nixos/screen-ocr.nix {
          inherit self;
        };

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };

          tesseractWithLanguages =
            pkgs.tesseract.override {
              enableLanguages = [
                "eng"
                "rus"
              ];
            };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              rust-analyzer

              pkg-config
              clang
              cmake

              curl
              jq

              tesseractWithLanguages

              dbus
              pipewire
              wayland
              wayland-protocols
              libxkbcommon

              libGL
              libglvnd
              mesa

              xorg.libxcb
              xorg.libXrandr

              grim
              slurp
              wl-clipboard
              libnotify
            ];

            LIBCLANG_PATH =
              "${pkgs.libclang.lib}/lib";

            RUST_LOG = "info";
          };
        }
      );
    };
}
