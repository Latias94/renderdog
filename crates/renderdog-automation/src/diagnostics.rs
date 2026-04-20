use std::{collections::BTreeSet, path::Path, process::Command, string::String};

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

struct EnvironmentAssessmentInputs<'a> {
    root_dir: &'a Path,
    renderdoccmd_exe: &'a Path,
    platform: &'a str,
    arch: &'a str,
    is_elevated: Option<bool>,
    renderdoccmd_version: Option<&'a str>,
    workspace_renderdoc_version: Option<&'a str>,
    replay_version_match: Option<bool>,
    vulkan_layer: Option<&'a VulkanLayerDiagnosis>,
    vulkan_layer_manifests: &'a [String],
    env: &'a [EnvironmentVarInfo],
}

struct EnvironmentFeedback {
    warnings: Vec<String>,
    suggested_commands: Vec<String>,
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

        Ok(parse_vulkan_layer_diagnosis(
            &self.renderdoccmd_exe,
            stdout,
            stderr,
        ))
    }

    pub fn diagnose_environment(&self) -> Result<EnvironmentDiagnosis, VulkanLayerDiagnosisError> {
        let renderdoccmd_version = self.version().ok().map(|s| s.trim().to_string());
        let workspace_renderdoc_version =
            renderdog_sys::workspace_renderdoc_replay_version().map(str::to_owned);
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

        let feedback = collect_environment_feedback(&EnvironmentAssessmentInputs {
            root_dir: &self.root_dir,
            renderdoccmd_exe: &self.renderdoccmd_exe,
            platform: &platform,
            arch: &arch,
            is_elevated,
            renderdoccmd_version: renderdoccmd_version.as_deref(),
            workspace_renderdoc_version: workspace_renderdoc_version.as_deref(),
            replay_version_match,
            vulkan_layer: vulkan_layer.as_ref(),
            vulkan_layer_manifests: &vulkan_layer_manifests,
            env: &env,
        });

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
            warnings: feedback.warnings,
            suggested_commands: feedback.suggested_commands,
        })
    }
}

fn parse_vulkan_layer_diagnosis(
    renderdoccmd_exe: &Path,
    stdout: String,
    stderr: String,
) -> VulkanLayerDiagnosis {
    let combined = format!("{stderr}\n{stdout}");
    let lower = combined.to_ascii_lowercase();

    let supported = !lower.contains("is not a valid command")
        && !lower.contains("not a valid command")
        && !lower.contains("unknown command")
        && !lower.contains("unrecognized command");

    if !supported {
        return VulkanLayerDiagnosis {
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
        };
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
            renderdoccmd_exe.display()
        ));
        suggested_commands.push(format!(
            "\"{}\" vulkanlayer --register --system",
            renderdoccmd_exe.display()
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

    VulkanLayerDiagnosis {
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
    }
}

fn compute_replay_version_match(
    renderdoccmd_version: Option<&str>,
    workspace_renderdoc_version: Option<&str>,
) -> Option<bool> {
    match (renderdoccmd_version, workspace_renderdoc_version) {
        (Some(installed), Some(workspace)) => Some(renderdog_sys::renderdoc_versions_match(
            installed, workspace,
        )),
        _ => None,
    }
}

fn collect_environment_feedback(inputs: &EnvironmentAssessmentInputs<'_>) -> EnvironmentFeedback {
    let mut warnings: Vec<String> = Vec::new();
    let mut suggested_commands: Vec<String> = Vec::new();

    if inputs.platform == "macos" {
        warnings.push(
            "RenderDoc on macOS is experimental and not officially supported for debugging; capture/replay may be unreliable."
                .to_string(),
        );
    }

    if inputs.platform == "linux" && inputs.arch != "x86_64" {
        warnings.push(format!(
            "RenderDoc officially supports only x86_64 Linux; current arch is `{}` (ARM/32-bit targets are not supported).",
            inputs.arch
        ));
    }

    if inputs.platform == "windows" && inputs.arch != "x86_64" {
        warnings.push(format!(
            "RenderDoc Windows support is primarily x86_64; current arch is `{}` and may not work.",
            inputs.arch
        ));
    }

    if let (Some(false), Some(installed), Some(workspace)) = (
        inputs.replay_version_match,
        inputs.renderdoccmd_version,
        inputs.workspace_renderdoc_version,
    ) {
        warnings.push(format!(
            "Installed RenderDoc version `{installed}` does not match workspace replay headers `{workspace}`; `renderdog-replay` requires an exact match and should be rebuilt after switching versions."
        ));
        suggested_commands.push(format!(
            "Install or select RenderDoc `{workspace}` when using `renderdog-replay`, or switch `third-party/renderdoc` to match the installed version and rebuild."
        ));
    }

    if let Some(vk) = inputs.vulkan_layer {
        if !vk.supported || vk.unfixable {
            warnings.push(vk.summary.clone());
        } else if vk.needs_attention {
            warnings.push(vk.summary.clone());
            suggested_commands.extend(vk.suggested_commands.iter().cloned());
            suggested_commands.push(format!(
                "\"{}\" capture <your_exe> [args...] (fallback: injection-based capture)",
                inputs.renderdoccmd_exe.display()
            ));
        }
    }

    if let (Some(vk), Some(false)) = (inputs.vulkan_layer, inputs.is_elevated)
        && vk.need_elevation
        && vk.needs_attention
    {
        warnings.push("Vulkan layer registration may require administrator privileges. Re-run the registration command as administrator.".to_string());
    }

    let vk_instance_layers = env_var_value(inputs.env, "VK_INSTANCE_LAYERS").unwrap_or("");
    if !vk_instance_layers.is_empty() && !vk_instance_layers.contains("VK_LAYER_RENDERDOC_Capture")
    {
        warnings.push("VK_INSTANCE_LAYERS is set but does not include VK_LAYER_RENDERDOC_Capture; this can prevent RenderDoc's layer from being enabled.".to_string());
        suggested_commands.push("Set VK_INSTANCE_LAYERS to include VK_LAYER_RENDERDOC_Capture, or clear it if it is forcing a different layer set.".to_string());
    }

    let vk_layer_path = env_var_value(inputs.env, "VK_LAYER_PATH").unwrap_or("");
    if !vk_layer_path.is_empty() && !vk_layer_path.contains(&inputs.root_dir.display().to_string())
    {
        warnings.push("VK_LAYER_PATH is set; if Vulkan capture fails, ensure it includes the RenderDoc layer JSON location or unregister conflicting installs.".to_string());
    }

    let mut dirs: Vec<String> = inputs
        .vulkan_layer_manifests
        .iter()
        .filter_map(|p| Path::new(p).parent().map(|d| d.display().to_string()))
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

    EnvironmentFeedback {
        warnings,
        suggested_commands,
    }
}

fn env_var_value<'a>(env: &'a [EnvironmentVarInfo], name: &str) -> Option<&'a str> {
    env.iter()
        .find(|entry| entry.name == name)
        .and_then(|entry| entry.value.as_deref())
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

#[cfg(test)]
mod tests {
    use super::{
        EnvironmentAssessmentInputs, EnvironmentVarInfo, VulkanLayerDiagnosis,
        collect_environment_feedback, compute_replay_version_match, parse_vulkan_layer_diagnosis,
    };
    use std::path::Path;

    #[test]
    fn parse_vulkan_layer_diagnosis_detects_unsupported_command() {
        let diag = parse_vulkan_layer_diagnosis(
            Path::new("/renderdoc/renderdoccmd"),
            String::new(),
            "renderdoccmd: 'vulkanlayer' is not a valid command".to_string(),
        );

        assert!(!diag.supported);
        assert!(!diag.needs_attention);
        assert_eq!(
            diag.summary,
            "renderdoccmd does not support the `vulkanlayer` command (too old?)"
        );
        assert!(diag.suggested_commands.is_empty());
    }

    #[test]
    fn parse_vulkan_layer_diagnosis_extracts_attention_state() {
        let stdout = "\
this build's renderdoc layer is not registered
other installs registered
/opt/renderdoc/renderdoc.json
/opt/renderdoc/renderdoc.json
administrator privileges required"
            .to_string();
        let diag = parse_vulkan_layer_diagnosis(
            Path::new("/renderdoc/renderdoccmd"),
            stdout,
            "Vulkan layer registration needs attention".to_string(),
        );

        assert!(diag.supported);
        assert!(diag.needs_attention);
        assert!(!diag.unfixable);
        assert!(diag.need_elevation);
        assert_eq!(diag.this_install_registered, Some(false));
        assert_eq!(diag.other_installs_registered, Some(true));
        assert_eq!(
            diag.conflicting_manifests,
            vec!["/opt/renderdoc/renderdoc.json".to_string()]
        );
        assert_eq!(
            diag.summary,
            "RenderDoc Vulkan layer is not correctly registered"
        );
        assert_eq!(diag.suggested_commands.len(), 2);
        assert!(
            diag.suggested_commands[0].ends_with("vulkanlayer --register --user"),
            "unexpected command: {}",
            diag.suggested_commands[0]
        );
        assert!(
            diag.suggested_commands[1].ends_with("vulkanlayer --register --system"),
            "unexpected command: {}",
            diag.suggested_commands[1]
        );
    }

    #[test]
    fn compute_replay_version_match_uses_workspace_policy() {
        assert_eq!(
            compute_replay_version_match(Some("v1.44"), Some("1.44")),
            Some(true)
        );
        assert_eq!(
            compute_replay_version_match(Some("1.43"), Some("1.44")),
            Some(false)
        );
        assert_eq!(compute_replay_version_match(Some("1.44"), None), None);
    }

    #[test]
    fn collect_environment_feedback_reports_version_and_layer_overrides() {
        let env = vec![
            EnvironmentVarInfo {
                name: "VK_INSTANCE_LAYERS".to_string(),
                value: Some("VK_LAYER_OTHER".to_string()),
            },
            EnvironmentVarInfo {
                name: "VK_LAYER_PATH".to_string(),
                value: Some("/custom/layers".to_string()),
            },
        ];
        let feedback = collect_environment_feedback(&EnvironmentAssessmentInputs {
            root_dir: Path::new("/renderdoc"),
            renderdoccmd_exe: Path::new("/renderdoc/renderdoccmd"),
            platform: "linux",
            arch: "x86_64",
            is_elevated: None,
            renderdoccmd_version: Some("1.43"),
            workspace_renderdoc_version: Some("1.44"),
            replay_version_match: Some(false),
            vulkan_layer: None,
            vulkan_layer_manifests: &["/opt/renderdoc/renderdoc_capture.json".to_string()],
            env: &env,
        });

        assert!(
            feedback
                .warnings
                .iter()
                .any(|warning| warning.contains("does not match workspace replay headers `1.44`"))
        );
        assert!(feedback.warnings.iter().any(|warning| warning.contains(
            "VK_INSTANCE_LAYERS is set but does not include VK_LAYER_RENDERDOC_Capture"
        )));
        assert!(
            feedback
                .warnings
                .iter()
                .any(|warning| warning.contains("VK_LAYER_PATH is set; if Vulkan capture fails"))
        );
        assert!(feedback.warnings.iter().any(|warning| {
            warning.contains(
                "does not appear to include the RenderDoc Vulkan layer manifest directory",
            )
        }));
        assert!(
            feedback
                .suggested_commands
                .iter()
                .any(|cmd| cmd.contains("Install or select RenderDoc `1.44`"))
        );
        assert!(feedback.suggested_commands.iter().any(|cmd| {
            cmd.contains("Set VK_INSTANCE_LAYERS to include VK_LAYER_RENDERDOC_Capture")
        }));
    }

    #[test]
    fn collect_environment_feedback_reports_vk_registration_actions() {
        let vk = VulkanLayerDiagnosis {
            supported: true,
            needs_attention: true,
            unfixable: false,
            need_elevation: true,
            this_install_registered: Some(false),
            other_installs_registered: Some(true),
            conflicting_manifests: Vec::new(),
            summary: "RenderDoc Vulkan layer is not correctly registered".to_string(),
            stdout: String::new(),
            stderr: String::new(),
            suggested_commands: vec![
                "\"/renderdoc/renderdoccmd\" vulkanlayer --register --user".to_string(),
                "\"/renderdoc/renderdoccmd\" vulkanlayer --register --system".to_string(),
            ],
        };
        let feedback = collect_environment_feedback(&EnvironmentAssessmentInputs {
            root_dir: Path::new("/renderdoc"),
            renderdoccmd_exe: Path::new("/renderdoc/renderdoccmd"),
            platform: "linux",
            arch: "x86_64",
            is_elevated: Some(false),
            renderdoccmd_version: Some("1.44"),
            workspace_renderdoc_version: Some("1.44"),
            replay_version_match: Some(true),
            vulkan_layer: Some(&vk),
            vulkan_layer_manifests: &[],
            env: &[],
        });

        assert!(
            feedback
                .warnings
                .iter()
                .any(|warning| warning == "RenderDoc Vulkan layer is not correctly registered")
        );
        assert!(
            feedback
                .warnings
                .iter()
                .any(|warning| warning.contains("administrator privileges"))
        );
        assert!(
            feedback
                .suggested_commands
                .iter()
                .any(|cmd| cmd.ends_with("vulkanlayer --register --user"))
        );
        assert!(
            feedback
                .suggested_commands
                .iter()
                .any(|cmd| cmd.contains("capture <your_exe> [args...]"))
        );
    }
}
