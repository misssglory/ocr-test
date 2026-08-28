{ self }:

{ config, lib, pkgs, ... }:

let
  cfg = config.services.screen-ocr-sender;

  system = pkgs.stdenv.hostPlatform.system;

  senderPackage =
    self.packages.${system}.screen-ocr;

  screenshotPackage =
    self.packages.${system}.screenshot-ocr;

  tesseractWithLanguages =
    pkgs.tesseract.override {
      enableLanguages = [
        "eng"
        "rus"
      ];
    };
in
{
  options.services.screen-ocr-sender = {
    enable =
      lib.mkEnableOption
        "screen OCR sender user service";

    user = lib.mkOption {
      type = lib.types.str;

      example = "mg";

      description = ''
        User whose systemd --user manager should run
        screen-ocr-sender.
      '';
    };

    configFile = lib.mkOption {
      type = lib.types.str;

      default =
        "%h/.config/screen-ocr/config.sender.toml";

      description = ''
        Path to sender configuration.
        Kept outside /nix/store because it may contain
        authentication tokens.
      '';
    };

    installScreenshotTool = lib.mkOption {
      type = lib.types.bool;

      default = true;

      description = ''
        Install screenshot-ocr globally.

        Disable this when screenshot-ocr is installed
        through Home Manager.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages =
      [
        senderPackage
      ]
      ++ lib.optional
        cfg.installScreenshotTool
        screenshotPackage;

    systemd.user.services.screen-ocr-sender = {
      description =
        "Screen OCR sender";

      wantedBy = [
        "graphical-session.target"
      ];

      after = [
        "graphical-session.target"
      ];

      partOf = [
        "graphical-session.target"
      ];

      unitConfig = {
        ConditionUser =
          cfg.user;
      };

      path = [
        tesseractWithLanguages
        pkgs.coreutils
      ];

      environment = {
        RUST_LOG = "info";
      };

      serviceConfig = {
        Type = "simple";

        ExecStart =
          "${senderPackage}/bin/screen-ocr-sender --config ${cfg.configFile}";

        Restart = "on-failure";

        RestartSec = "2s";
      };
    };
  };
}