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
