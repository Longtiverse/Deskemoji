#!/usr/bin/env python3
"""
Generate Deskemoji runtime assets from the local system emoji renderer.

This uses Microsoft Edge headless screenshots so the exported PNGs match
the same native glyph style shown in the approved HTML preview.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from urllib.parse import quote


ROOT = Path(__file__).resolve().parent
OUTPUT_DIR = ROOT / "assets" / "emoji"
VIEWPORT = 1024
FONT_SIZE = 860

EMOJIS = [
    ("happy.png", "\U0001F642", "Happy"),
    ("sad.png", "\U0001F622", "Sad"),
    ("angry.png", "\U0001F620", "Angry"),
    ("sleepy.png", "\U0001F634", "Sleepy"),
    ("thinking.png", "\U0001F914", "Thinking"),
    ("hot.png", "\U0001F975", "Hot"),
    ("mindblown.png", "\U0001F92F", "Mindblown"),
    ("goodnight.png", "\U0001F60C", "Goodnight"),
]


def find_edge() -> Path:
    candidates = [
        Path(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
        Path(r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"),
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate

    located = shutil.which("msedge")
    if located:
        return Path(located)

    raise FileNotFoundError("Microsoft Edge was not found")


def build_html(glyph: str) -> str:
    escaped = glyph.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    return f"""<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <style>
    html, body {{
      margin: 0;
      width: {VIEWPORT}px;
      height: {VIEWPORT}px;
      overflow: hidden;
      background: transparent;
    }}
    body {{
      display: flex;
      align-items: center;
      justify-content: center;
    }}
    .glyph {{
      font-family: "Segoe UI Emoji", "Apple Color Emoji", "Noto Color Emoji", sans-serif;
      font-size: {FONT_SIZE}px;
      line-height: 1;
      transform: translateY(-8px);
      user-select: none;
    }}
  </style>
</head>
<body>
  <div class="glyph">{escaped}</div>
</body>
</html>
"""


def screenshot_glyph(edge: Path, glyph: str, destination: Path) -> None:
    with tempfile.NamedTemporaryFile("w", suffix=".html", delete=False, encoding="utf-8") as tmp:
        tmp.write(build_html(glyph))
        tmp_path = Path(tmp.name)

    try:
        url = tmp_path.resolve().as_uri()
        with tempfile.TemporaryDirectory() as profile_dir:
            subprocess.run(
                [
                    str(edge),
                    "--headless",
                    "--disable-gpu",
                    "--hide-scrollbars",
                    f"--user-data-dir={profile_dir}",
                    f"--window-size={VIEWPORT},{VIEWPORT}",
                    "--default-background-color=00000000",
                    f"--screenshot={destination}",
                    url,
                ],
                check=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
    finally:
        tmp_path.unlink(missing_ok=True)


def main() -> int:
    edge = find_edge()
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    for filename, glyph, label in EMOJIS:
        output_path = OUTPUT_DIR / filename
        screenshot_glyph(edge, glyph, output_path)
        print(f"generated {filename} from native glyph ({label})")

    return 0


if __name__ == "__main__":
    sys.exit(main())
