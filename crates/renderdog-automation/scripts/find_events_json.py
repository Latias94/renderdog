import json
import traceback

import renderdoc as rd


REQ_PATH = "find_events_json.request.json"
RESP_PATH = "find_events_json.response.json"


FLAG_NAMES = [
    ("Drawcall", rd.ActionFlags.Drawcall),
    ("Dispatch", rd.ActionFlags.Dispatch),
    ("MeshDispatch", rd.ActionFlags.MeshDispatch),
    ("DispatchRay", rd.ActionFlags.DispatchRay),
    ("Present", rd.ActionFlags.Present),
    ("PushMarker", rd.ActionFlags.PushMarker),
    ("PopMarker", rd.ActionFlags.PopMarker),
    ("PassBoundary", rd.ActionFlags.PassBoundary),
    ("BeginPass", rd.ActionFlags.BeginPass),
    ("EndPass", rd.ActionFlags.EndPass),
    ("CommandBufferBoundary", rd.ActionFlags.CommandBufferBoundary),
]


def flags_to_names(flags):
    names = []
    for name, bit in FLAG_NAMES:
        if flags & bit:
            names.append(name)
    return names


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


def write_envelope(ok: bool, result=None, error: str = None) -> None:
    with open(RESP_PATH, "w", encoding="utf-8") as f:
        json.dump({"ok": ok, "result": result, "error": error}, f, ensure_ascii=False)


def iter_actions(
    structured_file,
    actions,
    marker_stack,
    parent_event_id,
    depth,
    out_list,
    counters,
    only_drawcalls: bool,
    marker_prefix: str,
    event_min,
    event_max,
    name_contains: str,
    marker_contains: str,
    case_sensitive: bool,
    max_results,
):
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
                iter_actions(
                    structured_file,
                    a.children,
                    marker_stack,
                    a.eventId,
                    depth + 1,
                    out_list,
                    counters,
                    only_drawcalls,
                    marker_prefix,
                    event_min,
                    event_max,
                    name_contains,
                    marker_contains,
                    case_sensitive,
                    max_results,
                )
                marker_stack.pop()
            else:
                iter_actions(
                    structured_file,
                    a.children,
                    marker_stack,
                    a.eventId,
                    depth + 1,
                    out_list,
                    counters,
                    only_drawcalls,
                    marker_prefix,
                    event_min,
                    event_max,
                    name_contains,
                    marker_contains,
                    case_sensitive,
                    max_results,
                )

        if marker_prefix:
            if not (
                joined_marker_path == marker_prefix
                or joined_marker_path.startswith(marker_prefix + "/")
            ):
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
            counters["total_matches"] += 1
            if counters.get("first_event_id", None) is None:
                counters["first_event_id"] = eid
            counters["last_event_id"] = eid
            if max_results is None or len(out_list) < int(max_results):
                out_list.append(
                    {
                        "event_id": eid,
                        "parent_event_id": int(parent_event_id)
                        if parent_event_id is not None
                        else None,
                        "depth": int(depth),
                        "name": name_str,
                        "flags": int(flags),
                        "flags_names": flags_to_names(flags),
                        "marker_path": effective_marker_path,
                        "marker_path_joined": joined_marker_path,
                    }
                )
            else:
                counters["truncated"] = True

        recurse()


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
            iter_actions(
                structured_file,
                roots,
                [],
                None,
                0,
                out_list,
                counters,
                bool(req.get("only_drawcalls", False)),
                req.get("marker_prefix", None),
                req.get("event_id_min", None),
                req.get("event_id_max", None),
                req.get("name_contains", None),
                req.get("marker_contains", None),
                bool(req.get("case_sensitive", False)),
                req.get("max_results", None),
            )

            write_envelope(
                True,
                result={
                    "capture_path": req["capture_path"],
                    "total_matches": int(counters["total_matches"]),
                    "truncated": bool(counters["truncated"]),
                    "first_event_id": counters.get("first_event_id", None),
                    "last_event_id": counters.get("last_event_id", None),
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
