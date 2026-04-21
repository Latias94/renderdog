import os

import renderdoc as rd

from renderdog_qrenderdoc import run_json_job, with_capture_controller


REQ_PATH = "replay_save_texture_png_json.request.json"
RESP_PATH = "replay_save_texture_png_json.response.json"


def handle_request(req):
    out_dir = os.path.dirname(req["output_path"])
    if out_dir:
        os.makedirs(out_dir, exist_ok=True)

    def run(controller):
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

        return {
            "capture_path": req["capture_path"],
            "event_id": event_id,
            "texture_index": int(req["texture_index"]),
            "output_path": str(req["output_path"]),
        }

    return with_capture_controller(req["capture_path"], run)


if __name__ == "__main__":
    run_json_job(REQ_PATH, RESP_PATH, handle_request)
    raise SystemExit(0)
