from renderdog_qrenderdoc import (
    run_json_job,
    set_frame_event_if_present,
    with_capture_controller,
)


REQ_PATH = "replay_list_textures.request.json"
RESP_PATH = "replay_list_textures.response.json"


def handle_request(req):
    def run(controller):
        event_id = set_frame_event_if_present(controller, req.get("event_id", None))

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
                if desc is not None and not name:
                    name = str(desc.name or "")
            except Exception:
                pass

            array_size = getattr(t, "arraysize", getattr(t, "arraySize", 1))
            ms_samp = getattr(t, "msSamp", getattr(t, "msSamples", 1))
            byte_size = getattr(
                t,
                "byteSize",
                getattr(t, "bytesize", getattr(t, "byte_size", 0)),
            )
            out.append(
                {
                    "index": int(i),
                    "resource_id": int(rid),
                    "name": name,
                    "width": int(t.width),
                    "height": int(t.height),
                    "depth": int(t.depth),
                    "mips": int(t.mips),
                    "array_size": int(array_size),
                    "ms_samp": int(ms_samp),
                    "byte_size": int(byte_size),
                }
            )

        return {
            "capture_path": req["capture_path"],
            "event_id": event_id,
            "textures": out,
        }

    return with_capture_controller(req["capture_path"], run)


if __name__ == "__main__":
    run_json_job(REQ_PATH, RESP_PATH, handle_request)
    raise SystemExit(0)
