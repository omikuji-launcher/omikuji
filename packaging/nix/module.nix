self:
{ lib, pkgs, config, ... }:

let
  inherit (lib)
  mkOption
  mkEnableOption
  mkPackageOption
  types
  mkIf
  optional
  literalExpression
  ;

  cfg = config.programs.omikuji;
  tomlFormat = pkgs.formats.toml { };
in
{
  options.programs.omikuji = {
    enable = mkEnableOption "omikuji";
    package = mkPackageOption self.packages.${pkgs.stdenv.hostPlatform.system} "omikuji" { nullable = true; };

    extraPackages = mkOption {
      type = with types; listOf package;
      default = [ ];
      example = "with pkgs; [mangohud winetricks gamescope gamemode umu-launcher]";
      description = ''
        List of packages to pass as extraPkgs to lutris.
        Please note runners are not detected properly this way, use a proper option for those.
      '';
    };

    steamPackage = mkOption {
      type = with types; nullOr package;
      default = null;
      example = "pkgs.steam or osConfig.programs.steam.package";
      description = ''
        This must be the same you use for your system, or two instances will conflict,
        for example, if you configure steam through the nixos module, a good value is "osConfig.programs.steam.package"
      '';
    };

    winePackages = mkOption {
      type = with types; listOf package;
      default = [ ];
      example = "[ pkgs.wineWow64Packages.full ]";
      description = ''
        List of wine packages to be added for omikuji to use.
      '';
    };

    protonPackages = mkOption {
      type = with types; listOf package;
      default = [ ];
      example = "[ pkgs.proton-ge-bin ]";
      description = ''
        List of proton packages to be added for omikuji to use with umu-launcher.
      '';
    };

    defaultWinePackage = mkOption {
      type = with types; nullOr package;
      default = null;
      example = "pkgs.proton-ge-bin";
      description = ''
        Default wine/proton package used in the settings.
      '';
    };

    settings = {
      defaults = mkOption {
        inherit (tomlFormat) type;
        default = { };
        example = literalExpression ''
          wine = {
            ntsync = true
            dxvk = true
            vkd3d = true
            d3d_extras = true
          };

          launch.env = {
            PROTON_USE_WAYLAND = "1";
          };

          graphics.mangohud = true;
          system.gamemode = true;
        '';
        description = ''
          Configuration written to
          {file}`$XDG_DATA_HOME/omikuji/defaults.toml`.
        '';
      };

      settings = mkOption {
        inherit (tomlFormat) type;
        default = { };
        description = ''
          Configuration written to
          {file}`$XDG_DATA_HOME/omikuji/settings.toml`.
        '';
      };

      ui = mkOption {
        inherit (tomlFormat) type;
        default = { };
        description = ''
          Configuration written to
          {file}`$XDG_DATA_HOME/omikuji/ui.toml`.
        '';
      };
    };
  };

  config = let
    formatWineName = (package: lib.toLower package.name);
  in
  mkIf cfg.enable
  {
    home.packages = mkIf (cfg.package != null) [
      (cfg.package.override {
        extraPkgs = (_prev: cfg.extraPackages ++ (optional (cfg.steamPackage != null) cfg.steamPackage));
      })
    ];

    xdg.dataFile =
    let
      buildWineLink =
        packages:
        map (
          # lutris seems to not detect wine/proton if the name has some caps
          package:
          (lib.nameValuePair "omikuji/runners/${formatWineName package}" {
            source = package;
          })
        ) packages;

      protonPackages = map (proton: proton.steamcompattool)
          (cfg.protonPackages ++ (lib.lists.optionals (cfg.defaultWinePackage != null) [ cfg.defaultWinePackage.steamcompattool ]));
    
      defaultSettingsMerged = lib.recursiveUpdate
        (lib.optionalAttrs (cfg.settings.defaults != { }) cfg.settings.defaults)
        (lib.optionalAttrs (cfg.defaultWinePackage != null) {
          wine.version = formatWineName cfg.defaultWinePackage;
        })
        ;
    in
    {
      "omikuji/defaults.toml" = mkIf (defaultSettingsMerged != { }) {
        source = (tomlFormat.generate "omikuji-config-defaults" defaultSettingsMerged);
      };

      "omikuji/settings.toml" = mkIf (cfg.settings.settings != { }) {
        source = (tomlFormat.generate "omikuji-config-defaults" cfg.settings.settings);
      };

      "omikuji/ui.toml" = mkIf (cfg.settings.ui != { }) {
        source = (tomlFormat.generate "omikuji-config-defaults" cfg.settings.ui);
      };
    }
    // lib.listToAttrs (
        buildWineLink cfg.winePackages
        ++ buildWineLink protonPackages
        );
  };
}

