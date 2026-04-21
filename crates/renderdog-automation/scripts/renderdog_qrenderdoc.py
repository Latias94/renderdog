import json
import traceback

import renderdoc as rd


def load_request(request_path):
    with open(request_path, "r", encoding="utf-8") as f:
        return json.load(f)


def write_envelope(response_path, ok: bool, result=None, error: str = None) -> None:
    with open(response_path, "w", encoding="utf-8") as f:
        json.dump({"ok": ok, "result": result, "error": error}, f, ensure_ascii=False)


def run_json_job(request_path, response_path, handler) -> None:
    try:
        request = load_request(request_path)
        result = handler(request)
    except Exception:
        write_envelope(response_path, False, error=traceback.format_exc())
    else:
        write_envelope(response_path, True, result=result)


def with_replay(callback):
    rd.InitialiseReplay(rd.GlobalEnvironment(), [])
    try:
        return callback()
    finally:
        rd.ShutdownReplay()


def with_capture_controller(capture_path, callback):
    def run():
        cap = rd.OpenCaptureFile()
        try:
            result = cap.OpenFile(capture_path, "", None)
            if result != rd.ResultCode.Succeeded:
                raise RuntimeError("Couldn't open file: " + str(result))

            if not cap.LocalReplaySupport():
                raise RuntimeError("Capture cannot be replayed")

            result, controller = cap.OpenCapture(rd.ReplayOptions(), None)
            if result != rd.ResultCode.Succeeded:
                raise RuntimeError("Couldn't initialise replay: " + str(result))

            try:
                return callback(controller)
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

    return with_replay(run)


def is_drawcall_like(flags: int) -> bool:
    return bool(
        (flags & rd.ActionFlags.Drawcall)
        or (flags & rd.ActionFlags.Dispatch)
        or (flags & rd.ActionFlags.MeshDispatch)
        or (flags & rd.ActionFlags.DispatchRay)
    )


def set_frame_event_if_present(controller, event_id, apply_changes: bool = True):
    if event_id is None:
        return None

    event_id = int(event_id)
    controller.SetFrameEvent(event_id, apply_changes)
    return event_id


def get_texture_by_index(controller, texture_index):
    textures = controller.GetTextures()
    texture_index = int(texture_index)
    if texture_index < 0 or texture_index >= len(textures):
        raise RuntimeError("texture_index out of range")

    return texture_index, textures[texture_index]


def flatten_actions(actions):
    out = []
    for action in actions:
        out.append(action)
        out.extend(flatten_actions(action.children))
    return out


def pick_last_drawcall_event_id(controller) -> int:
    actions = flatten_actions(controller.GetRootActions())
    if not actions:
        return 0

    drawcalls = []
    for action in actions:
        try:
            if is_drawcall_like(action.flags):
                drawcalls.append(action)
        except Exception:
            pass

    if drawcalls:
        return int(max(action.eventId for action in drawcalls))

    return int(max(action.eventId for action in actions))


def resolve_event_selection(controller, event_selection: str, event_id=None) -> int:
    if event_selection == "event_id":
        if event_id is None:
            raise RuntimeError("event_id is required when event_selection is event_id")
        return int(event_id)

    if event_selection == "last_drawcall":
        return pick_last_drawcall_event_id(controller)

    raise RuntimeError(f"unsupported event_selection: {event_selection}")
