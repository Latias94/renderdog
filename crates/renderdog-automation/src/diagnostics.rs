use std::{collections::BTreeSet, process::Command, string::String};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::RenderDocInstallation;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VulkanLayerDiagnosis {
    pub supported: bool,
    pub needs_attention: bool,
    pub unfixable: bool,
    pub need_elevation: bool,
    pub this_install_registered: Option<bool>,
    pub other_installs_registered: Option<bool>,
    pub conflicting_manifests: Vec<String>,
    pub summary: String,
    pub stdout: String,
    pub stderr: String,
    pub suggested_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnvironmentVarInfo {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnvironmentDiagnosis {
    pub root_dir: String,
    pub qrenderdoc_exe: String,
    pub renderdoccmd_exe: String,
    pub platform: String,
    pub arch: String,
    pub is_elevated: Option<bool>,
    pub renderdoccmd_version: Option<String>,
    pub vulkan_layer: Option<VulkanLayerDiagnosis>,
    pub vulkan_layer_manifests: Vec<String>,
    pub env: Vec<EnvironmentVarInfo>,
    pub warnings: Vec<String>,
    pub suggested_commands: Vec<String>,
}

#[derive(Debug, Error)]
pub enum VulkanLayerDiagnosisError {
    #[error("failed to run renderdoccmd: {0}")]
    Spawn(std::io::Error),
    #[error("renderdoccmd output was not valid UTF-8")]
    InvalidUtf8,
}

impl RenderDocInstallation {
    pub fn diagnose_vulkan_layer(&self) -> Result<VulkanLayerDiagnosis, VulkanLayerDiagnosisError> {
        let output = Command::new(&self.renderdoccmd_exe)
            .arg("vulkanlayer")
            .arg("--explain")
            .output()
            .map_err(VulkanLayerDiagnosisError::Spawn)?;

        let stdout =
            String::from_utf8(output.stdout).map_err(|_| VulkanLayerDiagnosisError::InvalidUtf8)?;
        let stderr =
            String::from_utf8(output.stderr).map_err(|_| VulkanLayerDiagnosisError::InvalidUtf8)?;

        let combined = format!("{stderr}\n{stdout}");
        let lower = combined.to_ascii_lowercase();

        let supported = !lower.contains("is not a valid command")
            && !lower.contains("not a valid command")
            && !lower.contains("unknown command")
            && !lower.contains("unrecognized command");

        if !supported {
            return Ok(VulkanLayerDiagnosis {
                supported: false,
                needs_attention: false,
                unfixable: false,
                need_elevation: false,
                this_install_registered: None,
                other_installs_registered: None,
                conflicting_manifests: Vec::new(),
                summary: "renderdoccmd does not support the `vulkanlayer` command (too old?)"
                    .to_string(),
                stdout,
                stderr,
                suggested_commands: Vec::new(),
            });
        }

        let ok = lower.contains("appears to be correctly registered");
        let unfixable = lower.contains("unfixable problem");
        let needs_attention = !ok && (lower.contains("vulkan layer") || unfixable);
        let need_elevation = lower.contains("administrator")
            || lower.contains("admin privileges")
            || lower.contains("elevation");

        let this_install_registered =
            if lower.contains("this build's renderdoc layer is not registered") {
                Some(false)
            } else if ok {
                Some(true)
            } else {
                None
            };

        let other_installs_registered = if lower.contains("non-matching renderdoc layer")
            || lower.contains("other installs registered")
        {
            Some(true)
        } else {
            None
        };

        let conflicting_manifests = extract_manifest_paths(&combined);

        let mut suggested_commands: Vec<String> = Vec::new();
        if needs_attention {
            suggested_commands.push(format!(
                "\"{}\" vulkanlayer --register --user",
                self.renderdoccmd_exe.display()
            ));
            suggested_commands.push(format!(
                "\"{}\" vulkanlayer --register --system",
                self.renderdoccmd_exe.display()
            ));
        }

        let summary = if ok {
            "RenderDoc Vulkan layer is correctly registered".to_string()
        } else if unfixable {
            "RenderDoc Vulkan layer configuration has an unfixable problem".to_string()
        } else if needs_attention {
            "RenderDoc Vulkan layer is not correctly registered".to_string()
        } else {
            "RenderDoc Vulkan layer status is unknown".to_string()
        };

        Ok(VulkanLayerDiagnosis {
            supported: true,
            needs_attention,
            unfixable,
            need_elevation,
            this_install_registered,
            other_installs_registered,
            conflicting_manifests,
            summary,
            stdout,
            stderr,
            suggested_commands,
        })
    }

    pub fn diagnose_environment(&self) -> Result<EnvironmentDiagnosis, VulkanLayerDiagnosisError> {
        let renderdoccmd_version = self.version().ok().map(|s| s.trim().to_string());

        let vulkan_layer = self.diagnose_vulkan_layer().ok();
        let vulkan_layer_manifests = find_vulkan_layer_manifests(&self.root_dir);
        let is_elevated = is_process_elevated();

        let platform = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();

        let env = [
            "VK_INSTANCE_LAYERS",
            "VK_LAYER_PATH",
            "VK_LOADER_DEBUG",
            "ENABLE_VULKAN_RENDERDOC_CAPTURE",
            "RENDERDOC_HOOK_EGL",
            "RENDERDOC_DEBUG_LOG_FILE",
        ]
        .into_iter()
        .map(|name| EnvironmentVarInfo {
            name: name.to_string(),
            value: std::env::var(name).ok(),
        })
        .collect::<Vec<_>>();

        let mut warnings: Vec<String> = Vec::new();
        let mut suggested_commands: Vec<String> = Vec::new();

        if platform == "macos" {
            warnings.push(
                "RenderDoc on macOS is experimental and not officially supported for debugging; capture/replay may be unreliable."
                    .to_string(),
            );
        }

        if platform == "linux" && arch != "x86_64" {
            warnings.push(format!(
                "RenderDoc officially supports only x86_64 Linux; current arch is `{arch}` (ARM/32-bit targets are not supported)."
            ));
        }

        if platform == "windows" && arch != "x86_64" {
            warnings.push(format!(
                "RenderDoc Windows support is primarily x86_64; current arch is `{arch}` and may not work."
            ));
        }

        if let Some(vk) = &vulkan_layer {
            if !vk.supported || vk.unfixable {
                warnings.push(vk.summary.clone());
            } else if vk.needs_attention {
                warnings.push(vk.summary.clone());
                suggested_commands.extend(vk.suggested_commands.iter().cloned());
                suggested_commands.push(format!(
                    "\"{}\" capture <your_exe> [args...] (fallback: injection-based capture)",
                    self.renderdoccmd_exe.display()
                ));
            }
        }

        if let (Some(vk), Some(false)) = (&vulkan_layer, is_elevated)
            && vk.need_elevation
            && vk.needs_attention
        {
            warnings.push("Vulkan layer registration may require administrator privileges. Re-run the registration command as administrator.".to_string());
        }

        let vk_instance_layers = env
            .iter()
            .find(|e| e.name == "VK_INSTANCE_LAYERS")
            .and_then(|e| e.value.as_deref())
            .unwrap_or("");
        if !vk_instance_layers.is_empty()
            && !vk_instance_layers.contains("VK_LAYER_RENDERDOC_Capture")
        {
            warnings.push("VK_INSTANCE_LAYERS is set but does not include VK_LAYER_RENDERDOC_Capture; this can prevent RenderDoc's layer from being enabled.".to_string());
            suggested_commands.push("Set VK_INSTANCE_LAYERS to include VK_LAYER_RENDERDOC_Capture, or clear it if it is forcing a different layer set.".to_string());
        }

        let vk_layer_path = env
            .iter()
            .find(|e| e.name == "VK_LAYER_PATH")
            .and_then(|e| e.value.as_deref())
            .unwrap_or("");
        if !vk_layer_path.is_empty()
            && !vk_layer_path.contains(&self.root_dir.display().to_string())
        {
            warnings.push("VK_LAYER_PATH is set; if Vulkan capture fails, ensure it includes the RenderDoc layer JSON location or unregister conflicting installs.".to_string());
        }

        if !vulkan_layer_manifests.is_empty() {
            let mut dirs: Vec<String> = vulkan_layer_manifests
                .iter()
                .filter_map(|p| {
                    std::path::Path::new(p)
                        .parent()
                        .map(|d| d.display().to_string())
                })
                .collect();
            dirs.sort();
            dirs.dedup();

            if !dirs.is_empty() && !vk_layer_path.is_empty() {
                let any_in_path = dirs.iter().any(|d| vk_layer_path.contains(d));
                if !any_in_path {
                    warnings.push(format!(
                        "VK_LAYER_PATH is set but does not appear to include the RenderDoc Vulkan layer manifest directory. Detected manifest dirs: {}",
                        dirs.join(" | ")
                    ));
                    let sep = if cfg!(windows) { ";" } else { ":" };
                    suggested_commands.push(format!(
                        "Update VK_LAYER_PATH to include the detected directories (separator `{sep}`), or unset VK_LAYER_PATH if it is causing conflicts."
                    ));
                }
            }
        }

        Ok(EnvironmentDiagnosis {
            root_dir: self.root_dir.display().to_string(),
            qrenderdoc_exe: self.qrenderdoc_exe.display().to_string(),
            renderdoccmd_exe: self.renderdoccmd_exe.display().to_string(),
            platform,
            arch,
            is_elevated,
            renderdoccmd_version,
            vulkan_layer,
            vulkan_layer_manifests,
            env,
            warnings,
            suggested_commands,
        })
    }
}

fn extract_manifest_paths(text: &str) -> Vec<String> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();
        if lower.ends_with(".json") || lower.contains(".json ") || lower.contains(".json\t") {
            set.insert(trimmed.to_string());
        }
    }
    set.into_iter().collect()
}

fn find_vulkan_layer_manifests(root_dir: &std::path::Path) -> Vec<String> {
    let mut hits: Vec<String> = Vec::new();

    let mut stack: Vec<(std::path::PathBuf, usize)> = vec![(root_dir.to_path_buf(), 0)];
    while let Some((dir, depth)) = stack.pop() {
        if depth > 6 {
            continue;
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(v) => v,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push((path, depth + 1));
                continue;
            }
            if path.extension().and_then(|s| s.to_str()).unwrap_or("") != "json" {
                continue;
            }
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            if content.contains("VK_LAYER_RENDERDOC_Capture") {
                hits.push(path.display().to_string());
            }
        }
    }

    hits.sort();
    hits.dedup();
    hits
}

fn is_process_elevated() -> Option<bool> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Security::{
            GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
        };
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        unsafe {
            let mut token = 0isize;
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return None;
            }
            let _guard = HandleGuard(token);

            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut returned = 0u32;
            let ok = GetTokenInformation(
                token,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut returned,
            );
            if ok == 0 {
                return None;
            }
            Some(elevation.TokenIsElevated != 0)
        }
    }

    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
struct HandleGuard(isize);

#[cfg(windows)]
impl Drop for HandleGuard {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}
