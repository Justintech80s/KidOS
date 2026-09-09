from __future__ import annotations

import hmac
import os
import sys
from pathlib import Path
from typing import Literal

import cv2
from fastapi import FastAPI, Header, HTTPException
from fastapi.responses import Response
from PIL import Image
from pydantic import BaseModel, Field
from transformers import pipeline

def _default_model() -> str:
    if getattr(sys, "frozen", False):
        bundled = Path(sys.executable).resolve().parent / "model"
        if bundled.is_dir():
            return str(bundled)
    return "openai/clip-vit-base-patch32"


MODEL_ID = os.getenv("KIDOS_MEDIA_MODEL", _default_model())
TOKEN = os.getenv("KIDOS_MEDIA_CLASSIFIER_TOKEN", "")
PORT = int(os.getenv("KIDOS_MEDIA_CLASSIFIER_PORT", "8765"))
MAX_VIDEO_FRAMES = int(os.getenv("KIDOS_MEDIA_MAX_VIDEO_FRAMES", "8"))

LABELS = [
    "safe child-friendly educational content",
    "adult nudity",
    "sexualized content",
    "graphic violence",
    "self harm",
    "drugs or drug use",
    "extremist content",
    "scam or deceptive content",
]

CATEGORY_MAP = {
    LABELS[0]: "safe",
    LABELS[1]: "adult_nudity",
    LABELS[2]: "sexualized_content",
    LABELS[3]: "graphic_violence",
    LABELS[4]: "self_harm",
    LABELS[5]: "drugs",
    LABELS[6]: "extremist_content",
    LABELS[7]: "scam",
}

IMAGE_EXTENSIONS = {".jpg", ".jpeg", ".png", ".gif", ".webp", ".bmp"}
VIDEO_EXTENSIONS = {".mp4", ".webm", ".mov", ".m4v", ".avi"}

app = FastAPI(title="KidOS Media Classifier", docs_url=None, redoc_url=None)
classifier = None


class ClassificationRequest(BaseModel):
    path: str = Field(min_length=1, max_length=2048)


class ClassificationResponse(BaseModel):
    category: str
    risk: Literal["low", "medium", "high"]
    confidence: float
    high_confidence: bool
    classifier_available: bool
    frames_checked: int = 1


def require_token(value: str | None) -> None:
    if not TOKEN:
        raise HTTPException(status_code=503, detail="classifier token is not configured")
    if value is None or not hmac.compare_digest(value, TOKEN):
        raise HTTPException(status_code=401, detail="unauthorized")


def get_classifier():
    global classifier
    if classifier is None:
        classifier = pipeline(
            "zero-shot-image-classification",
            model=MODEL_ID,
            device=-1,
        )
    return classifier


def risk_for(category: str, confidence: float) -> str:
    if category == "safe":
        return "low"
    if category in {"adult_nudity", "sexualized_content", "graphic_violence", "self_harm", "extremist_content"}:
        return "high"
    return "high" if confidence >= 0.65 else "medium"


def classify_image(image: Image.Image) -> tuple[str, str, float]:
    model = get_classifier()
    scores = model(image.convert("RGB"), candidate_labels=LABELS)
    best = scores[0]
    category = CATEGORY_MAP.get(best["label"], "uncertain")
    confidence = float(best["score"])
    return category, risk_for(category, confidence), confidence


def classify_video(path: Path) -> tuple[str, str, float, int]:
    capture = cv2.VideoCapture(str(path))
    if not capture.isOpened():
        raise ValueError("video could not be decoded")

    frame_count = int(capture.get(cv2.CAP_PROP_FRAME_COUNT) or 0)
    sample_count = max(1, min(MAX_VIDEO_FRAMES, frame_count if frame_count > 0 else MAX_VIDEO_FRAMES))
    indices = [int((frame_count - 1) * i / max(1, sample_count - 1)) for i in range(sample_count)] if frame_count > 1 else [0]

    worst_category = "safe"
    worst_risk = "low"
    worst_confidence = 0.0
    checked = 0
    rank = {"low": 0, "medium": 1, "high": 2}

    try:
        for index in indices:
            capture.set(cv2.CAP_PROP_POS_FRAMES, index)
            ok, frame = capture.read()
            if not ok:
                continue
            checked += 1
            rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
            category, risk, confidence = classify_image(Image.fromarray(rgb))
            if rank[risk] > rank[worst_risk] or (risk == worst_risk and confidence > worst_confidence):
                worst_category, worst_risk, worst_confidence = category, risk, confidence
    finally:
        capture.release()

    if checked == 0:
        raise ValueError("no video frames could be decoded")
    return worst_category, worst_risk, worst_confidence, checked


@app.get("/health")
def health(x_kidos_classifier_token: str | None = Header(default=None)):
    """Fast service-readiness probe used by the Windows installer/recovery path.

    Do not load the multi-hundred-megabyte AI model here. Installation should
    prove that the protected local service is alive, authenticated, and can see
    its bundled model path; deeper model initialization is validated separately.
    """
    require_token(x_kidos_classifier_token)
    bundled_model = Path(MODEL_ID)
    model_present = bundled_model.is_dir() if bundled_model.is_absolute() else True
    return {
        "status": "healthy",
        "model": MODEL_ID,
        "model_present": model_present,
        "classifier_loaded": classifier is not None,
    }


@app.get("/model-health")
def model_health(x_kidos_classifier_token: str | None = Header(default=None)):
    """Deep readiness probe for CI/Shadow validation.

    This endpoint intentionally initializes the model and may take much longer
    than the Windows service health probe.
    """
    require_token(x_kidos_classifier_token)
    try:
        get_classifier()
    except Exception as exc:
        raise HTTPException(status_code=503, detail=f"classifier unavailable: {type(exc).__name__}") from exc
    return {"status": "healthy", "model": MODEL_ID, "classifier_loaded": True}


@app.post("/classify", response_model=ClassificationResponse)
def classify(request: ClassificationRequest, x_kidos_classifier_token: str | None = Header(default=None)):
    require_token(x_kidos_classifier_token)
    path = Path(request.path)

    if not path.is_absolute() or not path.is_file():
        raise HTTPException(status_code=400, detail="media path is invalid")

    suffix = path.suffix.lower()
    try:
        if suffix in IMAGE_EXTENSIONS:
            category, risk, confidence = classify_image(Image.open(path))
            frames = 1
        elif suffix in VIDEO_EXTENSIONS:
            category, risk, confidence, frames = classify_video(path)
        else:
            raise HTTPException(status_code=415, detail="unsupported media type")
    except HTTPException:
        raise
    except Exception as exc:
        raise HTTPException(status_code=422, detail=f"media classification failed: {type(exc).__name__}") from exc

    return ClassificationResponse(
        category=category,
        risk=risk,
        confidence=confidence,
        high_confidence=confidence >= 0.72,
        classifier_available=True,
        frames_checked=frames,
    )




@app.post("/thumbnail")
def thumbnail(request: ClassificationRequest, x_kidos_classifier_token: str | None = Header(default=None)):
    require_token(x_kidos_classifier_token)
    path = Path(request.path)

    if not path.is_absolute() or not path.is_file():
        raise HTTPException(status_code=400, detail="media path is invalid")

    suffix = path.suffix.lower()
    try:
        if suffix in IMAGE_EXTENSIONS:
            image = Image.open(path).convert("RGB")
        elif suffix in VIDEO_EXTENSIONS:
            capture = cv2.VideoCapture(str(path))
            try:
                if not capture.isOpened():
                    raise ValueError("video could not be decoded")
                ok, frame = capture.read()
                if not ok:
                    raise ValueError("video preview frame could not be decoded")
                image = Image.fromarray(cv2.cvtColor(frame, cv2.COLOR_BGR2RGB))
            finally:
                capture.release()
        else:
            raise HTTPException(status_code=415, detail="unsupported media type")

        image.thumbnail((320, 320))
        from io import BytesIO
        buffer = BytesIO()
        image.save(buffer, format="JPEG", quality=65, optimize=True)
        data = buffer.getvalue()
        if len(data) > 48 * 1024:
            image.thumbnail((220, 220))
            buffer = BytesIO()
            image.save(buffer, format="JPEG", quality=50, optimize=True)
            data = buffer.getvalue()
        if len(data) > 48 * 1024:
            raise ValueError("thumbnail too large")
        return Response(content=data, media_type="image/jpeg")
    except HTTPException:
        raise
    except Exception as exc:
        raise HTTPException(status_code=422, detail=f"thumbnail generation failed: {type(exc).__name__}") from exc


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=PORT, access_log=False)
