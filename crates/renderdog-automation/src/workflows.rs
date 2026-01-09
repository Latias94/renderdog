use std::{ffi::OsString, path::Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    QRenderDocPythonRequest, RenderDocInstallation, default_scripts_dir, write_script_file,
};

#[derive(Debug, Clone)]
pub struct TriggerCaptureRequest {
    pub host: String,
    pub target_ident: u32,
    pub num_frames: u32,
    pub timeout_s: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TriggerCaptureResponse {
    pub capture_path: String,
    pub frame_number: u32,
    pub api: String,
}

#[derive(Debug, Clone)]
pub struct ExportActionsRequest {
    pub capture_path: String,
    pub output_dir: String,
    pub basename: String,
    pub only_drawcalls: bool,
    pub marker_prefix: Option<String>,
    pub event_id_min: Option<u32>,
    pub event_id_max: Option<u32>,
    pub name_contains: Option<String>,
    pub marker_contains: Option<String>,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportActionsResponse {
    pub capture_path: String,
    pub actions_jsonl_path: String,
    pub summary_json_path: String,
    pub total_actions: u64,
    pub drawcall_actions: u64,
}

#[derive(Debug, Error)]
pub enum TriggerCaptureError {
    #[error("failed to create artifacts dir: {0}")]
    CreateArtifactsDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("qrenderdoc python failed: {0}")]
    QRenderDocPython(Box<crate::QRenderDocPythonError>),
    #[error("failed to parse capture JSON: {0}")]
    ParseJson(serde_json::Error),
    #[error("missing capture_path in script output")]
    MissingCapturePath,
}

impl From<crate::QRenderDocPythonError> for TriggerCaptureError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

#[derive(Debug, Error)]
pub enum ExportActionsError {
    #[error("failed to create output dir: {0}")]
    CreateOutputDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("qrenderdoc python failed: {0}")]
    QRenderDocPython(Box<crate::QRenderDocPythonError>),
    #[error("failed to parse export JSON: {0}")]
    ParseJson(serde_json::Error),
}

impl From<crate::QRenderDocPythonError> for ExportActionsError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

impl RenderDocInstallation {
    pub fn trigger_capture_via_target_control(
        &self,
        cwd: &Path,
        req: &TriggerCaptureRequest,
    ) -> Result<TriggerCaptureResponse, TriggerCaptureError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(TriggerCaptureError::CreateArtifactsDir)?;

        let script_path = scripts_dir.join("trigger_capture.py");
        write_script_file(&script_path, TRIGGER_CAPTURE_PY)
            .map_err(TriggerCaptureError::WriteScript)?;

        let args: Vec<OsString> = vec![
            OsString::from("--host"),
            OsString::from(req.host.clone()),
            OsString::from("--ident"),
            OsString::from(req.target_ident.to_string()),
            OsString::from("--frames"),
            OsString::from(req.num_frames.to_string()),
            OsString::from("--timeout-s"),
            OsString::from(req.timeout_s.to_string()),
        ];

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args,
            working_dir: Some(cwd.to_path_buf()),
        })?;

        let line = result
            .stdout
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("");

        let parsed: TriggerCaptureResponse =
            serde_json::from_str(line).map_err(TriggerCaptureError::ParseJson)?;

        if parsed.capture_path.trim().is_empty() {
            return Err(TriggerCaptureError::MissingCapturePath);
        }

        Ok(parsed)
    }

    pub fn export_actions_jsonl(
        &self,
        cwd: &Path,
        req: &ExportActionsRequest,
    ) -> Result<ExportActionsResponse, ExportActionsError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(ExportActionsError::CreateOutputDir)?;

        let script_path = scripts_dir.join("export_actions_jsonl.py");
        write_script_file(&script_path, EXPORT_ACTIONS_JSONL_PY)
            .map_err(ExportActionsError::WriteScript)?;

        let mut args: Vec<OsString> = vec![
            OsString::from("--capture"),
            OsString::from(req.capture_path.clone()),
            OsString::from("--out-dir"),
            OsString::from(req.output_dir.clone()),
            OsString::from("--basename"),
            OsString::from(req.basename.clone()),
        ];
        if req.only_drawcalls {
            args.push(OsString::from("--only-drawcalls"));
        }
        if let Some(prefix) = &req.marker_prefix {
            args.push(OsString::from("--marker-prefix"));
            args.push(OsString::from(prefix.clone()));
        }
        if let Some(v) = req.event_id_min {
            args.push(OsString::from("--event-min"));
            args.push(OsString::from(v.to_string()));
        }
        if let Some(v) = req.event_id_max {
            args.push(OsString::from("--event-max"));
            args.push(OsString::from(v.to_string()));
        }
        if let Some(q) = &req.name_contains {
            args.push(OsString::from("--name-contains"));
            args.push(OsString::from(q.clone()));
        }
        if let Some(q) = &req.marker_contains {
            args.push(OsString::from("--marker-contains"));
            args.push(OsString::from(q.clone()));
        }
        if req.case_sensitive {
            args.push(OsString::from("--case-sensitive"));
        }

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args,
            working_dir: Some(cwd.to_path_buf()),
        })?;

        let line = result
            .stdout
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("");

        serde_json::from_str(line).map_err(ExportActionsError::ParseJson)
    }
}

const TRIGGER_CAPTURE_PY: &str = r#"
import argparse
import json
import time

import renderdoc as rd


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", required=True)
    parser.add_argument("--ident", required=True, type=int)
    parser.add_argument("--frames", required=True, type=int)
    parser.add_argument("--timeout-s", required=True, type=int)
    args = parser.parse_args()

    rd.InitialiseReplay(rd.GlobalEnvironment(), [])

    # Create a target control connection to an already-injected process (started via renderdoccmd capture).
    target = rd.CreateTargetControl(args.host, args.ident, "renderdog", True)
    if target is None:
        raise RuntimeError(f\"CreateTargetControl failed for {args.host}:{args.ident}\")

    try:
        target.TriggerCapture(args.frames)

        # Wait for NewCapture message(s)
        msg = None
        deadline = time.time() + float(args.timeout_s)
        while time.time() < deadline:
            msg = target.ReceiveMessage(None)
            if msg is None:
                continue
            if msg.type == rd.TargetControlMessageType.NewCapture:
                cap = msg.newCapture
                out = {
                    "capture_path": cap.path,
                    "frame_number": int(cap.frameNumber),
                    "api": str(cap.api),
                }
                print(json.dumps(out))
                return 0

        raise RuntimeError("Timed out waiting for NewCapture message")
    finally:
        try:
            target.Shutdown()
        except Exception:
            pass
        rd.ShutdownReplay()


if __name__ == "__main__":
    raise SystemExit(main())
"#;

const EXPORT_ACTIONS_JSONL_PY: &str = r#"
import argparse
import json
import os

import renderdoc as rd


FLAG_NAMES = [
    ("Clear", rd.ActionFlags.Clear),
    ("Drawcall", rd.ActionFlags.Drawcall),
    ("Dispatch", rd.ActionFlags.Dispatch),
    ("MeshDispatch", rd.ActionFlags.MeshDispatch),
    ("CmdList", rd.ActionFlags.CmdList),
    ("SetMarker", rd.ActionFlags.SetMarker),
    ("PushMarker", rd.ActionFlags.PushMarker),
    ("PopMarker", rd.ActionFlags.PopMarker),
    ("Present", rd.ActionFlags.Present),
    ("MultiAction", rd.ActionFlags.MultiAction),
    ("Copy", rd.ActionFlags.Copy),
    ("Resolve", rd.ActionFlags.Resolve),
    ("GenMips", rd.ActionFlags.GenMips),
    ("PassBoundary", rd.ActionFlags.PassBoundary),
    ("DispatchRay", rd.ActionFlags.DispatchRay),
    ("BuildAccStruct", rd.ActionFlags.BuildAccStruct),
    ("Indexed", rd.ActionFlags.Indexed),
    ("Instanced", rd.ActionFlags.Instanced),
    ("Auto", rd.ActionFlags.Auto),
    ("Indirect", rd.ActionFlags.Indirect),
    ("ClearColor", rd.ActionFlags.ClearColor),
    ("ClearDepthStencil", rd.ActionFlags.ClearDepthStencil),
    ("BeginPass", rd.ActionFlags.BeginPass),
    ("EndPass", rd.ActionFlags.EndPass),
    ("CommandBufferBoundary", rd.ActionFlags.CommandBufferBoundary),
]


def flags_to_names(flags):
    names = []
    for name, bit in FLAG_NAMES:
        if flags & bit:
            names.append(name)
    return names


def is_drawcall_like(flags: int) -> bool:
    return bool(
        (flags & rd.ActionFlags.Drawcall)
        or (flags & rd.ActionFlags.Dispatch)
        or (flags & rd.ActionFlags.MeshDispatch)
        or (flags & rd.ActionFlags.DispatchRay)
    )


def marker_path_join(marker_path) -> str:
    if not marker_path:
        return ""
    return "/".join([str(x) for x in marker_path])

def normalize(s: str, case_sensitive: bool) -> str:
    if s is None:
        return ""
    if case_sensitive:
        return str(s)
    return str(s).lower()


def iter_actions(structured_file, actions, marker_stack, parent_event_id, depth, out_fp, counters,
                 only_drawcalls: bool, marker_prefix: str,
                 event_min, event_max,
                 name_contains: str, marker_contains: str,
                 case_sensitive: bool):
    for a in actions:
        name = a.GetName(structured_file)
        flags = a.flags

        effective_marker_path = list(marker_stack)
        if flags & rd.ActionFlags.PushMarker:
            effective_marker_path.append(str(name))

        joined_marker_path = marker_path_join(effective_marker_path)
        name_str = str(name)

        def recurse():
            if flags & rd.ActionFlags.PushMarker:
                marker_stack.append(str(name))
                iter_actions(structured_file, a.children, marker_stack, a.eventId, depth + 1, out_fp, counters,
                             only_drawcalls, marker_prefix,
                             event_min, event_max,
                             name_contains, marker_contains,
                             case_sensitive)
                marker_stack.pop()
            else:
                iter_actions(structured_file, a.children, marker_stack, a.eventId, depth + 1, out_fp, counters,
                             only_drawcalls, marker_prefix,
                             event_min, event_max,
                             name_contains, marker_contains,
                             case_sensitive)

        if marker_prefix:
            if not (joined_marker_path == marker_prefix or joined_marker_path.startswith(marker_prefix + "/")):
                recurse()
                continue

        eid = int(a.eventId)

        should_emit = True
        if only_drawcalls and not is_drawcall_like(flags):
            should_emit = False
        if event_min is not None and eid < int(event_min):
            should_emit = False
        if event_max is not None and eid > int(event_max):
            should_emit = False

        if name_contains:
            if name_contains not in normalize(name_str, case_sensitive):
                should_emit = False
        if marker_contains:
            if marker_contains not in normalize(joined_marker_path, case_sensitive):
                should_emit = False

        if should_emit:
            rec = {
                "event_id": eid,
            "parent_event_id": int(parent_event_id) if parent_event_id is not None else None,
            "depth": int(depth),
            "name": name_str,
            "flags": int(flags),
            "flags_names": flags_to_names(flags),
            "marker_path": effective_marker_path,
            "num_children": int(len(a.children)),
            }

            out_fp.write(json.dumps(rec, ensure_ascii=False) + "\n")

            counters["total_actions"] += 1
            if is_drawcall_like(flags):
                counters["drawcall_actions"] += 1

        recurse()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--capture", required=True)
    parser.add_argument("--out-dir", required=True)
    parser.add_argument("--basename", required=True)
    parser.add_argument("--only-drawcalls", action="store_true")
    parser.add_argument("--marker-prefix", default="")
    parser.add_argument("--event-min", type=int, default=None)
    parser.add_argument("--event-max", type=int, default=None)
    parser.add_argument("--name-contains", default="")
    parser.add_argument("--marker-contains", default="")
    parser.add_argument("--case-sensitive", action="store_true")
    args = parser.parse_args()

    rd.InitialiseReplay(rd.GlobalEnvironment(), [])

    os.makedirs(args.out_dir, exist_ok=True)

    actions_path = os.path.join(args.out_dir, f"{args.basename}.actions.jsonl")
    summary_path = os.path.join(args.out_dir, f"{args.basename}.summary.json")

    cap = rd.OpenCaptureFile()
    try:
        result = cap.OpenFile(args.capture, "", None)
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("Couldn't open file: " + str(result))

        if not cap.LocalReplaySupport():
            raise RuntimeError("Capture cannot be replayed")

        result, controller = cap.OpenCapture(rd.ReplayOptions(), None)
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("Couldn't initialise replay: " + str(result))

        try:
            structured_file = controller.GetStructuredFile()
            roots = controller.GetRootActions()

            counters = {"total_actions": 0, "drawcall_actions": 0}
            with open(actions_path, "w", encoding="utf-8") as fp:
                iter_actions(structured_file, roots, [], None, 0, fp, counters,
                             args.only_drawcalls, args.marker_prefix,
                             args.event_min, args.event_max,
                             normalize(args.name_contains, args.case_sensitive),
                             normalize(args.marker_contains, args.case_sensitive),
                             args.case_sensitive)

            api = str(controller.GetAPIProperties().pipelineType)

            summary = {
                "capture_path": args.capture,
                "api": api,
                "total_actions": int(counters["total_actions"]),
                "drawcall_actions": int(counters["drawcall_actions"]),
                "actions_jsonl_path": actions_path,
            }

            with open(summary_path, "w", encoding="utf-8") as fp:
                json.dump(summary, fp, ensure_ascii=False, indent=2)

            print(json.dumps({
                "capture_path": args.capture,
                "actions_jsonl_path": actions_path,
                "summary_json_path": summary_path,
                "total_actions": int(counters["total_actions"]),
                "drawcall_actions": int(counters["drawcall_actions"]),
            }, ensure_ascii=False))
            return 0
        finally:
            try:
                controller.Shutdown()
            except Exception:
                pass
    finally:
        try:
            cap.Shutdown()
        except Exception:
            pass
        rd.ShutdownReplay()


if __name__ == "__main__":
    raise SystemExit(main())
"#;
