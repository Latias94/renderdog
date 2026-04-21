import json
import os
import traceback

import renderdoc as rd

from renderdog_action_query import ActionFilter, is_drawcall_like, walk_actions


REQ_PATH = "export_actions_jsonl.request.json"
RESP_PATH = "export_actions_jsonl.response.json"


def write_envelope(ok: bool, result=None, error: str = None) -> None:
    with open(RESP_PATH, "w", encoding="utf-8") as f:
        json.dump({"ok": ok, "result": result, "error": error}, f, ensure_ascii=False)


def main() -> None:
    with open(REQ_PATH, "r", encoding="utf-8") as f:
        req = json.load(f)

    rd.InitialiseReplay(rd.GlobalEnvironment(), [])

    os.makedirs(req["output_dir"], exist_ok=True)

    actions_path = os.path.join(req["output_dir"], f"{req['basename']}.actions.jsonl")
    summary_path = os.path.join(req["output_dir"], f"{req['basename']}.summary.json")

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

            counters = {"total_actions": 0, "drawcall_actions": 0}
            action_filter = ActionFilter(
                only_drawcalls=bool(req.get("only_drawcalls", False)),
                marker_prefix=str(req.get("marker_prefix") or ""),
                event_min=req.get("event_id_min", None),
                event_max=req.get("event_id_max", None),
                name_contains=req.get("name_contains") or "",
                marker_contains=req.get("marker_contains") or "",
                case_sensitive=bool(req.get("case_sensitive", False)),
            )

            with open(actions_path, "w", encoding="utf-8") as fp:
                def handle_action(action) -> None:
                    rec = {
                        "event_id": action.event_id,
                        "parent_event_id": action.parent_event_id,
                        "depth": action.depth,
                        "name": action.name,
                        "flags": action.flags,
                        "marker_path": action.marker_path,
                        "num_children": action.num_children,
                    }

                    fp.write(json.dumps(rec, ensure_ascii=False) + "\n")

                    counters["total_actions"] += 1
                    if is_drawcall_like(action.flags):
                        counters["drawcall_actions"] += 1

                walk_actions(structured_file, roots, action_filter, handle_action)

            api = str(controller.GetAPIProperties().pipelineType)

            summary = {
                "capture_path": req["capture_path"],
                "api": api,
                "total_actions": int(counters["total_actions"]),
                "drawcall_actions": int(counters["drawcall_actions"]),
                "actions_jsonl_path": actions_path,
            }

            with open(summary_path, "w", encoding="utf-8") as fp:
                json.dump(summary, fp, ensure_ascii=False, indent=2)

            write_envelope(
                True,
                result={
                    "actions_jsonl_path": actions_path,
                    "actions_summary_json_path": summary_path,
                    "total_actions": int(counters["total_actions"]),
                    "drawcall_actions": int(counters["drawcall_actions"]),
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
