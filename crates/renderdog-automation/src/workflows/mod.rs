//! High-level RenderDoc workflows built on `qrenderdoc --python`.

mod export_actions;
mod export_bindings_index;
mod export_bundle;
mod find_and_save_outputs;
mod find_events;
mod one_shot;
mod trigger_capture;

pub use export_bundle::ExportBundleError;
pub use find_and_save_outputs::{
    FindEventSelection, FindEventsAndSaveOutputsPngError, FindEventsAndSaveOutputsPngRequest,
    FindEventsAndSaveOutputsPngResponse,
};
pub use find_events::FindEventsError;
pub use one_shot::{
    CaptureAndExportBundleError, CaptureAndExportBundleRequest, CaptureAndExportBundleResponse,
    OneShotCaptureError,
};
pub use trigger_capture::TriggerCaptureError;

use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::scripting::PrepareQRenderDocJsonRequest;
use crate::{
    QRenderDocJsonError, normalize_capture_path, prepare_export_target,
    resolve_path_string_from_cwd,
};

fn default_max_results() -> Option<u32> {
    Some(200)
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_frames() -> u32 {
    1
}

fn default_timeout_s() -> u32 {
    60
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CapturePath {
    pub capture_path: String,
}

pub type CaptureInput = CapturePath;
pub type CaptureRef = CapturePath;

impl CapturePath {
    pub(crate) fn normalized_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            capture_path: normalize_capture_path(cwd, &self.capture_path),
        }
    }

    pub(crate) fn new(capture_path: impl Into<String>) -> Self {
        Self {
            capture_path: capture_path.into(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportOutput {
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub basename: Option<String>,
}

impl ExportOutput {
    pub(crate) fn normalized_for_capture(
        &self,
        cwd: &Path,
        capture: &CaptureInput,
    ) -> Result<(CaptureInput, Self), std::io::Error> {
        let prepared = prepare_export_target(
            cwd,
            &capture.capture_path,
            self.output_dir.as_deref(),
            self.basename.as_deref(),
        )?;

        Ok((
            CaptureInput {
                capture_path: prepared.capture_path,
            },
            Self {
                output_dir: Some(prepared.output_dir),
                basename: Some(prepared.basename),
            },
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OutputPath {
    pub output_path: String,
}

pub type OutputFile = OutputPath;
pub type OutputRef = OutputPath;

impl OutputPath {
    pub(crate) fn resolved_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            output_path: resolve_path_string_from_cwd(cwd, &self.output_path),
        }
    }

    pub(crate) fn new(output_path: impl Into<String>) -> Self {
        Self {
            output_path: output_path.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct TargetControlRef {
    pub target_ident: u32,
}

impl TargetControlRef {
    pub(crate) fn new(target_ident: u32) -> Self {
        Self { target_ident }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CapturePostActions {
    #[serde(default)]
    pub save_thumbnail: bool,
    #[serde(default)]
    pub thumbnail_output_path: Option<String>,
    #[serde(default)]
    pub open_capture_ui: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CapturePostActionOutputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_pid: Option<u32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EventFilter {
    #[serde(default)]
    pub marker_prefix: Option<String>,
    #[serde(default)]
    pub event_id_min: Option<u32>,
    #[serde(default)]
    pub event_id_max: Option<u32>,
    #[serde(default)]
    pub name_contains: Option<String>,
    #[serde(default)]
    pub marker_contains: Option<String>,
    #[serde(default)]
    pub case_sensitive: bool,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct DrawcallScope {
    #[serde(default)]
    pub only_drawcalls: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct FindEventsLimit {
    #[serde(default = "default_max_results")]
    pub max_results: Option<u32>,
}

impl Default for FindEventsLimit {
    fn default() -> Self {
        Self {
            max_results: default_max_results(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct BindingsExportOptions {
    #[serde(default)]
    pub include_cbuffers: bool,
    #[serde(default)]
    pub include_outputs: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BundleExportOptions {
    #[serde(flatten)]
    pub drawcall_scope: DrawcallScope,
    #[serde(flatten)]
    pub filter: EventFilter,
    #[serde(flatten)]
    pub bindings: BindingsExportOptions,
    #[serde(flatten)]
    pub post_actions: CapturePostActions,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ActionsExportArtifacts {
    pub actions_jsonl_path: String,
    pub actions_summary_json_path: String,
    pub total_actions: u64,
    pub drawcall_actions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BindingsExportArtifacts {
    pub bindings_jsonl_path: String,
    pub bindings_summary_json_path: String,
    pub total_drawcalls: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BundleExportArtifacts {
    #[serde(flatten)]
    pub actions: ActionsExportArtifacts,
    #[serde(flatten)]
    pub bindings: BindingsExportArtifacts,
    #[serde(flatten)]
    pub post_actions: CapturePostActionOutputs,
}

impl BundleExportArtifacts {
    pub(crate) fn from_parts(
        actions: ExportActionsResponse,
        bindings: ExportBindingsIndexResponse,
        post_actions: CapturePostActionOutputs,
    ) -> Self {
        let ExportActionsResponse { artifacts: actions } = actions;
        let ExportBindingsIndexResponse {
            artifacts: bindings,
        } = bindings;

        Self {
            actions,
            bindings,
            post_actions,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TriggerCaptureOptions {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_frames")]
    pub num_frames: u32,
    #[serde(default = "default_timeout_s")]
    pub timeout_s: u32,
}

impl Default for TriggerCaptureOptions {
    fn default() -> Self {
        Self {
            host: default_host(),
            num_frames: default_frames(),
            timeout_s: default_timeout_s(),
        }
    }
}

impl TriggerCaptureOptions {
    pub(crate) fn for_target(&self, target: TargetControlRef) -> TriggerCaptureRequest {
        TriggerCaptureRequest {
            target,
            trigger: self.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TriggerCaptureRequest {
    #[serde(flatten)]
    pub target: TargetControlRef,
    #[serde(flatten)]
    pub trigger: TriggerCaptureOptions,
}

impl PrepareQRenderDocJsonRequest for TriggerCaptureRequest {
    type Error = QRenderDocJsonError;

    fn prepare_in_cwd(&self, _cwd: &Path) -> Result<Self, Self::Error> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TriggerCaptureResponse {
    #[serde(flatten)]
    pub capture: CaptureRef,
    pub frame_number: u32,
    pub api: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub(crate) struct ExportActionsRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(flatten)]
    pub drawcall_scope: DrawcallScope,
    #[serde(flatten)]
    pub filter: EventFilter,
}

impl PrepareQRenderDocJsonRequest for ExportActionsRequest {
    type Error = QRenderDocJsonError;

    fn prepare_in_cwd(&self, _cwd: &Path) -> Result<Self, Self::Error> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub(crate) struct ExportActionsResponse {
    #[serde(flatten)]
    pub artifacts: ActionsExportArtifacts,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindEventsRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(flatten)]
    pub drawcall_scope: DrawcallScope,
    #[serde(flatten)]
    pub filter: EventFilter,
    #[serde(flatten)]
    pub limit: FindEventsLimit,
}

impl FindEventsRequest {
    pub(crate) fn normalized_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            capture: self.capture.normalized_in_cwd(cwd),
            ..self.clone()
        }
    }
}

impl PrepareQRenderDocJsonRequest for FindEventsRequest {
    type Error = QRenderDocJsonError;

    fn prepare_in_cwd(&self, cwd: &Path) -> Result<Self, Self::Error> {
        Ok(self.normalized_in_cwd(cwd))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct MarkerPath(pub Vec<String>);

impl MarkerPath {
    pub fn joined(&self) -> String {
        self.0.join("/")
    }
}

impl From<Vec<String>> for MarkerPath {
    fn from(value: Vec<String>) -> Self {
        Self(value)
    }
}

const EVENT_FLAG_NAMES: &[(u32, &str)] = &[
    (0x0001, "Clear"),
    (0x0002, "Drawcall"),
    (0x0004, "Dispatch"),
    (0x0008, "MeshDispatch"),
    (0x0010, "CmdList"),
    (0x0020, "SetMarker"),
    (0x0040, "PushMarker"),
    (0x0080, "PopMarker"),
    (0x0100, "Present"),
    (0x0200, "MultiAction"),
    (0x0400, "Copy"),
    (0x0800, "Resolve"),
    (0x1000, "GenMips"),
    (0x2000, "PassBoundary"),
    (0x4000, "DispatchRay"),
    (0x8000, "BuildAccStruct"),
    (0x010000, "Indexed"),
    (0x020000, "Instanced"),
    (0x040000, "Auto"),
    (0x080000, "Indirect"),
    (0x100000, "ClearColor"),
    (0x200000, "ClearDepthStencil"),
    (0x400000, "BeginPass"),
    (0x800000, "EndPass"),
    (0x1000000, "CommandBufferBoundary"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct EventFlags {
    pub bits: u32,
}

impl EventFlags {
    pub const DRAWCALL: Self = Self { bits: 0x0002 };

    pub fn new(bits: u32) -> Self {
        Self { bits }
    }

    pub fn names(&self) -> Vec<&'static str> {
        EVENT_FLAG_NAMES
            .iter()
            .filter_map(|(bit, name)| (self.bits & bit != 0).then_some(*name))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FoundEvent {
    pub event_id: u32,
    pub parent_event_id: Option<u32>,
    pub depth: u32,
    pub name: String,
    pub flags: EventFlags,
    pub marker_path: MarkerPath,
}

impl FoundEvent {
    pub fn marker_path_joined(&self) -> String {
        self.marker_path.joined()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindEventsSummary {
    pub total_matches: u64,
    pub truncated: bool,
    pub first_event_id: Option<u32>,
    pub last_event_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindEventsResponse {
    #[serde(flatten)]
    pub capture: CaptureRef,
    pub summary: FindEventsSummary,
    pub matches: Vec<FoundEvent>,
}

impl FindEventsResponse {
    pub fn first_event_id(&self) -> Option<u32> {
        self.summary.first_event_id
    }

    pub fn last_event_id(&self) -> Option<u32> {
        self.summary.last_event_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub(crate) struct ExportBindingsIndexRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(flatten)]
    pub filter: EventFilter,
    #[serde(flatten)]
    pub bindings: BindingsExportOptions,
}

impl PrepareQRenderDocJsonRequest for ExportBindingsIndexRequest {
    type Error = QRenderDocJsonError;

    fn prepare_in_cwd(&self, _cwd: &Path) -> Result<Self, Self::Error> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub(crate) struct ExportBindingsIndexResponse {
    #[serde(flatten)]
    pub artifacts: BindingsExportArtifacts,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportBundleRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(flatten)]
    pub bundle: BundleExportOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportBundleResponse {
    #[serde(flatten)]
    pub capture: CaptureRef,
    #[serde(flatten)]
    pub artifacts: BundleExportArtifacts,
}

impl ExportBundleResponse {
    pub(crate) fn from_parts(capture_path: String, artifacts: BundleExportArtifacts) -> Self {
        Self {
            capture: CaptureRef::new(capture_path),
            artifacts,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::Value;

    use super::{
        ActionsExportArtifacts, BindingsExportArtifacts, BindingsExportOptions,
        BundleExportArtifacts, BundleExportOptions, CaptureInput, CapturePostActionOutputs,
        CapturePostActions, CaptureRef, DrawcallScope, EventFilter, EventFlags,
        ExportActionsResponse, ExportBindingsIndexResponse, ExportBundleRequest,
        ExportBundleResponse, ExportOutput, FindEventsResponse, FindEventsSummary, FoundEvent,
        MarkerPath, OutputFile, TargetControlRef, TriggerCaptureOptions, TriggerCaptureRequest,
        TriggerCaptureResponse,
    };

    #[test]
    fn capture_input_normalizes_relative_path_in_cwd() {
        let capture = CaptureInput {
            capture_path: "captures/frame.rdc".to_string(),
        };

        let normalized = capture.normalized_in_cwd(Path::new("/tmp/project"));

        assert_eq!(normalized.capture_path, "/tmp/project/captures/frame.rdc");
    }

    #[test]
    fn export_output_normalization_uses_capture_basename_when_missing() {
        let capture = CaptureInput {
            capture_path: "captures/frame.rdc".to_string(),
        };
        let output = ExportOutput::default();

        let (capture, output) = output
            .normalized_for_capture(Path::new("/tmp/project"), &capture)
            .expect("normalize export target");

        assert_eq!(capture.capture_path, "/tmp/project/captures/frame.rdc");
        assert_eq!(
            output.output_dir.as_deref(),
            Some("/tmp/project/artifacts/renderdoc/exports")
        );
        assert_eq!(output.basename.as_deref(), Some("frame"));
    }

    #[test]
    fn output_file_resolves_relative_path_in_cwd() {
        let output = OutputFile {
            output_path: "artifacts/frame.png".to_string(),
        };

        let resolved = output.resolved_in_cwd(Path::new("/tmp/project"));

        assert_eq!(resolved.output_path, "/tmp/project/artifacts/frame.png");
    }

    #[test]
    fn trigger_capture_options_build_request_for_target() {
        let req = TriggerCaptureOptions {
            host: "renderdoc-host".to_string(),
            num_frames: 3,
            timeout_s: 90,
        }
        .for_target(TargetControlRef::new(17));

        assert_eq!(req.target.target_ident, 17);
        assert_eq!(req.trigger.host, "renderdoc-host");
        assert_eq!(req.trigger.num_frames, 3);
        assert_eq!(req.trigger.timeout_s, 90);
    }

    #[test]
    fn trigger_capture_request_serializes_options_flattened() {
        let req = TriggerCaptureRequest {
            target: TargetControlRef::new(17),
            trigger: TriggerCaptureOptions {
                host: "renderdoc-host".to_string(),
                num_frames: 3,
                timeout_s: 90,
            },
        };

        let json = serde_json::to_value(req).expect("serialize request");
        let object = json.as_object().expect("request object");

        assert_eq!(
            object.get("target_ident"),
            Some(&Value::Number(17_u32.into()))
        );
        assert_eq!(
            object.get("host"),
            Some(&Value::String("renderdoc-host".to_string()))
        );
        assert_eq!(object.get("num_frames"), Some(&Value::Number(3_u32.into())));
        assert_eq!(object.get("timeout_s"), Some(&Value::Number(90_u32.into())));
        assert!(!object.contains_key("trigger"));
    }

    #[test]
    fn target_control_ref_serializes_flat_field() {
        let target = TargetControlRef::new(17);

        let json = serde_json::to_value(target).expect("serialize target");
        let object = json.as_object().expect("target object");

        assert_eq!(
            object.get("target_ident"),
            Some(&Value::Number(17_u32.into()))
        );
    }

    #[test]
    fn trigger_capture_response_serializes_capture_flattened() {
        let response = TriggerCaptureResponse {
            capture: CaptureRef::new("/tmp/frame.rdc"),
            frame_number: 2,
            api: "Vulkan".to_string(),
        };

        let json = serde_json::to_value(response).expect("serialize response");
        let object = json.as_object().expect("response object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(
            object.get("frame_number"),
            Some(&Value::Number(2_u32.into()))
        );
        assert_eq!(
            object.get("api"),
            Some(&Value::String("Vulkan".to_string()))
        );
        assert!(!object.contains_key("capture"));
    }

    #[test]
    fn export_bundle_request_serializes_bundle_options_flattened() {
        let req = ExportBundleRequest {
            capture: CaptureInput {
                capture_path: "/tmp/frame.rdc".to_string(),
            },
            output: ExportOutput {
                output_dir: Some("/tmp/out".to_string()),
                basename: Some("frame".to_string()),
            },
            bundle: BundleExportOptions {
                drawcall_scope: DrawcallScope {
                    only_drawcalls: true,
                },
                filter: EventFilter {
                    marker_contains: Some("fret".to_string()),
                    ..EventFilter::default()
                },
                bindings: BindingsExportOptions {
                    include_cbuffers: true,
                    include_outputs: false,
                },
                post_actions: CapturePostActions {
                    save_thumbnail: true,
                    thumbnail_output_path: Some("thumb.png".to_string()),
                    open_capture_ui: false,
                },
            },
        };

        let json = serde_json::to_value(req).expect("serialize request");
        let object = json.as_object().expect("request object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(
            object.get("output_dir"),
            Some(&Value::String("/tmp/out".to_string()))
        );
        assert_eq!(
            object.get("basename"),
            Some(&Value::String("frame".to_string()))
        );
        assert_eq!(object.get("only_drawcalls"), Some(&Value::Bool(true)));
        assert_eq!(
            object.get("marker_contains"),
            Some(&Value::String("fret".to_string()))
        );
        assert_eq!(object.get("include_cbuffers"), Some(&Value::Bool(true)));
        assert_eq!(object.get("save_thumbnail"), Some(&Value::Bool(true)));
        assert!(!object.contains_key("bundle"));
    }

    #[test]
    fn find_events_response_serializes_capture_flattened() {
        let response = FindEventsResponse {
            capture: CaptureRef::new("/tmp/frame.rdc"),
            summary: FindEventsSummary {
                total_matches: 3,
                truncated: false,
                first_event_id: Some(11),
                last_event_id: Some(42),
            },
            matches: Vec::new(),
        };

        let json = serde_json::to_value(response).expect("serialize response");
        let object = json.as_object().expect("response object");
        let summary = object
            .get("summary")
            .and_then(Value::as_object)
            .expect("summary object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(
            summary.get("total_matches"),
            Some(&Value::Number(3_u32.into()))
        );
        assert_eq!(
            summary.get("first_event_id"),
            Some(&Value::Number(11_u32.into()))
        );
        assert_eq!(
            summary.get("last_event_id"),
            Some(&Value::Number(42_u32.into()))
        );
        assert!(!object.contains_key("capture"));
    }

    #[test]
    fn found_event_derives_joined_marker_path() {
        let event = FoundEvent {
            event_id: 42,
            parent_event_id: Some(11),
            depth: 3,
            name: "DrawIndexed".to_string(),
            flags: EventFlags::DRAWCALL,
            marker_path: MarkerPath::from(vec!["Frame".to_string(), "Main".to_string()]),
        };

        let json = serde_json::to_value(&event).expect("serialize event");
        let object = json.as_object().expect("event object");

        assert_eq!(event.marker_path_joined(), "Frame/Main");
        assert_eq!(event.flags.names(), vec!["Drawcall"]);
        assert!(
            object
                .get("marker_path")
                .and_then(Value::as_array)
                .is_some()
        );
        assert_eq!(object.get("flags"), Some(&Value::Number(2_u32.into())));
        assert!(!object.contains_key("flags_names"));
        assert!(!object.contains_key("marker_path_joined"));
    }

    #[test]
    fn export_bundle_response_serializes_artifacts_flattened() {
        let response = ExportBundleResponse::from_parts(
            "/tmp/frame.rdc".to_string(),
            BundleExportArtifacts::from_parts(
                ExportActionsResponse {
                    artifacts: ActionsExportArtifacts {
                        actions_jsonl_path: "/tmp/out/frame.actions.jsonl".to_string(),
                        actions_summary_json_path: "/tmp/out/frame.summary.json".to_string(),
                        total_actions: 10,
                        drawcall_actions: 4,
                    },
                },
                ExportBindingsIndexResponse {
                    artifacts: BindingsExportArtifacts {
                        bindings_jsonl_path: "/tmp/out/frame.bindings.jsonl".to_string(),
                        bindings_summary_json_path: "/tmp/out/frame.bindings_summary.json"
                            .to_string(),
                        total_drawcalls: 4,
                    },
                },
                CapturePostActionOutputs {
                    thumbnail_output_path: Some("/tmp/out/frame.thumb.png".to_string()),
                    ui_pid: Some(123),
                },
            ),
        );

        let json = serde_json::to_value(response).expect("serialize response");
        let object = json.as_object().expect("response object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(
            object.get("actions_jsonl_path"),
            Some(&Value::String("/tmp/out/frame.actions.jsonl".to_string()))
        );
        assert_eq!(
            object.get("bindings_jsonl_path"),
            Some(&Value::String("/tmp/out/frame.bindings.jsonl".to_string()))
        );
        assert_eq!(
            object.get("thumbnail_output_path"),
            Some(&Value::String("/tmp/out/frame.thumb.png".to_string()))
        );
        assert_eq!(object.get("ui_pid"), Some(&Value::Number(123_u32.into())));
        assert!(!object.contains_key("capture"));
        assert!(!object.contains_key("artifacts"));
        assert!(!object.contains_key("actions"));
        assert!(!object.contains_key("bindings"));
    }
}
