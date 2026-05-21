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

      mutableDefaults = mkOption {
        type = types.bool;
        default = true;
        description = ''
          Wether configuration in `defaults.toml` can be updated by omikuji.
        '';
      };

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

      mutableSettings = mkOption {
        type = types.bool;
        default = true;
        description = ''
          Wether configuration in `settings.toml` can be updated by omikuji.
        '';
      };

      settings = mkOption {
        inherit (tomlFormat) type;
        default = { };
        example = literalExpression ''
          runners = [
            {
              name = "Proton-GE";
              kind = "proton";
              api_url = "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases";
              asset_pattern = ".tar.gz";
              extract = "tar_gz";
            }
            {
              name = "Proton-Cachyos";
              kind = "proton";
              api_url = "https://api.github.com/repos/CachyOS/proton-cachyos/releases";
              asset_pattern = ".tar.xz";
              extract = "tar_xz";
            }
          ];

          dll_packs = [
            {
              name = "DXVK";
              kind = "dxvk";
              api_url = "https://api.github.com/repos/doitsujin/dxvk/releases";
              asset_pattern = ".tar.gz";
              extract = "tar_gz";
            }
          ];
        '';
        description = ''
          Configuration written to
          {file}`$XDG_DATA_HOME/omikuji/settings.toml`.
        '';
      };

      mutableUi = mkOption {
        type = types.bool;
        default = true;
        description = ''
          Wether configuration in `ui.toml` can be updated by omikuji.
        '';
      };

      ui = mkOption {
        inherit (tomlFormat) type;
        default = { };
        example = literalExpression ''
          theme = {
            follow_system_colors = false;
            colors = {
              bg = "#181825";
              surface = "#1e1e2e";
              accent = "#cba6f7";
              accentText = "#11111b";
              text = "#cdd6f4";
              error = "#f38ba8";
              success = "#a6e3a1";
              warning = "#f9e2af";
            };
          };

          console_mode = {
            background = "wave";
            active = false;
          };
        '';
        description = ''
          Configuration written to
          {file}`$XDG_DATA_HOME/omikuji/ui.toml`.
        '';
      };
    };
  };

  config = let
    formatWineName = (package: lib.toLower package.name);

    defaultSettingsMerged = lib.recursiveUpdate
      (lib.optionalAttrs (cfg.settings.defaults != { }) cfg.settings.defaults)
      (lib.optionalAttrs (cfg.defaultWinePackage != null) {
        wine.version = formatWineName cfg.defaultWinePackage;
      })
      ;

    # Merges already existing config file (or empty toml if it doesn't exists)
    # with the config generated by options and saves the file.
    impureConfigMerger = initialConfig: generatedConfig: ''
      ${lib.getExe pkgs.nushell} -c "
      if ('${initialConfig}' | path exists) { open '${initialConfig}' } else { ' ' | from toml }
      | merge deep (open ${generatedConfig} | from toml)
      | save ${initialConfig} -f
      "
    '';

    impureConfigActivation = initialConfig: generatedConfig: (lib.hm.dag.entryAfter [ "linkGeneration" ] (
      impureConfigMerger initialConfig generatedConfig
    ));

    defaultsToml = tomlFormat.generate "omikuji-config-defaults" defaultSettingsMerged;
    settingsToml = tomlFormat.generate "omikuji-config-settings" cfg.settings.settings;
    uiToml = tomlFormat.generate "omikuji-config-ui" cfg.settings.ui;
  in
  mkIf cfg.enable
  {
    home = {
      packages = mkIf (cfg.package != null) [
        (cfg.package.override {
          extraPkgs = (_prev: cfg.extraPackages ++ (optional (cfg.steamPackage != null) cfg.steamPackage));
        })
      ];

      activation = {
        # Generating settings
        omikujiDefaultsSettings = mkIf (defaultSettingsMerged != { } && cfg.settings.mutableDefaults) (impureConfigActivation "${config.xdg.dataHome}/omikuji/defaults.toml" defaultsToml);
        omikujiSettingsSettings = mkIf (cfg.settings.settings != { } && cfg.settings.mutableSettings) (impureConfigActivation "${config.xdg.dataHome}/omikuji/settings.toml" settingsToml);
        omikujiUiSettings = mkIf (cfg.settings.ui != { } && cfg.settings.mutableUi) (impureConfigActivation "${config.xdg.dataHome}/omikuji/ui.toml" uiToml);
      };
    };

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
    in
    {
      "omikuji/defaults.toml" = mkIf (defaultSettingsMerged != { } && !cfg.settings.mutableDefaults) {
        source = defaultsToml;
      };

      "omikuji/settings.toml" = mkIf (cfg.settings.settings != { } && !cfg.settings.mutableSettings) {
        source = settingsToml;
      };

      "omikuji/ui.toml" = mkIf (cfg.settings.ui != { } && !cfg.settings.mutableUi) {
        source = uiToml;
      };
    }
    // lib.listToAttrs (
        buildWineLink cfg.winePackages
        ++ buildWineLink protonPackages
        );
  };
}
