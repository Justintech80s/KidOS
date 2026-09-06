from __future__ import annotations

import os
import sys
import threading
import time
from pathlib import Path

import servicemanager
import uvicorn
import win32event
import win32service
import win32serviceutil

SERVICE_NAME = "KidOSMediaClassifier"
DISPLAY_NAME = "KidOS Media Classifier"
DESCRIPTION = "Local KidOS image and video safety classification service."


def _load_token() -> str:
    token_path = Path(os.environ.get("KIDOS_MEDIA_TOKEN_FILE", r"C:\ProgramData\KidOS\Guardian\media-classifier.token"))
    if not token_path.is_file():
        raise RuntimeError("KidOS media classifier token file is missing")
    token = token_path.read_text(encoding="utf-8").strip()
    if len(token) < 32:
        raise RuntimeError("KidOS media classifier token is invalid")
    return token


class KidOSMediaClassifierService(win32serviceutil.ServiceFramework):
    _svc_name_ = SERVICE_NAME
    _svc_display_name_ = DISPLAY_NAME
    _svc_description_ = DESCRIPTION

    def __init__(self, args):
        super().__init__(args)
        self.stop_event = win32event.CreateEvent(None, 0, 0, None)
        self.server: uvicorn.Server | None = None
        self.worker: threading.Thread | None = None

    def SvcStop(self):
        self.ReportServiceStatus(win32service.SERVICE_STOP_PENDING)
        if self.server is not None:
            self.server.should_exit = True
        win32event.SetEvent(self.stop_event)

    def SvcDoRun(self):
        try:
            os.environ["KIDOS_MEDIA_CLASSIFIER_TOKEN"] = _load_token()
            os.environ.setdefault("KIDOS_MEDIA_CLASSIFIER_PORT", "8765")
            config = uvicorn.Config(
                "app:app",
                host="127.0.0.1",
                port=int(os.environ["KIDOS_MEDIA_CLASSIFIER_PORT"]),
                access_log=False,
                log_level="warning",
            )
            self.server = uvicorn.Server(config)
            self.worker = threading.Thread(target=self.server.run, daemon=True)
            self.worker.start()

            while True:
                result = win32event.WaitForSingleObject(self.stop_event, 1000)
                if result == win32event.WAIT_OBJECT_0:
                    break
                if self.worker is not None and not self.worker.is_alive():
                    raise RuntimeError("KidOS media classifier web service stopped unexpectedly")
        except Exception as exc:
            servicemanager.LogErrorMsg(f"KidOS Media Classifier failed: {exc}")
            raise
        finally:
            if self.server is not None:
                self.server.should_exit = True
            if self.worker is not None:
                self.worker.join(timeout=10)


def main():
    if len(sys.argv) == 1:
        servicemanager.Initialize()
        servicemanager.PrepareToHostSingle(KidOSMediaClassifierService)
        servicemanager.StartServiceCtrlDispatcher()
    else:
        win32serviceutil.HandleCommandLine(KidOSMediaClassifierService)


if __name__ == "__main__":
    main()
