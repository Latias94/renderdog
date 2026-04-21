import json
import os
import traceback

import renderdoc as rd


REQ_PATH = "replay_save_outputs_png_json.request.json"
RESP_PATH = "replay_save_outputs_png_json.response.json"
OUTPUT_KIND_COLOR = "color"
OUTPUT_KIND_DEPTH = "depth"


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
    # Prefer the last drawcall-like event over e.g. Present, so outputs are meaningful.
    drawcalls = []
    for a in actions:
        try:
            if (
                (a.flags & rd.ActionFlags.Drawcall)
                or (a.flags & rd.ActionFlags.Dispatch)
                or (a.flags & rd.ActionFlags.MeshDispatch)
                or (a.flags & rd.ActionFlags.DispatchRay)
            ):
                drawcalls.append(a)
        except Exception:
            pass

    if drawcalls:
        return int(max(a.eventId for a in drawcalls))

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
                        "kind": OUTPUT_KIND_COLOR,
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
                            "kind": OUTPUT_KIND_DEPTH,
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
