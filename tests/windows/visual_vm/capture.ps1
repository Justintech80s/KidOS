param(
  [Parameter(Mandatory = $true)]
  [string]$ArtifactDir,

  [Parameter(Mandatory = $true)]
  [ValidateSet("healthy-home", "restricted-safe-mode")]
  [string]$Label,

  [switch]$RecordVideo,
  [int]$VideoSeconds = 8
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

New-Item -ItemType Directory -Force -Path $ArtifactDir | Out-Null
$artifactRoot = (Resolve-Path -LiteralPath $ArtifactDir).Path

function Save-DesktopScreenshot {
  param(
    [Parameter(Mandatory = $true)]
    [string]$OutputPath
  )

  $bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
  if ($bounds.Width -le 0 -or $bounds.Height -le 0) {
    throw "Interactive Windows desktop is unavailable for visual capture."
  }

  $bitmap = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  try {
    $graphics.CopyFromScreen(
      $bounds.Left,
      $bounds.Top,
      0,
      0,
      $bitmap.Size,
      [System.Drawing.CopyPixelOperation]::SourceCopy
    )
    $bitmap.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
  }
  finally {
    $graphics.Dispose()
    $bitmap.Dispose()
  }

  $file = Get-Item -LiteralPath $OutputPath -ErrorAction Stop
  if ($file.Length -le 0) {
    throw "Visual capture produced an empty screenshot: $OutputPath"
  }
  return $file.FullName
}

$screenshotName = switch ($Label) {
  "healthy-home" { "healthy-home.png" }
  "restricted-safe-mode" { "restricted-safe-mode.png" }
}

$screenshotPath = Join-Path $artifactRoot $screenshotName
$screenshotPath = Save-DesktopScreenshot -OutputPath $screenshotPath

$videoPath = $null
$videoAvailable = $false
$videoReason = "not_requested"

if ($RecordVideo) {
  $ffmpeg = Get-Command ffmpeg -ErrorAction SilentlyContinue
  if ($null -eq $ffmpeg) {
    $videoReason = "ffmpeg_unavailable"
  }
  else {
    $videoPath = Join-Path $artifactRoot "KidOS-Visual-VM.mp4"
    $arguments = @(
      "-y",
      "-f", "gdigrab",
      "-framerate", "15",
      "-i", "desktop",
      "-t", [string]$VideoSeconds,
      "-c:v", "libx264",
      "-preset", "veryfast",
      "-pix_fmt", "yuv420p",
      $videoPath
    )

    $recording = Start-Process -FilePath $ffmpeg.Source -ArgumentList $arguments -Wait -PassThru -WindowStyle Hidden
    if ($recording.ExitCode -eq 0 -and (Test-Path -LiteralPath $videoPath)) {
      $videoFile = Get-Item -LiteralPath $videoPath
      if ($videoFile.Length -gt 0) {
        $videoAvailable = $true
        $videoReason = "recorded"
        $videoPath = $videoFile.FullName
      }
      else {
        $videoPath = $null
        $videoReason = "empty_video"
      }
    }
    else {
      $videoPath = $null
      $videoReason = "ffmpeg_failed"
    }
  }
}

$result = [ordered]@{
  visualCapture = $true
  label = $Label
  screenshots = @($screenshotPath)
  video = $videoPath
  videoAvailable = $videoAvailable
  videoReason = $videoReason
  capturedAt = (Get-Date).ToString("o")
}

$result | ConvertTo-Json -Depth 5
