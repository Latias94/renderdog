import json
import os

from renderdog_action_query import ActionFilter, is_drawcall_like, walk_actions
from renderdog_qrenderdoc import run_job, with_capture_controller


REQUEST_PATH = "export_actions_jsonl.request"
RESPONSE_PATH = "export_actions_jsonl.response"


def handle_request(req):
    os.makedirs(req["output_dir"], exist_ok=True)

    actions_path = os.path.join(req["output_dir"], f"{req['basename']}.actions.jsonl")
    summary_path = os.path.join(req["output_dir"], f"{req['basename']}.summary.json")

    def run(controller):
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

        return {
            "actions_jsonl_path": actions_path,
            "actions_summary_json_path": summary_path,
            "total_actions": int(counters["total_actions"]),
            "drawcall_actions": int(counters["drawcall_actions"]),
        }

    return with_capture_controller(req["capture_path"], run)


if __name__ == "__main__":
    run_job(REQUEST_PATH, RESPONSE_PATH, handle_request)
    raise SystemExit(0)
