use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::resolve_path_string_from_cwd;
use crate::scripting::{QRenderDocJsonEnvelope, create_qrenderdoc_run_dir};
use crate::{
    QRenderDocPythonRequest, RenderDocInstallation, default_scripts_dir, write_script_file,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayListTexturesRequest {
    pub capture_path: String,
    pub event_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayTextureInfo {
    pub index: u32,
    pub resource_id: u64,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mips: u32,
    pub arraysize: u32,
    pub ms_samp: u32,
    pub byte_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayListTexturesResponse {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub textures: Vec<ReplayTextureInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayPickPixelRequest {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayPickPixelResponse {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    pub x: u32,
    pub y: u32,
    pub rgba: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveTexturePngRequest {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveTexturePngResponse {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveOutputsPngRequest {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub output_dir: String,
    pub basename: String,
    pub include_depth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySavedImage {
    pub kind: String,
    pub index: Option<u32>,
    pub resource_id: u64,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveOutputsPngResponse {
    pub capture_path: String,
    pub event_id: u32,
    pub outputs: Vec<ReplaySavedImage>,
}

#[derive(Debug, Error)]
pub enum ReplayListTexturesError {
    #[error("failed to create scripts dir: {0}")]
    CreateScriptsDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("failed to write request JSON: {0}")]
    WriteRequest(std::io::Error),
    #[error("qrenderdoc python failed: {0}")]
    QRenderDocPython(Box<crate::QRenderDocPythonError>),
    #[error("failed to read response JSON: {0}")]
    ReadResponse(std::io::Error),
    #[error("failed to parse JSON: {0}")]
    ParseJson(serde_json::Error),
    #[error("qrenderdoc script error: {0}")]
    ScriptError(String),
}

impl From<crate::QRenderDocPythonError> for ReplayListTexturesError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

#[derive(Debug, Error)]
pub enum ReplayPickPixelError {
    #[error("failed to create scripts dir: {0}")]
    CreateScriptsDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("failed to write request JSON: {0}")]
    WriteRequest(std::io::Error),
    #[error("qrenderdoc python failed: {0}")]
    QRenderDocPython(Box<crate::QRenderDocPythonError>),
    #[error("failed to read response JSON: {0}")]
    ReadResponse(std::io::Error),
    #[error("failed to parse JSON: {0}")]
    ParseJson(serde_json::Error),
    #[error("qrenderdoc script error: {0}")]
    ScriptError(String),
}

impl From<crate::QRenderDocPythonError> for ReplayPickPixelError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

#[derive(Debug, Error)]
pub enum ReplaySaveTexturePngError {
    #[error("failed to create scripts dir: {0}")]
    CreateScriptsDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("failed to write request JSON: {0}")]
    WriteRequest(std::io::Error),
    #[error("qrenderdoc python failed: {0}")]
    QRenderDocPython(Box<crate::QRenderDocPythonError>),
    #[error("failed to read response JSON: {0}")]
    ReadResponse(std::io::Error),
    #[error("failed to parse JSON: {0}")]
    ParseJson(serde_json::Error),
    #[error("qrenderdoc script error: {0}")]
    ScriptError(String),
}

impl From<crate::QRenderDocPythonError> for ReplaySaveTexturePngError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

#[derive(Debug, Error)]
pub enum ReplaySaveOutputsPngError {
    #[error("failed to create scripts dir: {0}")]
    CreateScriptsDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("failed to write request JSON: {0}")]
    WriteRequest(std::io::Error),
    #[error("qrenderdoc python failed: {0}")]
    QRenderDocPython(Box<crate::QRenderDocPythonError>),
    #[error("failed to read response JSON: {0}")]
    ReadResponse(std::io::Error),
    #[error("failed to parse JSON: {0}")]
    ParseJson(serde_json::Error),
    #[error("qrenderdoc script error: {0}")]
    ScriptError(String),
}

impl From<crate::QRenderDocPythonError> for ReplaySaveOutputsPngError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

fn remove_if_exists(path: &Path) -> Result<(), std::io::Error> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

impl RenderDocInstallation {
    pub fn replay_list_textures(
        &self,
        cwd: &Path,
        req: &ReplayListTexturesRequest,
    ) -> Result<ReplayListTexturesResponse, ReplayListTexturesError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(ReplayListTexturesError::CreateScriptsDir)?;

        let script_path = scripts_dir.join("replay_list_textures_json.py");
        write_script_file(&script_path, REPLAY_LIST_TEXTURES_JSON_PY)
            .map_err(ReplayListTexturesError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "replay_list_textures")
            .map_err(ReplayListTexturesError::CreateScriptsDir)?;
        let request_path = run_dir.join("replay_list_textures_json.request.json");
        let response_path = run_dir.join("replay_list_textures_json.response.json");
        remove_if_exists(&response_path).map_err(ReplayListTexturesError::WriteRequest)?;

        let req = ReplayListTexturesRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            ..req.clone()
        };
        std::fs::write(
            &request_path,
            serde_json::to_vec(&req).map_err(ReplayListTexturesError::ParseJson)?,
        )
        .map_err(ReplayListTexturesError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;

        let _ = result;
        let bytes = std::fs::read(&response_path).map_err(ReplayListTexturesError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<ReplayListTexturesResponse> =
            serde_json::from_slice(&bytes).map_err(ReplayListTexturesError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| ReplayListTexturesError::ScriptError("missing result".into()))
        } else {
            Err(ReplayListTexturesError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }

    pub fn replay_pick_pixel(
        &self,
        cwd: &Path,
        req: &ReplayPickPixelRequest,
    ) -> Result<ReplayPickPixelResponse, ReplayPickPixelError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(ReplayPickPixelError::CreateScriptsDir)?;

        let script_path = scripts_dir.join("replay_pick_pixel_json.py");
        write_script_file(&script_path, REPLAY_PICK_PIXEL_JSON_PY)
            .map_err(ReplayPickPixelError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "replay_pick_pixel")
            .map_err(ReplayPickPixelError::CreateScriptsDir)?;
        let request_path = run_dir.join("replay_pick_pixel_json.request.json");
        let response_path = run_dir.join("replay_pick_pixel_json.response.json");
        remove_if_exists(&response_path).map_err(ReplayPickPixelError::WriteRequest)?;

        let req = ReplayPickPixelRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            ..req.clone()
        };
        std::fs::write(
            &request_path,
            serde_json::to_vec(&req).map_err(ReplayPickPixelError::ParseJson)?,
        )
        .map_err(ReplayPickPixelError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;

        let _ = result;
        let bytes = std::fs::read(&response_path).map_err(ReplayPickPixelError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<ReplayPickPixelResponse> =
            serde_json::from_slice(&bytes).map_err(ReplayPickPixelError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| ReplayPickPixelError::ScriptError("missing result".into()))
        } else {
            Err(ReplayPickPixelError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }

    pub fn replay_save_texture_png(
        &self,
        cwd: &Path,
        req: &ReplaySaveTexturePngRequest,
    ) -> Result<ReplaySaveTexturePngResponse, ReplaySaveTexturePngError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir)
            .map_err(ReplaySaveTexturePngError::CreateScriptsDir)?;

        let script_path = scripts_dir.join("replay_save_texture_png_json.py");
        write_script_file(&script_path, REPLAY_SAVE_TEXTURE_PNG_JSON_PY)
            .map_err(ReplaySaveTexturePngError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "replay_save_texture_png")
            .map_err(ReplaySaveTexturePngError::CreateScriptsDir)?;
        let request_path = run_dir.join("replay_save_texture_png_json.request.json");
        let response_path = run_dir.join("replay_save_texture_png_json.response.json");
        remove_if_exists(&response_path).map_err(ReplaySaveTexturePngError::WriteRequest)?;

        let req = ReplaySaveTexturePngRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            output_path: resolve_path_string_from_cwd(cwd, &req.output_path),
            ..req.clone()
        };
        std::fs::write(
            &request_path,
            serde_json::to_vec(&req).map_err(ReplaySaveTexturePngError::ParseJson)?,
        )
        .map_err(ReplaySaveTexturePngError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;

        let _ = result;
        let bytes =
            std::fs::read(&response_path).map_err(ReplaySaveTexturePngError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<ReplaySaveTexturePngResponse> =
            serde_json::from_slice(&bytes).map_err(ReplaySaveTexturePngError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| ReplaySaveTexturePngError::ScriptError("missing result".into()))
        } else {
            Err(ReplaySaveTexturePngError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }

    pub fn replay_save_outputs_png(
        &self,
        cwd: &Path,
        req: &ReplaySaveOutputsPngRequest,
    ) -> Result<ReplaySaveOutputsPngResponse, ReplaySaveOutputsPngError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir)
            .map_err(ReplaySaveOutputsPngError::CreateScriptsDir)?;

        let script_path = scripts_dir.join("replay_save_outputs_png_json.py");
        write_script_file(&script_path, REPLAY_SAVE_OUTPUTS_PNG_JSON_PY)
            .map_err(ReplaySaveOutputsPngError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "replay_save_outputs_png")
            .map_err(ReplaySaveOutputsPngError::CreateScriptsDir)?;
        let request_path = run_dir.join("replay_save_outputs_png_json.request.json");
        let response_path = run_dir.join("replay_save_outputs_png_json.response.json");
        remove_if_exists(&response_path).map_err(ReplaySaveOutputsPngError::WriteRequest)?;

        let req = ReplaySaveOutputsPngRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            output_dir: resolve_path_string_from_cwd(cwd, &req.output_dir),
            ..req.clone()
        };
        std::fs::write(
            &request_path,
            serde_json::to_vec(&req).map_err(ReplaySaveOutputsPngError::ParseJson)?,
        )
        .map_err(ReplaySaveOutputsPngError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;

        let _ = result;
        let bytes =
            std::fs::read(&response_path).map_err(ReplaySaveOutputsPngError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<ReplaySaveOutputsPngResponse> =
            serde_json::from_slice(&bytes).map_err(ReplaySaveOutputsPngError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| ReplaySaveOutputsPngError::ScriptError("missing result".into()))
        } else {
            Err(ReplaySaveOutputsPngError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }
}

const REPLAY_LIST_TEXTURES_JSON_PY: &str = r#"
import json
import os
import traceback

import renderdoc as rd


REQ_PATH = "replay_list_textures_json.request.json"
RESP_PATH = "replay_list_textures_json.response.json"


def write_response(obj) -> None:
    with open(RESP_PATH, "w", encoding="utf-8") as f:
        json.dump(obj, f, ensure_ascii=False)


def main() -> None:
    with open(REQ_PATH, "r", encoding="utf-8") as f:
        req = json.load(f)

    rd.InitialiseReplay(rd.GlobalEnvironment(), [])

    cap = rd.OpenCaptureFile()
    try:
        result = cap.OpenFile(req["capture_path"], "", None)
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("Couldn't open file: " + str(result))

        if not cap.LocalReplaySupport():
            raise RuntimeError("Capture cannot be replayed")

        result, controller = cap.OpenCapture(rd.ReplayOptions(), None)
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("Couldn't initialise replay: " + str(result))

        try:
            event_id = req.get("event_id", None)
            if event_id is not None:
                controller.SetFrameEvent(int(event_id), True)

            name_by_id = {}
            try:
                for r in controller.GetResources():
                    rrid = int(r.resourceId)
                    n = getattr(r, "name", None)
                    if n is None:
                        n = getattr(r, "resourceName", None)
                    if n is None:
                        continue
                    name_by_id[rrid] = str(n or "")
            except Exception:
                name_by_id = {}

            textures = controller.GetTextures()
            out = []
            for i, t in enumerate(textures):
                rid = t.resourceId
                name = name_by_id.get(int(rid), "") or ""
                try:
                    desc = controller.GetResourceDescription(rid)
                    if desc is not None:
                        if not name:
                            name = str(desc.name or "")
                except Exception:
                    pass

                arraysize = getattr(t, "arraysize", getattr(t, "arraySize", 1))
                ms_samp = getattr(t, "msSamp", getattr(t, "msSamples", 1))
                byte_size = getattr(t, "byteSize", getattr(t, "bytesize", getattr(t, "byte_size", 0)))
                out.append(
                    {
                        "index": int(i),
                        "resource_id": int(rid),
                        "name": name,
                        "width": int(t.width),
                        "height": int(t.height),
                        "depth": int(t.depth),
                        "mips": int(t.mips),
                        "arraysize": int(arraysize),
                        "ms_samp": int(ms_samp),
                        "byte_size": int(byte_size),
                    }
                )

            write_response(
                {
                    "capture_path": req["capture_path"],
                    "event_id": event_id,
                    "textures": out,
                }
            )
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
    try:
        main()
    except Exception:
        write_response({"ok": False, "error": traceback.format_exc()})
    else:
        # Wrap successful output written by main().
        with open(RESP_PATH, "r", encoding="utf-8") as f:
            payload = json.load(f)
        write_response({"ok": True, "result": payload})
    raise SystemExit(0)
"#;

const REPLAY_PICK_PIXEL_JSON_PY: &str = r#"
import json
import os
import traceback

import renderdoc as rd


REQ_PATH = "replay_pick_pixel_json.request.json"
RESP_PATH = "replay_pick_pixel_json.response.json"


def write_response(obj) -> None:
    with open(RESP_PATH, "w", encoding="utf-8") as f:
        json.dump(obj, f, ensure_ascii=False)


def main() -> None:
    with open(REQ_PATH, "r", encoding="utf-8") as f:
        req = json.load(f)

    rd.InitialiseReplay(rd.GlobalEnvironment(), [])

    cap = rd.OpenCaptureFile()
    try:
        result = cap.OpenFile(req["capture_path"], "", None)
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("Couldn't open file: " + str(result))

        if not cap.LocalReplaySupport():
            raise RuntimeError("Capture cannot be replayed")

        result, controller = cap.OpenCapture(rd.ReplayOptions(), None)
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("Couldn't initialise replay: " + str(result))

        try:
            event_id = req.get("event_id", None)
            if event_id is not None:
                controller.SetFrameEvent(int(event_id), True)

            textures = controller.GetTextures()
            idx = int(req["texture_index"])
            if idx < 0 or idx >= len(textures):
                raise RuntimeError("texture_index out of range")

            t = textures[idx]
            pv = controller.PickPixel(
                t.resourceId,
                int(req["x"]),
                int(req["y"]),
                rd.Subresource(0, 0, 0),
                rd.CompType.Typeless,
            )

            rgba = [
                float(pv.floatValue[0]),
                float(pv.floatValue[1]),
                float(pv.floatValue[2]),
                float(pv.floatValue[3]),
            ]

            write_response(
                {
                    "capture_path": req["capture_path"],
                    "event_id": event_id,
                    "texture_index": int(req["texture_index"]),
                    "x": int(req["x"]),
                    "y": int(req["y"]),
                    "rgba": rgba,
                }
            )
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
    try:
        main()
    except Exception:
        write_response({"ok": False, "error": traceback.format_exc()})
    else:
        with open(RESP_PATH, "r", encoding="utf-8") as f:
            payload = json.load(f)
        write_response({"ok": True, "result": payload})
    raise SystemExit(0)
"#;

const REPLAY_SAVE_TEXTURE_PNG_JSON_PY: &str = r#"
import json
import os
import traceback

import renderdoc as rd


REQ_PATH = "replay_save_texture_png_json.request.json"
RESP_PATH = "replay_save_texture_png_json.response.json"


def write_response(obj) -> None:
    with open(RESP_PATH, "w", encoding="utf-8") as f:
        json.dump(obj, f, ensure_ascii=False)


def main() -> None:
    with open(REQ_PATH, "r", encoding="utf-8") as f:
        req = json.load(f)

    out_dir = os.path.dirname(req["output_path"])
    if out_dir:
        os.makedirs(out_dir, exist_ok=True)

    rd.InitialiseReplay(rd.GlobalEnvironment(), [])

    cap = rd.OpenCaptureFile()
    try:
        result = cap.OpenFile(req["capture_path"], "", None)
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("Couldn't open file: " + str(result))

        if not cap.LocalReplaySupport():
            raise RuntimeError("Capture cannot be replayed")

        result, controller = cap.OpenCapture(rd.ReplayOptions(), None)
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("Couldn't initialise replay: " + str(result))

        try:
            event_id = req.get("event_id", None)
            if event_id is not None:
                controller.SetFrameEvent(int(event_id), True)

            textures = controller.GetTextures()
            idx = int(req["texture_index"])
            if idx < 0 or idx >= len(textures):
                raise RuntimeError("texture_index out of range")

            t = textures[idx]

            save = rd.TextureSave()
            save.resourceId = t.resourceId
            save.destType = rd.FileType.PNG
            save.mip = 0

            result = controller.SaveTexture(save, str(req["output_path"]))
            if result != rd.ResultCode.Succeeded:
                raise RuntimeError("SaveTexture failed: " + str(result))

            write_response(
                {
                    "capture_path": req["capture_path"],
                    "event_id": event_id,
                    "texture_index": int(req["texture_index"]),
                    "output_path": str(req["output_path"]),
                }
            )
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
    try:
        main()
    except Exception:
        write_response({"ok": False, "error": traceback.format_exc()})
    else:
        with open(RESP_PATH, "r", encoding="utf-8") as f:
            payload = json.load(f)
        write_response({"ok": True, "result": payload})
    raise SystemExit(0)
"#;

const REPLAY_SAVE_OUTPUTS_PNG_JSON_PY: &str = r#"
import json
import os
import traceback

import renderdoc as rd


REQ_PATH = "replay_save_outputs_png_json.request.json"
RESP_PATH = "replay_save_outputs_png_json.response.json"


def write_response(obj) -> None:
    with open(RESP_PATH, "w", encoding="utf-8") as f:
        json.dump(obj, f, ensure_ascii=False)


def flatten_actions(actions):
    out = []
    for a in actions:
        out.append(a)
        out.extend(flatten_actions(a.children))
    return out


def pick_default_event_id(controller) -> int:
    actions = flatten_actions(controller.GetRootActions())
    if not actions:
        return 0
    return int(max(a.eventId for a in actions))


def extract_resource_id(obj):
    if obj is None:
        return None
    if hasattr(obj, "resourceId"):
        return obj.resourceId
    if hasattr(obj, "resource"):
        return obj.resource
    return None


def is_null_resource_id(rid) -> bool:
    try:
        if rid == rd.ResourceId():
            return True
    except Exception:
        pass

    try:
        return int(rid) == 0
    except Exception:
        try:
            return int(rid.value) == 0
        except Exception:
            return False


def set_save_params_from_bound_resource(save, br):
    if hasattr(br, "firstMip"):
        try:
            save.mip = int(br.firstMip)
        except Exception:
            pass

    if hasattr(br, "firstSlice"):
        try:
            save.slice = int(br.firstSlice)
        except Exception:
            pass

    if hasattr(save, "sampleIdx"):
        try:
            save.sampleIdx = 0
        except Exception:
            pass


def main() -> None:
    with open(REQ_PATH, "r", encoding="utf-8") as f:
        req = json.load(f)

    os.makedirs(req["output_dir"], exist_ok=True)

    rd.InitialiseReplay(rd.GlobalEnvironment(), [])

    cap = rd.OpenCaptureFile()
    try:
        result = cap.OpenFile(req["capture_path"], "", None)
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("Couldn't open file: " + str(result))

        if not cap.LocalReplaySupport():
            raise RuntimeError("Capture cannot be replayed")

        result, controller = cap.OpenCapture(rd.ReplayOptions(), None)
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("Couldn't initialise replay: " + str(result))

        try:
            event_id = req.get("event_id", None)
            if event_id is None:
                event_id = pick_default_event_id(controller)

            controller.SetFrameEvent(int(event_id), True)

            pipe = controller.GetPipelineState()
            outputs = []

            for i, br in enumerate(pipe.GetOutputTargets()):
                rid = extract_resource_id(br)
                if rid is None or is_null_resource_id(rid):
                    continue

                out_path = os.path.join(
                    req["output_dir"], f"{req['basename']}.event{int(event_id)}.rt{i}.png"
                )

                save = rd.TextureSave()
                save.resourceId = rid
                save.destType = rd.FileType.PNG
                save.mip = 0
                set_save_params_from_bound_resource(save, br)

                result = controller.SaveTexture(save, out_path)
                if result != rd.ResultCode.Succeeded:
                    raise RuntimeError("SaveTexture failed: " + str(result))

                outputs.append(
                    {
                        "kind": "color",
                        "index": int(i),
                        "resource_id": int(rid),
                        "output_path": out_path,
                    }
                )

            if bool(req.get("include_depth", False)):
                br = pipe.GetDepthTarget()
                rid = extract_resource_id(br)
                if rid is not None and not is_null_resource_id(rid):
                    out_path = os.path.join(
                        req["output_dir"], f"{req['basename']}.event{int(event_id)}.depth.png"
                    )

                    save = rd.TextureSave()
                    save.resourceId = rid
                    save.destType = rd.FileType.PNG
                    save.mip = 0
                    set_save_params_from_bound_resource(save, br)

                    result = controller.SaveTexture(save, out_path)
                    if result != rd.ResultCode.Succeeded:
                        raise RuntimeError("SaveTexture(depth) failed: " + str(result))

                    outputs.append(
                        {
                            "kind": "depth",
                            "index": None,
                            "resource_id": int(rid),
                            "output_path": out_path,
                        }
                    )

            write_response(
                {
                    "capture_path": req["capture_path"],
                    "event_id": int(event_id),
                    "outputs": outputs,
                }
            )
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
    try:
        main()
    except Exception:
        write_response({"ok": False, "error": traceback.format_exc()})
    else:
        with open(RESP_PATH, "r", encoding="utf-8") as f:
            payload = json.load(f)
        write_response({"ok": True, "result": payload})
    raise SystemExit(0)
"#;
