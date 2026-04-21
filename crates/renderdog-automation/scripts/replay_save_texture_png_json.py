import os

import renderdoc as rd

from renderdog_qrenderdoc import (
    get_texture_by_index,
    run_json_job,
    set_frame_event_if_present,
    with_capture_controller,
)


REQ_PATH = "replay_save_texture_png_json.request.json"
RESP_PATH = "replay_save_texture_png_json.response.json"


def handle_request(req):
    out_dir = os.path.dirname(req["output_path"])
    if out_dir:
        os.makedirs(out_dir, exist_ok=True)

    def run(controller):
        event_id = set_frame_event_if_present(controller, req.get("event_id", None))
        idx, t = get_texture_by_index(controller, req["texture_index"])

        save = rd.TextureSave()
        save.resourceId = t.resourceId
        save.destType = rd.FileType.PNG
        save.mip = 0

        result = controller.SaveTexture(save, str(req["output_path"]))
        if result != rd.ResultCode.Succeeded:
            raise RuntimeError("SaveTexture failed: " + str(result))

        return {
            "capture_path": req["capture_path"],
            "event_id": event_id,
            "texture_index": idx,
            "output_path": str(req["output_path"]),
        }

    return with_capture_controller(req["capture_path"], run)


if __name__ == "__main__":
    run_json_job(REQ_PATH, RESP_PATH, handle_request)
    raise SystemExit(0)
