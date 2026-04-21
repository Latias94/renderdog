import json
import traceback

import renderdoc as rd

from renderdog_action_query import ActionFilter, walk_actions


REQ_PATH = "find_events_json.request.json"
RESP_PATH = "find_events_json.response.json"


def write_envelope(ok: bool, result=None, error: str = None) -> None:
    with open(RESP_PATH, "w", encoding="utf-8") as f:
        json.dump({"ok": ok, "result": result, "error": error}, f, ensure_ascii=False)


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
            structured_file = controller.GetStructuredFile()
            roots = controller.GetRootActions()

            out_list = []
            counters = {"truncated": False, "total_matches": 0}
            action_filter = ActionFilter(
                bool(req.get("only_drawcalls", False)),
                req.get("marker_prefix", None),
                req.get("event_id_min", None),
                req.get("event_id_max", None),
                req.get("name_contains", None),
                req.get("marker_contains", None),
                bool(req.get("case_sensitive", False)),
            )
            max_results = req.get("max_results", None)

            def handle_action(action) -> None:
                counters["total_matches"] += 1
                if counters.get("first_event_id", None) is None:
                    counters["first_event_id"] = action.event_id
                counters["last_event_id"] = action.event_id
                if max_results is None or len(out_list) < int(max_results):
                    out_list.append(
                        {
                            "event_id": action.event_id,
                            "parent_event_id": action.parent_event_id,
                            "depth": action.depth,
                            "name": action.name,
                            "flags": action.flags,
                            "marker_path": action.marker_path,
                        }
                    )
                else:
                    counters["truncated"] = True

            walk_actions(structured_file, roots, action_filter, handle_action)

            write_envelope(
                True,
                result={
                    "capture_path": req["capture_path"],
                    "summary": {
                        "total_matches": int(counters["total_matches"]),
                        "truncated": bool(counters["truncated"]),
                        "first_event_id": counters.get("first_event_id", None),
                        "last_event_id": counters.get("last_event_id", None),
                    },
                    "matches": out_list,
                },
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
        write_envelope(False, error=traceback.format_exc())
    raise SystemExit(0)
