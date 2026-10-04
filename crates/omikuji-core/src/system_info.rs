use crate::components;
use ash::vk;
use std::ffi::c_char;
use std::fmt::Write as _;

const VULKAN_DATA_DIRS: &[&str] = &[
    "/usr/local/etc",
    "/usr/local/share",
    "/etc",
    "/usr/share",
    "/usr/lib/x86_64-linux-gnu/GL",
    "/usr/lib/i386-linux-gnu/GL",
    "/opt/amdgpu-pro/etc",
];

struct Gpu {
    name: String,
    driver: String,
    vendor_id: u32,
    device_id: u32,
    nvidia: bool,
    uuid: String,
    loader: String,
}

impl Gpu {
    fn display(&self) -> String {
        if self.driver.is_empty() {
            self.name.clone()
        } else {
            format!("{} ({})", self.name, self.driver)
        }
    }

    fn pci_pair(&self) -> String {
        format!("{:04x}:{:04x}", self.vendor_id, self.device_id)
    }
}

pub fn report(app_version: &str, qt_version: &str) -> String {
    let mut fields: Vec<(String, String)> = vec![
        ("Omikuji".into(), app_version.to_string()),
        ("Distro".into(), distro()),
        ("Kernel".into(), kernel()),
        ("WM/DE".into(), desktop()),
        ("CPU".into(), cpu()),
        ("RAM".into(), ram()),
    ];

    let gpus = enumerate();
    match gpus.len() {
        0 => fields.push(("GPU".into(), "unknown".into())),
        1 => fields.push(("GPU".into(), gpus[0].display())),
        _ => {
            for (i, g) in gpus.iter().enumerate() {
                fields.push((format!("GPU {}", i + 1), g.display()));
            }
        }
    }

    fields.push(("Qt".into(), qt_version.to_string()));
    fields.push((
        "Flatpak".into(),
        if flatpak() { "yes" } else { "no" }.into(),
    ));
    for spec in components::specs::all() {
        fields.push((
            spec.name.into(),
            components::status_for(spec).version_label(),
        ));
    }

    let pad = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0) + 1 + 5;
    let mut out = String::new();
    for (k, v) in &fields {
        let _ = writeln!(out, "{:<pad$}{}", format!("{k}:"), v);
    }
    out.trim_end().to_string()
}

pub fn gpu_select_list() -> Vec<(String, String)> {
    enumerate().into_iter().map(|g| (g.name, g.uuid)).collect()
}

pub fn gpu_launch_env(uuid: &str) -> Vec<(String, String)> {
    if uuid.is_empty() {
        return Vec::new();
    }
    let Some(gpu) = enumerate().into_iter().find(|g| g.uuid == uuid) else {
        return Vec::new();
    };

    let mut env = Vec::new();
    if gpu.nvidia {
        env.push(("DRI_PRIME".into(), "1".into()));
        env.push(("__NV_PRIME_RENDER_OFFLOAD".into(), "1".into()));
        env.push(("__GLX_VENDOR_LIBRARY_NAME".into(), "nvidia".into()));
        env.push(("__VK_LAYER_NV_optimus".into(), "NVIDIA_only".into()));
    } else {
        env.push(("DRI_PRIME".into(), gpu.pci_pair()));
    }

    let icd = icd_files(&gpu.loader);
    if !icd.is_empty() {
        env.push(("VK_DRIVER_FILES".into(), icd.clone()));
        env.push(("VK_ICD_FILENAMES".into(), icd));
    }

    env.push(("DXVK_FILTER_DEVICE_UUID".into(), gpu.uuid.replace('-', "")));
    env
}

fn os_release(key: &str) -> Option<String> {
    let content = fs_err::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        if let Some(val) = line.strip_prefix(key).and_then(|r| r.strip_prefix('=')) {
            return Some(val.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn distro() -> String {
    os_release("PRETTY_NAME")
        .or_else(|| os_release("NAME"))
        .unwrap_or_else(|| "Linux".into())
}

fn kernel() -> String {
    fs_err::read_to_string("/proc/sys/kernel/osrelease")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".into())
}

fn desktop() -> String {
    let de = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("XDG_SESSION_DESKTOP"))
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_default();
    let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    match (de.is_empty(), session.is_empty()) {
        (false, false) => format!("{de} ({session})"),
        (false, true) => de,
        (true, false) => session,
        (true, true) => "unknown".into(),
    }
}

fn cpu() -> String {
    if let Ok(info) = fs_err::read_to_string("/proc/cpuinfo") {
        for line in info.lines() {
            if let Some(rest) = line.strip_prefix("model name")
                && let Some(idx) = rest.find(':')
            {
                return rest[idx + 1..].trim().to_string();
            }
        }
    }
    "unknown".into()
}

fn ram() -> String {
    if let Ok(info) = fs_err::read_to_string("/proc/meminfo") {
        for line in info.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:")
                && let Some(kb) = rest
                    .split_whitespace()
                    .next()
                    .and_then(|n| n.parse::<f64>().ok())
            {
                return format!("{:.1} GiB", kb / 1024.0 / 1024.0);
            }
        }
    }
    "unknown".into()
}

fn flatpak() -> bool {
    std::env::var("FLATPAK_ID").is_ok() || std::path::Path::new("/.flatpak-info").exists()
}

fn enumerate() -> Vec<Gpu> {
    let entry = match unsafe { ash::Entry::load() } {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let app_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_1);
    let create_info = vk::InstanceCreateInfo::default().application_info(&app_info);
    let instance = match unsafe { entry.create_instance(&create_info, None) } {
        Ok(i) => i,
        Err(_) => return Vec::new(),
    };

    let mut out = Vec::new();
    if let Ok(devices) = unsafe { instance.enumerate_physical_devices() } {
        for pd in devices {
            let mut driver = vk::PhysicalDeviceDriverProperties::default();
            let mut ids = vk::PhysicalDeviceIDProperties::default();
            let mut props2 = vk::PhysicalDeviceProperties2::default()
                .push_next(&mut driver)
                .push_next(&mut ids);
            unsafe { instance.get_physical_device_properties2(pd, &mut props2) };

            let name = pretty_name(&c_array_to_string(&props2.properties.device_name));
            let vendor_id = props2.properties.vendor_id;
            let device_id = props2.properties.device_id;

            let driver_str = driver_summary(
                &c_array_to_string(&driver.driver_name),
                &c_array_to_string(&driver.driver_info),
            );
            let nvidia = driver.driver_id == vk::DriverId::NVIDIA_PROPRIETARY;
            let uuid = format_uuid(&ids.device_uuid);
            let loader = loader_for(driver.driver_id, vendor_id).to_string();

            out.push(Gpu {
                name,
                driver: driver_str,
                vendor_id,
                device_id,
                nvidia,
                uuid,
                loader,
            });
        }
    }

    unsafe { instance.destroy_instance(None) };
    out
}

fn loader_for(driver_id: vk::DriverId, vendor_id: u32) -> &'static str {
    use vk::DriverId;
    if driver_id == DriverId::MESA_RADV {
        return "radeon";
    }
    if driver_id == DriverId::INTEL_OPEN_SOURCE_MESA {
        return "intel";
    }
    if driver_id == DriverId::NVIDIA_PROPRIETARY {
        return "nvidia";
    }
    if driver_id == DriverId::MESA_LLVMPIPE {
        return "lvp";
    }
    if driver_id == DriverId::AMD_OPEN_SOURCE || driver_id == DriverId::AMD_PROPRIETARY {
        return "amd";
    }
    match vendor_id {
        0x1002 => "radeon",
        0x8086 => "intel",
        0x10de => "nvidia",
        _ => "",
    }
}

fn icd_files(loader: &str) -> String {
    if loader.is_empty() {
        return String::new();
    }
    let mut files = Vec::new();
    for dir in VULKAN_DATA_DIRS {
        let icd_dir = std::path::Path::new(dir).join("vulkan").join("icd.d");
        let Ok(entries) = fs_err::read_dir(&icd_dir) else {
            continue;
        };
        let mut found: Vec<String> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.extension().and_then(|x| x.to_str()) == Some("json")
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.contains(loader))
            })
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        found.sort();
        files.append(&mut found);
    }
    files.join(":")
}

fn driver_summary(name: &str, info: &str) -> String {
    match (name.is_empty(), info.is_empty()) {
        (false, false) => format!("{name} {info}"),
        (false, true) => name.to_string(),
        (true, false) => info.to_string(),
        (true, true) => String::new(),
    }
}

fn format_uuid(b: &[u8; 16]) -> String {
    let mut s = String::with_capacity(36);
    for (i, byte) in b.iter().enumerate() {
        if matches!(i, 4 | 6 | 8 | 10) {
            s.push('-');
        }
        let _ = write!(s, "{byte:02x}");
    }
    s
}

fn c_array_to_string(arr: &[c_char]) -> String {
    let bytes: Vec<u8> = arr
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn pretty_name(s: &str) -> String {
    let s = s.trim();
    if s.ends_with(')')
        && let Some(idx) = s.rfind(" (")
    {
        return s[..idx].trim().to_string();
    }
    s.to_string()
}
