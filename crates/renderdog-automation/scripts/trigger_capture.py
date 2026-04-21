import time

import renderdoc as rd

from renderdog_qrenderdoc import run_job, with_replay


REQUEST_PATH = "trigger_capture.request"
RESPONSE_PATH = "trigger_capture.response"


def handle_request(req):
    def run():
        target = rd.CreateTargetControl(
            req["host"],
            int(req["target_ident"]),
            "renderdog",
            True,
        )
        if target is None:
            raise RuntimeError(
                f"CreateTargetControl failed for {req['host']}:{int(req['target_ident'])}"
            )

        try:
            target.TriggerCapture(int(req["num_frames"]))

            deadline = time.time() + float(req["timeout_s"])
            while time.time() < deadline:
                msg = target.ReceiveMessage(None)
                if msg is None:
                    continue
                if msg.type == rd.TargetControlMessageType.NewCapture:
                    cap = msg.newCapture
                    return {
                        "capture_path": cap.path,
                        "frame_number": int(cap.frameNumber),
                        "api": str(cap.api),
                    }

            raise RuntimeError("Timed out waiting for NewCapture message")
        finally:
            try:
                target.Shutdown()
            except Exception:
                pass

    return with_replay(run)


if __name__ == "__main__":
    run_job(REQUEST_PATH, RESPONSE_PATH, handle_request)
    raise SystemExit(0)
