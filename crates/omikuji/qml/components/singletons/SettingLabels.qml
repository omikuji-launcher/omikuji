pragma Singleton

import QtQuick

QtObject {
    function label(key) {
        switch (key) {
        case "wine.version":                       return qsTr("Version")
        case "wine.prefix":                        return qsTr("Prefix")
        case "wine.prefix_arch":                   return qsTr("Architecture")
        case "wine.esync":                         return qsTr("Esync")
        case "wine.fsync":                         return qsTr("Fsync")
        case "wine.ntsync":                        return qsTr("NTSync")
        case "wine.dxvk":                          return "DXVK"
        case "wine.dxvk_version":                  return qsTr("DXVK version")
        case "wine.vkd3d":                         return "VKD3D"
        case "wine.vkd3d_version":                 return qsTr("VKD3D version")
        case "wine.dxvk_nvapi":                    return "DXVK-NVAPI"
        case "wine.dxvk_nvapi_version":            return qsTr("DXVK-NVAPI version")
        case "wine.battleye":                      return "BattlEye"
        case "wine.easyanticheat":                 return "EasyAntiCheat"
        case "wine.fsr":                           return "FSR"
        case "wine.dpi_scaling":                   return qsTr("DPI Scaling")
        case "wine.dpi":                           return qsTr("DPI")
        case "wine.audio_driver":                  return qsTr("Audio Driver")
        case "wine.graphics_driver":               return qsTr("Graphics Driver")
        case "wine.dll_overrides":                 return qsTr("DLL Overrides")
        case "launch.command_prefix":              return qsTr("Command Prefix")
        case "launch.env":                         return qsTr("Environment Variables")
        case "graphics.mangohud":                  return "MangoHUD"
        case "graphics.gpu":                       return qsTr("GPU")
        case "graphics.gamescope.enabled":         return qsTr("Enable")
        case "graphics.gamescope.width":           return qsTr("Width")
        case "graphics.gamescope.height":          return qsTr("Height")
        case "graphics.gamescope.game_width":      return qsTr("Game Width")
        case "graphics.gamescope.game_height":     return qsTr("Game Height")
        case "graphics.gamescope.fullscreen":      return qsTr("Fullscreen")
        case "graphics.gamescope.borderless":      return qsTr("Borderless")
        case "graphics.gamescope.integer_scaling": return qsTr("Integer Scaling")
        case "graphics.gamescope.hdr":             return qsTr("HDR")
        case "graphics.gamescope.fps":             return qsTr("FPS Limit")
        case "graphics.gamescope.refresh_rate":    return qsTr("Refresh Rate")
        case "graphics.gamescope.filter":          return qsTr("Filter")
        case "graphics.gamescope.fsr_sharpness":   return qsTr("FSR Sharpness")
        case "system.gamemode":                    return "GameMode"
        case "system.cpu_limit":                   return qsTr("CPU Cores")
        case "system.pulse_latency":               return qsTr("Reduce Pulse Latency")
        case "system.prevent_sleep":               return qsTr("Prevent Sleep")
        case "system.discord_rpc":                 return qsTr("Discord Rich Presence")
        }
        return key
    }

    function groupTitle(group) {
        switch (group) {
        case "wine":               return "Wine"
        case "sync":               return qsTr("Sync")
        case "translation_layers": return qsTr("Translation Layers")
        case "compatibility":      return qsTr("Compatibility")
        case "display":            return qsTr("Display")
        case "drivers":            return qsTr("Drivers")
        case "dll_overrides":      return qsTr("DLL Overrides")
        case "launch":             return qsTr("Launch")
        case "environment":        return qsTr("Environment")
        case "graphics":           return qsTr("Graphics")
        case "gamescope":          return "Gamescope"
        case "performance":        return qsTr("Performance")
        case "audio":              return qsTr("Audio")
        case "power":              return qsTr("Power")
        case "discord":            return qsTr("Discord")
        }
        return group
    }
}
