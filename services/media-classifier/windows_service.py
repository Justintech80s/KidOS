from __future__ import annotations

import importlib
import os
import sys
import threading
from pathlib import Path
from datetime import datetime, timezone

import servicemanager
import uvicorn
import win32event
import win32service
import win32serviceutil

SERVICE_NAME = "KidOSMediaClassifier"
DISPLAY_NAME = "KidOS Media Classifier"
DESCRIPTION = "Local KidOS image and video safety classification service."

DIAGNOSTICS_DIR = Path(r"C:\ProgramData\KidOS\Diagnostics")
SERVICE_LOG = DIAGNOSTICS_DIR / "media-classifier-service.log"


def _service_log(message: str) -> None:
    try:
        DIAGNOSTICS_DIR.mkdir(parents=True, exist_ok=True)
        stamp = datetime.now(timezone.utc).isoformat()
        with SERVICE_LOG.open("a", encoding="utf-8") as handle:
            handle.write(f"{stamp} {message}\n")
    except Exception:
        pass


def _runtime_dir() -> Path:
    if getattr(sys, "frozen", False):
        return Path(sys.executable).resolve().parent
    return Path(__file__).resolve().parent


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
            runtime_dir = _runtime_dir()
            os.chdir(runtime_dir)
            _service_log(f"Starting KidOS Media Classifier from {runtime_dir}")

            os.environ["KIDOS_MEDIA_CLASSIFIER_TOKEN"] = _load_token()
            os.environ.setdefault("KIDOS_MEDIA_CLASSIFIER_PORT", "8765")
            _service_log("Protected classifier token loaded.")

            # Import the application only after the service token and runtime
            # directory are configured. Passing the application object directly
            # avoids Uvicorn trying to resolve "app:app" from System32 when the
            # process is launched by the Windows Service Control Manager.
            app_module = importlib.import_module("app")
            application = getattr(app_module, "app")
            _service_log("FastAPI application imported successfully.")

            config = uvicorn.Config(
                application,
                host="127.0.0.1",
                port=int(os.environ["KIDOS_MEDIA_CLASSIFIER_PORT"]),
                access_log=False,
                log_level="warning",
            )
            self.server = uvicorn.Server(config)

            worker_error: list[BaseException] = []

            def run_server() -> None:
                try:
                    self.server.run()
                except BaseException as exc:
                    worker_error.append(exc)
                    _service_log(f"Uvicorn worker crashed: {type(exc).__name__}: {exc}")

            self.worker = threading.Thread(
                target=run_server,
                name="KidOSMediaClassifierUvicorn",
                daemon=True,
            )
            self.worker.start()
            _service_log("Uvicorn worker started.")

            while True:
                result = win32event.WaitForSingleObject(self.stop_event, 1000)
                if result == win32event.WAIT_OBJECT_0:
                    _service_log("Service stop requested.")
                    break
                if worker_error:
                    raise RuntimeError(
                        f"KidOS media classifier web service crashed: {worker_error[0]}"
                    )
                if self.worker is not None and not self.worker.is_alive():
                    raise RuntimeError("KidOS media classifier web service stopped unexpectedly")
        except Exception as exc:
            _service_log(f"Service failed: {type(exc).__name__}: {exc}")
            servicemanager.LogErrorMsg(f"KidOS Media Classifier failed: {exc}")
            raise
        finally:
            if self.server is not None:
                self.server.should_exit = True
            if self.worker is not None:
                self.worker.join(timeout=10)
            _service_log("KidOS Media Classifier service stopped.")


def main():
    if len(sys.argv) == 1:
        servicemanager.Initialize()
        servicemanager.PrepareToHostSingle(KidOSMediaClassifierService)
        servicemanager.StartServiceCtrlDispatcher()
    else:
        win32serviceutil.HandleCommandLine(KidOSMediaClassifierService)


if __name__ == "__main__":
    main()
