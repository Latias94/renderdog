import os

import renderdoc as rd

from renderdog_qrenderdoc import (
    resolve_event_selection,
    run_json_job,
    set_frame_event_if_present,
    with_capture_controller,
)


REQ_PATH = "replay_save_outputs_png_json.request.json"
RESP_PATH = "replay_save_outputs_png_json.response.json"
OUTPUT_KIND_COLOR = "color"
OUTPUT_KIND_DEPTH = "depth"


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


def handle_request(req):
    os.makedirs(req["output_dir"], exist_ok=True)

    def run(controller):
        event_id = resolve_event_selection(
            controller,
            req.get("event_selection", "last_drawcall"),
            req.get("event_id", None),
        )
        event_id = set_frame_event_if_present(controller, event_id)

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

        return {
            "capture_path": req["capture_path"],
            "event_id": int(event_id),
            "outputs": outputs,
        }

    return with_capture_controller(req["capture_path"], run)


if __name__ == "__main__":
    run_json_job(REQ_PATH, RESP_PATH, handle_request)
    raise SystemExit(0)
