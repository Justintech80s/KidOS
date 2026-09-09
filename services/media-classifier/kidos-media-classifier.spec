# -*- mode: python ; coding: utf-8 -*-
from PyInstaller.utils.hooks import collect_all

datas = []
binaries = []
hiddenimports = []

for package in ["transformers", "torch", "cv2", "PIL", "uvicorn", "fastapi", "win32timezone"]:
    package_datas, package_binaries, package_hidden = collect_all(package)
    datas += package_datas
    binaries += package_binaries
    hiddenimports += package_hidden

hiddenimports += [
    "app",
    "win32serviceutil",
    "win32service",
    "win32event",
    "servicemanager",
]

a = Analysis(
    ["windows_service.py"],
    pathex=["."],
    binaries=binaries,
    datas=datas,
    hiddenimports=hiddenimports,
    noarchive=False,
)
pyz = PYZ(a.pure)

# Windows services should not use a giant one-file PyInstaller payload. One-file
# executables must unpack Python/Torch/Transformers before connecting to the
# Service Control Manager, which can leave SCM in START_PENDING long enough to
# trigger startup timeouts. Build an on-disk bundle instead so the service host
# can connect immediately and load its dependencies in place.
exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name="kidos-media-classifier",
    console=False,
    disable_windowed_traceback=False,
)

coll = COLLECT(
    exe,
    a.binaries,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name="kidos-media-classifier",
)
