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
    pub workspace_renderdoc_version: Option<String>,
    pub replay_version_match: Option<bool>,
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
        let workspace_renderdoc_version = workspace_renderdoc_version();
        let replay_version_match = compute_replay_version_match(
            renderdoccmd_version.as_deref(),
            workspace_renderdoc_version.as_deref(),
        );

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

        if let (Some(false), Some(installed), Some(workspace)) = (
            replay_version_match,
            renderdoccmd_version.as_deref(),
            workspace_renderdoc_version.as_deref(),
        ) {
            warnings.push(format!(
                "Installed RenderDoc version `{installed}` does not match workspace replay headers `{workspace}`; `renderdog-replay` requires an exact match and should be rebuilt after switching versions."
            ));
            suggested_commands.push(format!(
                "Install or select RenderDoc `{workspace}` when using `renderdog-replay`, or switch `third-party/renderdoc` to match the installed version and rebuild."
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
            workspace_renderdoc_version,
            replay_version_match,
            vulkan_layer,
            vulkan_layer_manifests,
            env,
            warnings,
            suggested_commands,
        })
    }
}

fn workspace_renderdoc_version() -> Option<String> {
    let version_header = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("third-party")
        .join("renderdoc")
        .join("renderdoc")
        .join("api")
        .join("replay")
        .join("version.h");
    let content = std::fs::read_to_string(version_header).ok()?;
    parse_workspace_renderdoc_version(&content)
}

fn parse_workspace_renderdoc_version(content: &str) -> Option<String> {
    let mut major: Option<String> = None;
    let mut minor: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("#define RENDERDOC_VERSION_MAJOR") {
            major = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("#define RENDERDOC_VERSION_MINOR") {
            minor = Some(value.trim().to_string());
        }
    }

    match (major, minor) {
        (Some(major), Some(minor)) => Some(format!("{major}.{minor}")),
        _ => None,
    }
}

fn compute_replay_version_match(installed: Option<&str>, workspace: Option<&str>) -> Option<bool> {
    let installed = installed.and_then(normalize_renderdoc_version)?;
    let workspace = workspace.and_then(normalize_renderdoc_version)?;
    Some(installed == workspace)
}

fn normalize_renderdoc_version(value: &str) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();

    for ch in value.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
            continue;
        }

        if !current.is_empty() {
            parts.push(std::mem::take(&mut current));
            if parts.len() == 2 {
                break;
            }
        }
    }

    if !current.is_empty() && parts.len() < 2 {
        parts.push(current);
    }

    if parts.len() >= 2 {
        Some(format!("{}.{}", parts[0], parts[1]))
    } else {
        None
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

#[cfg(test)]
mod tests {
    use super::{
        compute_replay_version_match, normalize_renderdoc_version,
        parse_workspace_renderdoc_version,
    };

    #[test]
    fn normalize_renderdoc_version_extracts_major_minor() {
        assert_eq!(
            normalize_renderdoc_version("1.43"),
            Some("1.43".to_string())
        );
        assert_eq!(
            normalize_renderdoc_version("RenderDoc v1.43 loaded"),
            Some("1.43".to_string())
        );
        assert_eq!(normalize_renderdoc_version("unknown"), None);
    }

    #[test]
    fn compute_replay_version_match_compares_normalized_versions() {
        assert_eq!(
            compute_replay_version_match(Some("v1.43"), Some("1.43")),
            Some(true)
        );
        assert_eq!(
            compute_replay_version_match(Some("1.42"), Some("1.43")),
            Some(false)
        );
        assert_eq!(
            compute_replay_version_match(Some("dev"), Some("1.43")),
            None
        );
    }

    #[test]
    fn parse_workspace_renderdoc_version_reads_version_header_macros() {
        let content = r#"
#define RENDERDOC_VERSION_MAJOR 1
#define RENDERDOC_VERSION_MINOR 43
"#;

        assert_eq!(
            parse_workspace_renderdoc_version(content),
            Some("1.43".to_string())
        );
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
