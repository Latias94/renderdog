import renderdoc as rd

from renderdog_qrenderdoc import (
    get_texture_by_index,
    run_json_job,
    set_frame_event_if_present,
    with_capture_controller,
)


REQ_PATH = "replay_pick_pixel.request.json"
RESP_PATH = "replay_pick_pixel.response.json"


def handle_request(req):
    def run(controller):
        event_id = set_frame_event_if_present(controller, req.get("event_id", None))
        idx, t = get_texture_by_index(controller, req["texture_index"])
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
            "texture_index": idx,
            "x": int(req["x"]),
            "y": int(req["y"]),
            "rgba": rgba,
        }

    return with_capture_controller(req["capture_path"], run)


if __name__ == "__main__":
    run_json_job(REQ_PATH, RESP_PATH, handle_request)
    raise SystemExit(0)
