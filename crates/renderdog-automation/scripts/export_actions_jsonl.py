import json
import os
import traceback

import renderdoc as rd


REQ_PATH = "export_actions_jsonl.request.json"
RESP_PATH = "export_actions_jsonl.response.json"


def write_envelope(ok: bool, result=None, error: str = None) -> None:
    with open(RESP_PATH, "w", encoding="utf-8") as f:
        json.dump({"ok": ok, "result": result, "error": error}, f, ensure_ascii=False)


def is_drawcall_like(flags: int) -> bool:
    return bool(
        (flags & rd.ActionFlags.Drawcall)
        or (flags & rd.ActionFlags.Dispatch)
        or (flags & rd.ActionFlags.MeshDispatch)
        or (flags & rd.ActionFlags.DispatchRay)
    )


def marker_path_join(marker_path) -> str:
    if not marker_path:
        return ""
    return "/".join([str(x) for x in marker_path])

def normalize(s: str, case_sensitive: bool) -> str:
    if s is None:
        return ""
    if case_sensitive:
        return str(s)
    return str(s).lower()


def iter_actions(structured_file, actions, marker_stack, parent_event_id, depth, out_fp, counters,
                 only_drawcalls: bool, marker_prefix: str,
                 event_min, event_max,
                 name_contains: str, marker_contains: str,
                 case_sensitive: bool):
    for a in actions:
        name = a.GetName(structured_file)
        flags = a.flags

        effective_marker_path = list(marker_stack)
        if flags & rd.ActionFlags.PushMarker:
            effective_marker_path.append(str(name))

        joined_marker_path = marker_path_join(effective_marker_path)
        name_str = str(name)

        def recurse():
            if flags & rd.ActionFlags.PushMarker:
                marker_stack.append(str(name))
                iter_actions(structured_file, a.children, marker_stack, a.eventId, depth + 1, out_fp, counters,
                             only_drawcalls, marker_prefix,
                             event_min, event_max,
                             name_contains, marker_contains,
                             case_sensitive)
                marker_stack.pop()
            else:
                iter_actions(structured_file, a.children, marker_stack, a.eventId, depth + 1, out_fp, counters,
                             only_drawcalls, marker_prefix,
                             event_min, event_max,
                             name_contains, marker_contains,
                             case_sensitive)

        if marker_prefix:
            if not (joined_marker_path == marker_prefix or joined_marker_path.startswith(marker_prefix + "/")):
                recurse()
                continue

        eid = int(a.eventId)

        should_emit = True
        if only_drawcalls and not is_drawcall_like(flags):
            should_emit = False
        if event_min is not None and eid < int(event_min):
            should_emit = False
        if event_max is not None and eid > int(event_max):
            should_emit = False

        if name_contains:
            if name_contains not in normalize(name_str, case_sensitive):
                should_emit = False
        if marker_contains:
            if marker_contains not in normalize(joined_marker_path, case_sensitive):
                should_emit = False

        if should_emit:
            rec = {
                "event_id": eid,
                "parent_event_id": int(parent_event_id) if parent_event_id is not None else None,
                "depth": int(depth),
                "name": name_str,
                "flags": int(flags),
                "marker_path": effective_marker_path,
                "num_children": int(len(a.children)),
            }

            out_fp.write(json.dumps(rec, ensure_ascii=False) + "\n")

            counters["total_actions"] += 1
            if is_drawcall_like(flags):
                counters["drawcall_actions"] += 1

        recurse()


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
            with open(actions_path, "w", encoding="utf-8") as fp:
                iter_actions(structured_file, roots, [], None, 0, fp, counters,
                             bool(req.get("only_drawcalls", False)),
                             str(req.get("marker_prefix") or ""),
                             req.get("event_id_min", None),
                             req.get("event_id_max", None),
                             normalize(req.get("name_contains") or "", bool(req.get("case_sensitive", False))),
                             normalize(req.get("marker_contains") or "", bool(req.get("case_sensitive", False))),
                             bool(req.get("case_sensitive", False)))

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
