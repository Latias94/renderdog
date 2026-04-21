import renderdoc as rd

from renderdog_qrenderdoc import run_json_job, with_capture_controller


REQ_PATH = "replay_pick_pixel_json.request.json"
RESP_PATH = "replay_pick_pixel_json.response.json"


def handle_request(req):
    def run(controller):
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

        return {
            "capture_path": req["capture_path"],
            "event_id": event_id,
            "texture_index": int(req["texture_index"]),
            "x": int(req["x"]),
            "y": int(req["y"]),
            "rgba": rgba,
        }

    return with_capture_controller(req["capture_path"], run)


if __name__ == "__main__":
    run_json_job(REQ_PATH, RESP_PATH, handle_request)
    raise SystemExit(0)
