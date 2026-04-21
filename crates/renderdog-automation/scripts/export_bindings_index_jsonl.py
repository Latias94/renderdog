import json
import os
import traceback

import renderdoc as rd


REQ_PATH = "export_bindings_index_jsonl.request.json"
RESP_PATH = "export_bindings_index_jsonl.response.json"


def write_envelope(ok: bool, result=None, error: str = None) -> None:
    with open(RESP_PATH, "w", encoding="utf-8") as f:
        json.dump({"ok": ok, "result": result, "error": error}, f, ensure_ascii=False)


def normalize(s: str, case_sensitive: bool) -> str:
    if s is None:
        return ""
    if case_sensitive:
        return str(s)
    return str(s).lower()


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


def iter_actions(structured_file, controller, actions, marker_stack, depth,
                 out_fp, counters,
                 marker_prefix: str,
                 event_min, event_max,
                 name_contains: str, marker_contains: str,
                 case_sensitive: bool,
                 include_cbuffers: bool,
                 include_outputs: bool):
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
                iter_actions(structured_file, controller, a.children, marker_stack, depth + 1,
                             out_fp, counters,
                             marker_prefix,
                             event_min, event_max,
                             name_contains, marker_contains,
                             case_sensitive,
                             include_cbuffers, include_outputs)
                marker_stack.pop()
            else:
                iter_actions(structured_file, controller, a.children, marker_stack, depth + 1,
                             out_fp, counters,
                             marker_prefix,
                             event_min, event_max,
                             name_contains, marker_contains,
                             case_sensitive,
                             include_cbuffers, include_outputs)

        if marker_prefix:
            if not (joined_marker_path == marker_prefix or joined_marker_path.startswith(marker_prefix + "/")):
                recurse()
                continue

        eid = int(a.eventId)

        should_emit = is_drawcall_like(flags)
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
            controller.SetFrameEvent(eid, False)
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
                st_info = serialize_bindings_for_stage(controller, pipe, st, include_cbuffers)
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
                "event_id": eid,
                "depth": int(depth),
                "name": name_str,
                "marker_path": effective_marker_path,
                "stages": stage_map,
                "shader_names": shader_names,
                "resource_names": resource_names,
            }

            if include_outputs:
                rec["outputs"] = serialize_outputs(controller, pipe)

            out_fp.write(json.dumps(rec, ensure_ascii=False) + "\n")
            counters["total_drawcalls"] += 1

        recurse()


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
            with open(bindings_path, "w", encoding="utf-8") as fp:
                iter_actions(
                    structured_file,
                    controller,
                    roots,
                    [],
                    0,
                    fp,
                    counters,
                    str(req.get("marker_prefix") or ""),
                    req.get("event_id_min", None),
                    req.get("event_id_max", None),
                    normalize(req.get("name_contains") or "", bool(req.get("case_sensitive", False))),
                    normalize(req.get("marker_contains") or "", bool(req.get("case_sensitive", False))),
                    bool(req.get("case_sensitive", False)),
                    bool(req.get("include_cbuffers", False)),
                    bool(req.get("include_outputs", False)),
                )

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
