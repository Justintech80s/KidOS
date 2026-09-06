# KidOS Media Classifier

Local-only image/video moderation service used by the privileged KidOS Guardian service.

It uses a zero-shot vision model to classify downloaded media into KidOS safety categories. Video files are sampled across multiple frames. The service binds only to `127.0.0.1` and requires the shared `KIDOS_MEDIA_CLASSIFIER_TOKEN`.

This classifier is one safety signal, not a guarantee. Guardian remains fail-closed: if the model is unavailable, confidence is insufficient, or classification fails, the media stays quarantined for parent review.

## Run for development

```powershell
$env:KIDOS_MEDIA_CLASSIFIER_TOKEN="development-secret"
python -m pip install -e services/media-classifier
python services/media-classifier/app.py
```

The production installer should provision a random service token and package the model locally so classification does not depend on a child account's network access.
