import json
import os
import traceback

import renderdoc as rd

from renderdog_action_query import ActionFilter, walk_actions


REQ_PATH = "export_bindings_index_jsonl.request.json"
RESP_PATH = "export_bindings_index_jsonl.response.json"


def write_envelope(ok: bool, result=None, error: str = None) -> None:
    with open(RESP_PATH, "w", encoding="utf-8") as f:
        json.dump({"ok": ok, "result": result, "error": error}, f, ensure_ascii=False)


def try_res_name(controller, rid) -> str:
    try:
        desc = controller.GetResourceDescription(rid)
        if desc is None:
            return ""
        return str(desc.name or "")
    except Exception:
        return ""


def stage_name(stage) -> str:
    try:
        return str(stage)
    except Exception:
        return "Unknown"


def build_reflection_name_map(reflection, access: str):
    m = {}
    if reflection is None:
        return m
    try:
        if access == "ro":
            for res in reflection.readOnlyResources:
                m[int(res.fixedBindNumber)] = str(res.name)
        elif access == "rw":
            for res in reflection.readWriteResources:
                m[int(res.fixedBindNumber)] = str(res.name)
    except Exception:
        pass
    return m


def serialize_bindings_for_stage(controller, pipe, stage, include_cbuffers: bool):
    shader = pipe.GetShader(stage)
    if shader == rd.ResourceId.Null():
        return None

    info = {
        "shader": {
            "resource_id": str(shader),
            "name": try_res_name(controller, shader),
            "entry_point": str(pipe.GetShaderEntryPoint(stage) or ""),
        },
        "srvs": [],
        "uavs": [],
        "cbuffers": [],
    }

    reflection = None
    try:
        reflection = pipe.GetShaderReflection(stage)
    except Exception:
        reflection = None

    ro_name_map = build_reflection_name_map(reflection, "ro")
    rw_name_map = build_reflection_name_map(reflection, "rw")

    # SRVs
    try:
        srvs = pipe.GetReadOnlyResources(stage, False)
        for srv in srvs:
            rid = srv.descriptor.resource
            if rid == rd.ResourceId.Null():
                continue
            slot = int(srv.access.index)
            info["srvs"].append(
                {
                    "slot": slot,
                    "name": ro_name_map.get(slot, ""),
                    "resource_id": str(rid),
                    "resource_name": try_res_name(controller, rid),
                }
            )
    except Exception:
        pass

    # UAVs
    try:
        uavs = pipe.GetReadWriteResources(stage, False)
        for uav in uavs:
            rid = uav.descriptor.resource
            if rid == rd.ResourceId.Null():
                continue
            slot = int(uav.access.index)
            info["uavs"].append(
                {
                    "slot": slot,
                    "name": rw_name_map.get(slot, ""),
                    "resource_id": str(rid),
                    "resource_name": try_res_name(controller, rid),
                }
            )
    except Exception:
        pass

    # Constant buffers (metadata only; no variable dumping)
    if include_cbuffers and reflection is not None:
        try:
            for i, cb in enumerate(reflection.constantBlocks):
                entry = {
                    "slot": int(i),
                    "name": str(cb.name),
                    "size": int(cb.byteSize),
                    "resource_id": None,
                    "resource_name": "",
                }
                try:
                    bind = pipe.GetConstantBuffer(stage, i, 0)
                    if bind.resourceId != rd.ResourceId.Null():
                        entry["resource_id"] = str(bind.resourceId)
                        entry["resource_name"] = try_res_name(controller, bind.resourceId)
                except Exception:
                    pass
                info["cbuffers"].append(entry)
        except Exception:
            pass

    return info


def serialize_outputs(controller, pipe):
    out = {"render_targets": [], "depth_target": None}
    try:
        om = pipe.GetOutputMerger()
        if om is None:
            return out

        rts = []
        for i, rt in enumerate(om.renderTargets):
            rid = rt.resourceId
            if rid == rd.ResourceId.Null():
                continue
            rts.append(
                {
                    "index": int(i),
                    "resource_id": str(rid),
                    "resource_name": try_res_name(controller, rid),
                }
            )
        out["render_targets"] = rts

        dt = om.depthTarget.resourceId
        if dt != rd.ResourceId.Null():
            out["depth_target"] = {
                "resource_id": str(dt),
                "resource_name": try_res_name(controller, dt),
            }
    except Exception:
        pass
    return out


def main() -> None:
    with open(REQ_PATH, "r", encoding="utf-8") as f:
        req = json.load(f)

    rd.InitialiseReplay(rd.GlobalEnvironment(), [])

    os.makedirs(req["output_dir"], exist_ok=True)

    bindings_path = os.path.join(req["output_dir"], f"{req['basename']}.bindings.jsonl")
    summary_path = os.path.join(req["output_dir"], f"{req['basename']}.bindings_summary.json")

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
            structured_file = controller.GetStructuredFile()
            roots = controller.GetRootActions()

            counters = {"total_drawcalls": 0}
            action_filter = ActionFilter(
                only_drawcalls=True,
                marker_prefix=str(req.get("marker_prefix") or ""),
                event_min=req.get("event_id_min", None),
                event_max=req.get("event_id_max", None),
                name_contains=req.get("name_contains") or "",
                marker_contains=req.get("marker_contains") or "",
                case_sensitive=bool(req.get("case_sensitive", False)),
            )

            with open(bindings_path, "w", encoding="utf-8") as fp:
                include_cbuffers = bool(req.get("include_cbuffers", False))
                include_outputs = bool(req.get("include_outputs", False))

                def handle_action(action) -> None:
                    controller.SetFrameEvent(action.event_id, False)
                    pipe = controller.GetPipelineState()

                    stages = [
                        rd.ShaderStage.Vertex,
                        rd.ShaderStage.Hull,
                        rd.ShaderStage.Domain,
                        rd.ShaderStage.Geometry,
                        rd.ShaderStage.Pixel,
                        rd.ShaderStage.Compute,
                    ]

                    stage_map = {}
                    shader_names = []
                    resource_names = []

                    for st in stages:
                        st_info = serialize_bindings_for_stage(
                            controller,
                            pipe,
                            st,
                            include_cbuffers,
                        )
                        if st_info is None:
                            continue

                        st_key = stage_name(st)
                        stage_map[st_key] = st_info

                        sh = st_info.get("shader") or {}
                        if sh.get("name"):
                            shader_names.append(sh.get("name"))
                        if sh.get("entry_point"):
                            shader_names.append(sh.get("entry_point"))

                        for srv in st_info.get("srvs") or []:
                            if srv.get("name"):
                                resource_names.append(srv.get("name"))
                            if srv.get("resource_name"):
                                resource_names.append(srv.get("resource_name"))
                        for uav in st_info.get("uavs") or []:
                            if uav.get("name"):
                                resource_names.append(uav.get("name"))
                            if uav.get("resource_name"):
                                resource_names.append(uav.get("resource_name"))
                        for cb in st_info.get("cbuffers") or []:
                            if cb.get("name"):
                                resource_names.append(cb.get("name"))
                            if cb.get("resource_name"):
                                resource_names.append(cb.get("resource_name"))

                    rec = {
                        "event_id": action.event_id,
                        "depth": action.depth,
                        "name": action.name,
                        "marker_path": action.marker_path,
                        "stages": stage_map,
                        "shader_names": shader_names,
                        "resource_names": resource_names,
                    }

                    if include_outputs:
                        rec["outputs"] = serialize_outputs(controller, pipe)

                    fp.write(json.dumps(rec, ensure_ascii=False) + "\n")
                    counters["total_drawcalls"] += 1

                walk_actions(structured_file, roots, action_filter, handle_action)

            api = str(controller.GetAPIProperties().pipelineType)

            summary = {
                "capture_path": req["capture_path"],
                "api": api,
                "total_drawcalls": int(counters["total_drawcalls"]),
                "bindings_jsonl_path": bindings_path,
            }

            with open(summary_path, "w", encoding="utf-8") as fp:
                json.dump(summary, fp, ensure_ascii=False, indent=2)

            write_envelope(
                True,
                result={
                    "bindings_jsonl_path": bindings_path,
                    "bindings_summary_json_path": summary_path,
                    "total_drawcalls": int(counters["total_drawcalls"]),
                },
            )
            return
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
        write_envelope(False, error=traceback.format_exc())
    raise SystemExit(0)
