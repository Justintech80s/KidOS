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
exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name="kidos-media-classifier",
    console=False,
    disable_windowed_traceback=False,
)
