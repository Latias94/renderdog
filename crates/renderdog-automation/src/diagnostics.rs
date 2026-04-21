use std::{
    collections::BTreeSet,
    ffi::OsStr,
    path::{Component, Path, PathBuf},
    process::Command,
    string::String,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    RenderDocInstallation,
    version_policy::{renderdoc_versions_match, workspace_renderdoc_replay_version},
};

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
pub struct InstallationProbeSummary {
    pub renderdoccmd_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renderdoccmd_version_error: Option<String>,
    pub workspace_renderdoc_version: String,
    pub replay_version_match: Option<bool>,
    pub vulkan_layer: Option<VulkanLayerDiagnosis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulkan_layer_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InstallationDetection {
    pub root_dir: String,
    pub qrenderdoc_exe: String,
    pub renderdoccmd_exe: String,
    pub renderdoccmd_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renderdoccmd_version_error: Option<String>,
    pub workspace_renderdoc_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_version_match: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulkan_layer: Option<VulkanLayerDiagnosis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulkan_layer_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnvironmentDiagnosis {
    #[serde(flatten)]
    pub installation: InstallationDetection,
    pub platform: String,
    pub arch: String,
    pub is_elevated: Option<bool>,
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
    renderdoccmd_exe: &'a Path,
    platform: &'a str,
    arch: &'a str,
    is_elevated: Option<bool>,
    renderdoccmd_version: Option<&'a str>,
    renderdoccmd_version_error: Option<&'a str>,
    workspace_renderdoc_version: &'a str,
    replay_version_match: Option<bool>,
    vulkan_layer: Option<&'a VulkanLayerDiagnosis>,
    vulkan_layer_error: Option<&'a str>,
    vulkan_layer_manifests: &'a [String],
    env: &'a [EnvironmentVarInfo],
}

struct EnvironmentFeedback {
    warnings: Vec<String>,
    suggested_commands: Vec<String>,
}

#[derive(Debug, Clone)]
enum EnvironmentFinding {
    RenderdocVersionQueryFailed {
        error: String,
    },
    VulkanLayerDiagnosisFailed {
        error: String,
    },
    ExperimentalMacOsSupport,
    UnsupportedLinuxArch {
        arch: String,
    },
    UnsupportedWindowsArch {
        arch: String,
    },
    ReplayVersionMismatch {
        installed: String,
        workspace: String,
    },
    VulkanLayerStatus {
        summary: String,
    },
    VulkanLayerAttention {
        summary: String,
        suggested_commands: Vec<String>,
    },
    VulkanLayerNeedsElevation,
    MissingRenderDocInstanceLayer,
    MissingRenderDocManifestDir {
        manifest_dirs: Vec<PathBuf>,
    },
}

impl EnvironmentFinding {
    fn warning(&self) -> String {
        match self {
            Self::RenderdocVersionQueryFailed { error } => {
                format!("Failed to query `renderdoccmd version`: {error}")
            }
            Self::VulkanLayerDiagnosisFailed { error } => {
                format!("Failed to diagnose Vulkan layer registration: {error}")
            }
            Self::ExperimentalMacOsSupport => "RenderDoc on macOS is experimental and not officially supported for debugging; capture/replay may be unreliable.".to_string(),
            Self::UnsupportedLinuxArch { arch } => format!(
                "RenderDoc officially supports only x86_64 Linux; current arch is `{arch}` (ARM/32-bit targets are not supported)."
            ),
            Self::UnsupportedWindowsArch { arch } => format!(
                "RenderDoc Windows support is primarily x86_64; current arch is `{arch}` and may not work."
            ),
            Self::ReplayVersionMismatch {
                installed,
                workspace,
            } => format!(
                "Installed RenderDoc version `{installed}` does not match workspace replay headers `{workspace}`; `renderdog-replay` requires an exact match and should be rebuilt after switching versions."
            ),
            Self::VulkanLayerStatus { summary } => summary.clone(),
            Self::VulkanLayerAttention { summary, .. } => summary.clone(),
            Self::VulkanLayerNeedsElevation => "Vulkan layer registration may require administrator privileges. Re-run the registration command as administrator.".to_string(),
            Self::MissingRenderDocInstanceLayer => "VK_INSTANCE_LAYERS is set but does not include VK_LAYER_RENDERDOC_Capture; this can prevent RenderDoc's layer from being enabled.".to_string(),
            Self::MissingRenderDocManifestDir { manifest_dirs } => format!(
                "VK_LAYER_PATH is set but does not appear to include the RenderDoc Vulkan layer manifest directory. Detected manifest dirs: {}",
                display_paths(manifest_dirs).join(" | ")
            ),
        }
    }

    fn suggested_commands(&self, renderdoccmd_exe: &Path) -> Vec<String> {
        match self {
            Self::ReplayVersionMismatch { workspace, .. } => vec![format!(
                "Install or select RenderDoc `{workspace}` when using `renderdog-replay`, or switch `third-party/renderdoc` to match the installed version and rebuild."
            )],
            Self::VulkanLayerAttention {
                suggested_commands, ..
            } => {
                let mut commands = suggested_commands.clone();
                commands.push(format!(
                    "\"{}\" capture <your_exe> [args...] (fallback: injection-based capture)",
                    renderdoccmd_exe.display()
                ));
                commands
            }
            Self::MissingRenderDocInstanceLayer => vec![
                "Set VK_INSTANCE_LAYERS to include VK_LAYER_RENDERDOC_Capture, or clear it if it is forcing a different layer set.".to_string(),
            ],
            Self::MissingRenderDocManifestDir { .. } => {
                let sep = if cfg!(windows) { ";" } else { ":" };
                vec![format!(
                    "Update VK_LAYER_PATH to include the detected directories (separator `{sep}`), or unset VK_LAYER_PATH if it is causing conflicts."
                )]
            }
            _ => Vec::new(),
        }
    }
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

    pub fn diagnose_environment(&self) -> EnvironmentDiagnosis {
        let installation = self.describe_installation();
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
            renderdoccmd_exe: &self.renderdoccmd_exe,
            platform: &platform,
            arch: &arch,
            is_elevated,
            renderdoccmd_version: installation.renderdoccmd_version.as_deref(),
            renderdoccmd_version_error: installation.renderdoccmd_version_error.as_deref(),
            workspace_renderdoc_version: installation.workspace_renderdoc_version.as_str(),
            replay_version_match: installation.replay_version_match,
            vulkan_layer: installation.vulkan_layer.as_ref(),
            vulkan_layer_error: installation.vulkan_layer_error.as_deref(),
            vulkan_layer_manifests: &vulkan_layer_manifests,
            env: &env,
        });

        EnvironmentDiagnosis {
            installation,
            platform,
            arch,
            is_elevated,
            vulkan_layer_manifests,
            env,
            warnings: feedback.warnings,
            suggested_commands: feedback.suggested_commands,
        }
    }

    pub fn probe_installation(&self) -> InstallationProbeSummary {
        let (renderdoccmd_version, renderdoccmd_version_error) = match self.version() {
            Ok(version) => (Some(version.trim().to_string()), None),
            Err(err) => (None, Some(err.to_string())),
        };
        let workspace_renderdoc_version = workspace_renderdoc_replay_version().to_owned();
        let replay_version_match = compute_replay_version_match(
            renderdoccmd_version.as_deref(),
            &workspace_renderdoc_version,
        );
        let (vulkan_layer, vulkan_layer_error) = match self.diagnose_vulkan_layer() {
            Ok(diag) => (Some(diag), None),
            Err(err) => (None, Some(err.to_string())),
        };

        InstallationProbeSummary {
            renderdoccmd_version,
            renderdoccmd_version_error,
            workspace_renderdoc_version,
            replay_version_match,
            vulkan_layer,
            vulkan_layer_error,
        }
    }

    pub fn describe_installation(&self) -> InstallationDetection {
        let probe = self.probe_installation();

        InstallationDetection {
            root_dir: self.root_dir.display().to_string(),
            qrenderdoc_exe: self.qrenderdoc_exe.display().to_string(),
            renderdoccmd_exe: self.renderdoccmd_exe.display().to_string(),
            renderdoccmd_version: probe.renderdoccmd_version,
            renderdoccmd_version_error: probe.renderdoccmd_version_error,
            workspace_renderdoc_version: probe.workspace_renderdoc_version,
            replay_version_match: probe.replay_version_match,
            vulkan_layer: probe.vulkan_layer,
            vulkan_layer_error: probe.vulkan_layer_error,
        }
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
    workspace_renderdoc_version: &str,
) -> Option<bool> {
    renderdoccmd_version
        .map(|installed| renderdoc_versions_match(installed, workspace_renderdoc_version))
}

fn assess_environment(inputs: &EnvironmentAssessmentInputs<'_>) -> Vec<EnvironmentFinding> {
    let mut findings = Vec::new();

    if let Some(err) = inputs.renderdoccmd_version_error {
        findings.push(EnvironmentFinding::RenderdocVersionQueryFailed {
            error: err.to_string(),
        });
    }

    if let Some(err) = inputs.vulkan_layer_error {
        findings.push(EnvironmentFinding::VulkanLayerDiagnosisFailed {
            error: err.to_string(),
        });
    }

    if inputs.platform == "macos" {
        findings.push(EnvironmentFinding::ExperimentalMacOsSupport);
    }

    if inputs.platform == "linux" && inputs.arch != "x86_64" {
        findings.push(EnvironmentFinding::UnsupportedLinuxArch {
            arch: inputs.arch.to_string(),
        });
    }

    if inputs.platform == "windows" && inputs.arch != "x86_64" {
        findings.push(EnvironmentFinding::UnsupportedWindowsArch {
            arch: inputs.arch.to_string(),
        });
    }

    if let (Some(false), Some(installed)) =
        (inputs.replay_version_match, inputs.renderdoccmd_version)
    {
        findings.push(EnvironmentFinding::ReplayVersionMismatch {
            installed: installed.to_string(),
            workspace: inputs.workspace_renderdoc_version.to_string(),
        });
    }

    if let Some(vk) = inputs.vulkan_layer {
        if !vk.supported || vk.unfixable {
            findings.push(EnvironmentFinding::VulkanLayerStatus {
                summary: vk.summary.clone(),
            });
        } else if vk.needs_attention {
            findings.push(EnvironmentFinding::VulkanLayerAttention {
                summary: vk.summary.clone(),
                suggested_commands: vk.suggested_commands.clone(),
            });
        }
    }

    if let (Some(vk), Some(false)) = (inputs.vulkan_layer, inputs.is_elevated)
        && vk.need_elevation
        && vk.needs_attention
    {
        findings.push(EnvironmentFinding::VulkanLayerNeedsElevation);
    }

    let vk_instance_layers = env_var_value(inputs.env, "VK_INSTANCE_LAYERS").unwrap_or("");
    if !vk_instance_layers.is_empty() && !vk_instance_layers.contains("VK_LAYER_RENDERDOC_Capture")
    {
        findings.push(EnvironmentFinding::MissingRenderDocInstanceLayer);
    }

    let vk_layer_path = env_var_value(inputs.env, "VK_LAYER_PATH").unwrap_or("");
    let manifest_dirs = vulkan_layer_manifest_dirs(inputs.vulkan_layer_manifests);
    if !manifest_dirs.is_empty() && !vk_layer_path.is_empty() {
        let search_paths = split_search_path_list(vk_layer_path);
        let any_in_path = manifest_dirs
            .iter()
            .any(|dir| search_paths.iter().any(|entry| paths_match(entry, dir)));
        if !any_in_path {
            findings.push(EnvironmentFinding::MissingRenderDocManifestDir { manifest_dirs });
        }
    }

    findings
}

fn collect_environment_feedback(inputs: &EnvironmentAssessmentInputs<'_>) -> EnvironmentFeedback {
    let findings = assess_environment(inputs);
    let warnings = findings.iter().map(EnvironmentFinding::warning).collect();
    let suggested_commands = findings
        .iter()
        .flat_map(|finding| finding.suggested_commands(inputs.renderdoccmd_exe))
        .collect();

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

fn split_search_path_list(value: &str) -> Vec<PathBuf> {
    std::env::split_paths(OsStr::new(value))
        .map(|path| normalize_path_for_match(&path))
        .collect()
}

fn vulkan_layer_manifest_dirs(manifests: &[String]) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    for manifest in manifests {
        let Some(parent) = Path::new(manifest).parent() else {
            continue;
        };
        let parent = normalize_path_for_match(parent);
        if dirs.iter().any(|existing| paths_match(existing, &parent)) {
            continue;
        }
        dirs.push(parent);
    }
    dirs.sort_by(|lhs, rhs| lhs.as_os_str().cmp(rhs.as_os_str()));
    dirs
}

fn display_paths(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect()
}

fn paths_match(lhs: &Path, rhs: &Path) -> bool {
    let lhs = normalize_path_for_match(lhs);
    let rhs = normalize_path_for_match(rhs);

    if cfg!(windows) {
        lhs.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&rhs.as_os_str().to_string_lossy())
    } else {
        lhs == rhs
    }
}

fn normalize_path_for_match(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
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
    let home_dir = std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from));
    let xdg_data_home = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from);

    find_vulkan_layer_manifests_with_hints(root_dir, home_dir.as_deref(), xdg_data_home.as_deref())
}

fn find_vulkan_layer_manifests_with_hints(
    root_dir: &Path,
    home_dir: Option<&Path>,
    xdg_data_home: Option<&Path>,
) -> Vec<String> {
    let mut hits: Vec<String> =
        candidate_vulkan_layer_manifest_paths(root_dir, home_dir, xdg_data_home)
            .into_iter()
            .filter(|path| path.is_file())
            .filter_map(|path| {
                let content = std::fs::read_to_string(&path).ok()?;
                if content.contains("VK_LAYER_RENDERDOC_Capture") {
                    Some(path.display().to_string())
                } else {
                    None
                }
            })
            .collect();

    hits.sort();
    hits.dedup();
    hits
}

fn candidate_vulkan_layer_manifest_paths(
    root_dir: &Path,
    home_dir: Option<&Path>,
    xdg_data_home: Option<&Path>,
) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = vec![
        root_dir.to_path_buf(),
        root_dir
            .join("share")
            .join("vulkan")
            .join("implicit_layer.d"),
        root_dir.join("etc").join("vulkan").join("implicit_layer.d"),
    ];

    #[cfg(not(windows))]
    {
        dirs.push(PathBuf::from("/usr/share/vulkan/implicit_layer.d"));
        dirs.push(PathBuf::from("/etc/vulkan/implicit_layer.d"));
    }

    if let Some(xdg_data_home) = xdg_data_home.filter(|path| !path.as_os_str().is_empty()) {
        dirs.push(xdg_data_home.join("vulkan").join("implicit_layer.d"));
    } else if let Some(home_dir) = home_dir.filter(|path| !path.as_os_str().is_empty()) {
        dirs.push(
            home_dir
                .join(".local")
                .join("share")
                .join("vulkan")
                .join("implicit_layer.d"),
        );
    }

    let mut paths: Vec<PathBuf> = Vec::new();
    for dir in dirs {
        let dir = normalize_path_for_match(&dir);
        for basename in vulkan_layer_manifest_basenames() {
            let candidate = dir.join(basename);
            if paths
                .iter()
                .any(|existing| paths_match(existing, &candidate))
            {
                continue;
            }
            paths.push(candidate);
        }
    }

    paths
}

fn vulkan_layer_manifest_basenames() -> &'static [&'static str] {
    &["renderdoc_capture.json", "renderdoc.json"]
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
    use crate::version_policy::workspace_renderdoc_replay_version;

    use super::{
        EnvironmentAssessmentInputs, EnvironmentVarInfo, VulkanLayerDiagnosis,
        candidate_vulkan_layer_manifest_paths, collect_environment_feedback,
        compute_replay_version_match, find_vulkan_layer_manifests_with_hints,
        parse_vulkan_layer_diagnosis, paths_match, split_search_path_list,
        vulkan_layer_manifest_dirs,
    };
    use std::{
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn make_temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "renderdog-diagnostics-test-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir
    }

    fn workspace_replay_version() -> &'static str {
        workspace_renderdoc_replay_version()
    }

    fn mismatched_replay_version() -> &'static str {
        match workspace_replay_version() {
            "0.0" => "999.999",
            _ => "0.0",
        }
    }

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
        let workspace_version = workspace_replay_version();
        let mismatched_version = mismatched_replay_version();
        assert_eq!(
            compute_replay_version_match(Some(&format!("v{workspace_version}")), workspace_version),
            Some(true)
        );
        assert_eq!(
            compute_replay_version_match(Some(mismatched_version), workspace_version),
            Some(false)
        );
        assert_eq!(compute_replay_version_match(None, workspace_version), None);
    }

    #[test]
    fn collect_environment_feedback_reports_version_and_layer_overrides() {
        let workspace_version = workspace_replay_version();
        let mismatched_version = mismatched_replay_version();
        let vk_layer_path = std::env::join_paths([Path::new("custom/layers")])
            .expect("valid path list")
            .to_string_lossy()
            .into_owned();
        let env = vec![
            EnvironmentVarInfo {
                name: "VK_INSTANCE_LAYERS".to_string(),
                value: Some("VK_LAYER_OTHER".to_string()),
            },
            EnvironmentVarInfo {
                name: "VK_LAYER_PATH".to_string(),
                value: Some(vk_layer_path),
            },
        ];
        let feedback = collect_environment_feedback(&EnvironmentAssessmentInputs {
            renderdoccmd_exe: Path::new("renderdoc/renderdoccmd"),
            platform: "linux",
            arch: "x86_64",
            is_elevated: None,
            renderdoccmd_version: Some(mismatched_version),
            renderdoccmd_version_error: None,
            workspace_renderdoc_version: workspace_version,
            replay_version_match: Some(false),
            vulkan_layer: None,
            vulkan_layer_error: None,
            vulkan_layer_manifests: &["/opt/renderdoc/renderdoc_capture.json".to_string()],
            env: &env,
        });

        assert!(
            feedback
                .warnings
                .iter()
                .any(|warning| warning.contains(&format!(
                    "does not match workspace replay headers `{workspace_version}`"
                )))
        );
        assert!(feedback.warnings.iter().any(|warning| warning.contains(
            "VK_INSTANCE_LAYERS is set but does not include VK_LAYER_RENDERDOC_Capture"
        )));
        assert!(feedback.warnings.iter().any(|warning| {
            warning.contains(
                "does not appear to include the RenderDoc Vulkan layer manifest directory",
            )
        }));
        assert!(
            feedback
                .suggested_commands
                .iter()
                .any(|cmd| cmd.contains(&format!(
                    "Install or select RenderDoc `{workspace_version}`"
                )))
        );
        assert!(feedback.suggested_commands.iter().any(|cmd| {
            cmd.contains("Set VK_INSTANCE_LAYERS to include VK_LAYER_RENDERDOC_Capture")
        }));
    }

    #[test]
    fn collect_environment_feedback_reports_vk_registration_actions() {
        let workspace_version = workspace_replay_version();
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
            renderdoccmd_exe: Path::new("/renderdoc/renderdoccmd"),
            platform: "linux",
            arch: "x86_64",
            is_elevated: Some(false),
            renderdoccmd_version: Some(workspace_version),
            renderdoccmd_version_error: None,
            workspace_renderdoc_version: workspace_version,
            replay_version_match: Some(true),
            vulkan_layer: Some(&vk),
            vulkan_layer_error: None,
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

    #[test]
    fn collect_environment_feedback_surfaces_probe_errors() {
        let workspace_version = workspace_replay_version();
        let feedback = collect_environment_feedback(&EnvironmentAssessmentInputs {
            renderdoccmd_exe: Path::new("/renderdoc/renderdoccmd"),
            platform: "linux",
            arch: "x86_64",
            is_elevated: None,
            renderdoccmd_version: None,
            renderdoccmd_version_error: Some("spawn failed"),
            workspace_renderdoc_version: workspace_version,
            replay_version_match: None,
            vulkan_layer: None,
            vulkan_layer_error: Some("renderdoccmd output was not valid UTF-8"),
            vulkan_layer_manifests: &[],
            env: &[],
        });

        assert!(feedback.warnings.iter().any(|warning| {
            warning.contains("Failed to query `renderdoccmd version`: spawn failed")
        }));
        assert!(feedback.warnings.iter().any(|warning| {
            warning.contains("Failed to diagnose Vulkan layer registration: renderdoccmd output was not valid UTF-8")
        }));
    }

    #[test]
    fn split_search_path_list_and_manifest_dirs_use_exact_path_matching() {
        let vk_layer_path =
            std::env::join_paths([Path::new("renderdoc-old/layers"), Path::new("other/layers")])
                .expect("valid path list")
                .to_string_lossy()
                .into_owned();
        let search_paths = split_search_path_list(&vk_layer_path);
        let manifest_dirs =
            vulkan_layer_manifest_dirs(&["renderdoc/layers/renderdoc_capture.json".to_string()]);

        assert_eq!(manifest_dirs.len(), 1);
        assert!(
            !search_paths
                .iter()
                .any(|entry| paths_match(entry, &manifest_dirs[0])),
            "substring path entries must not count as exact manifest dir matches"
        );
    }

    #[test]
    fn candidate_vulkan_layer_manifest_paths_include_known_install_locations() {
        let root_dir = Path::new("/opt/renderdoc");
        let home_dir = Path::new("/home/tester");
        let xdg_data_home = Path::new("/tmp/xdg-data");

        let candidates =
            candidate_vulkan_layer_manifest_paths(root_dir, Some(home_dir), Some(xdg_data_home));
        assert!(
            candidates
                .iter()
                .any(|path| paths_match(path, &root_dir.join("renderdoc.json")))
        );
        assert!(
            candidates
                .iter()
                .any(|path| paths_match(path, &root_dir.join("renderdoc_capture.json")))
        );
        assert!(candidates.iter().any(|path| {
            paths_match(
                path,
                &root_dir
                    .join("share")
                    .join("vulkan")
                    .join("implicit_layer.d")
                    .join("renderdoc_capture.json"),
            )
        }));
        assert!(candidates.iter().any(|path| {
            paths_match(
                path,
                &xdg_data_home
                    .join("vulkan")
                    .join("implicit_layer.d")
                    .join("renderdoc_capture.json"),
            )
        }));
    }

    #[test]
    fn find_vulkan_layer_manifests_with_hints_only_reads_known_candidates() {
        let root_dir = make_temp_dir();
        let home_dir = root_dir.join("home");
        let xdg_data_home = root_dir.join("xdg");

        let candidate_manifest = xdg_data_home
            .join("vulkan")
            .join("implicit_layer.d")
            .join("renderdoc_capture.json");
        std::fs::create_dir_all(
            candidate_manifest
                .parent()
                .expect("candidate manifest should have parent"),
        )
        .expect("failed to create candidate dir");
        std::fs::write(
            &candidate_manifest,
            r#"{"layer":{"name":"VK_LAYER_RENDERDOC_Capture"}}"#,
        )
        .expect("failed to write candidate manifest");

        let unrelated_manifest = root_dir
            .join("deep")
            .join("nested")
            .join("renderdoc_capture.json");
        std::fs::create_dir_all(
            unrelated_manifest
                .parent()
                .expect("unrelated manifest should have parent"),
        )
        .expect("failed to create unrelated dir");
        std::fs::write(
            &unrelated_manifest,
            r#"{"layer":{"name":"VK_LAYER_RENDERDOC_Capture"}}"#,
        )
        .expect("failed to write unrelated manifest");

        let hits = find_vulkan_layer_manifests_with_hints(
            &root_dir,
            Some(&home_dir),
            Some(&xdg_data_home),
        );

        assert_eq!(hits, vec![candidate_manifest.display().to_string()]);

        std::fs::remove_dir_all(&root_dir).expect("cleanup should succeed");
    }
}
