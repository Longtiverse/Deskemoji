#!/usr/bin/env python3
from __future__ import annotations

import math
import shutil
from collections import deque
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw


ROOT = Path(__file__).resolve().parent
ASSET_DIR = ROOT / "assets" / "emoji"
OPEN_EYE_STATES = ("happy", "sad", "angry", "thinking", "hot", "mindblown")
WHITE_MIN = 242
DARK_MAX = (110, 95, 105)
DIR_OFFSETS = {
    "center": (0, 0),
    "left": (-30, 0),
    "right": (30, 0),
    "up": (0, -24),
    "down": (0, 16),
    "ul": (-24, -20),
    "ur": (24, -20),
    "dl": (-20, 14),
    "dr": (20, 14),
}
BASELINE_SIZE = 512


def main() -> int:
    for stem in OPEN_EYE_STATES:
        image = Image.open(ASSET_DIR / f"{stem}.png").convert("RGBA")
        eyes = detect_eyes(image)
        if len(eyes) != 2:
            raise RuntimeError(f"expected 2 eyes in {stem}.png, found {len(eyes)}")

        for suffix, delta in DIR_OFFSETS.items():
            if suffix == "center":
                shutil.copyfile(ASSET_DIR / f"{stem}.png", ASSET_DIR / f"{stem}-{suffix}.png")
                continue

            scale = image.width / BASELINE_SIZE
            scaled_delta = (round(delta[0] * scale), round(delta[1] * scale))
            variant = render_variant(image, eyes, scaled_delta)
            variant.save(ASSET_DIR / f"{stem}-{suffix}.png")

        print(f"generated gaze variants for {stem}")

    return 0


def detect_eyes(image: Image.Image) -> list[dict[str, object]]:
    width, height = image.size
    pixels = image.load()
    white_mask = [
        [
            pixels[x, y][3] > 200
            and pixels[x, y][0] >= WHITE_MIN
            and pixels[x, y][1] >= WHITE_MIN
            and pixels[x, y][2] >= WHITE_MIN
            for x in range(width)
        ]
        for y in range(height)
    ]

    components = connected_components(white_mask)
    eye_components = [component for component in components if len(component) > 8000]
    eye_components.sort(key=lambda component: len(component), reverse=True)
    eye_components = eye_components[:2]
    eye_components.sort(key=lambda component: centroid(component)[0])

    eyes = []
    for component in eye_components:
        xs = [x for x, _ in component]
        ys = [y for _, y in component]
        bbox = (min(xs), min(ys), max(xs), max(ys))
        mask = Image.new("L", image.size, 0)
        mask_pixels = mask.load()
        component_mask = [[False] * (bbox[2] - bbox[0] + 1) for _ in range(bbox[3] - bbox[1] + 1)]
        r_sum = 0
        g_sum = 0
        b_sum = 0
        a_sum = 0
        for x, y in component:
            mask_pixels[x, y] = 255
            component_mask[y - bbox[1]][x - bbox[0]] = True
            r, g, b, a = pixels[x, y]
            r_sum += r
            g_sum += g
            b_sum += b
            a_sum += a

        count = len(component)
        white = (
            round(r_sum / count),
            round(g_sum / count),
            round(b_sum / count),
            round(a_sum / count),
        )
        paint_mask = build_paint_mask(image.size, bbox, component_mask)
        pupil = detect_pupil(image, bbox)
        eyes.append(
            {
                "bbox": bbox,
                "mask": paint_mask,
                "white": white,
                "pupil_center": pupil["center"],
                "pupil_radius": pupil["radius"],
                "pupil_color": pupil["color"],
            }
        )

    return eyes


def detect_pupil(image: Image.Image, bbox: tuple[int, int, int, int]) -> dict[str, object]:
    pixels = image.load()
    x0, y0, x1, y1 = bbox
    dark_mask = [[False] * (x1 - x0 + 1) for _ in range(y1 - y0 + 1)]

    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            r, g, b, a = pixels[x, y]
            if (
                a > 200
                and r <= DARK_MAX[0]
                and g <= DARK_MAX[1]
                and b <= DARK_MAX[2]
            ):
                dark_mask[y - y0][x - x0] = True

    components = connected_components(dark_mask, offset=(x0, y0))
    center_x = (x0 + x1) / 2.0
    center_y = (y0 + y1) / 2.0

    best = None
    for component in components:
        if len(component) < 1000:
            continue
        pupil_center = centroid(component)
        score = ((pupil_center[0] - center_x) ** 2 + (pupil_center[1] - center_y) ** 2, -len(component))
        if best is None or score < best[0]:
            best = (score, component)

    if best is None:
        raise RuntimeError(f"failed to detect pupil in bbox {bbox}")

    pupil_component = best[1]
    pupil_center = centroid(pupil_component)
    scale = image.size[0] / BASELINE_SIZE
    radius = max(round(8 * scale), round(math.sqrt(len(pupil_component) / math.pi)))
    sample_x = max(x0, min(x1, round(pupil_center[0])))
    sample_y = max(y0, min(y1, round(pupil_center[1])))
    color = pixels[sample_x, sample_y]
    return {
        "center": pupil_center,
        "radius": radius,
        "color": color,
    }


def render_variant(
    base_image: Image.Image,
    eyes: list[dict[str, object]],
    delta: tuple[int, int],
) -> Image.Image:
    image = base_image.copy()

    for eye in eyes:
        image = paint_circle(
            image,
            eye["mask"],
            eye["pupil_center"],
            eye["pupil_radius"] + 4,
            eye["white"],
        )

    for eye in eyes:
        center_x, center_y = eye["pupil_center"]
        radius = eye["pupil_radius"]
        bbox = eye["bbox"]
        margin = max(6, round(6 * base_image.size[0] / BASELINE_SIZE))
        next_x = clamp(round(center_x + delta[0]), bbox[0] + radius + margin, bbox[2] - radius - margin)
        next_y = clamp(round(center_y + delta[1]), bbox[1] + radius + margin, bbox[3] - radius - margin)
        image = paint_circle(
            image,
            eye["mask"],
            (next_x, next_y),
            radius,
            eye["pupil_color"],
        )

    return image


def paint_circle(
    image: Image.Image,
    clip_mask: Image.Image,
    center: tuple[float, float],
    radius: int,
    color: tuple[int, int, int, int],
) -> Image.Image:
    circle_mask = Image.new("L", image.size, 0)
    draw = ImageDraw.Draw(circle_mask)
    cx, cy = center
    draw.ellipse(
        (
            round(cx - radius),
            round(cy - radius),
            round(cx + radius),
            round(cy + radius),
        ),
        fill=255,
    )
    masked = ImageChops.multiply(circle_mask, clip_mask)
    overlay = Image.new("RGBA", image.size, color)
    return Image.composite(overlay, image, masked)


def build_paint_mask(
    image_size: tuple[int, int],
    bbox: tuple[int, int, int, int],
    component_mask: list[list[bool]],
) -> Image.Image:
    filled = fill_component_holes(component_mask)
    mask = Image.new("L", image_size, 0)
    mask_pixels = mask.load()
    x0, y0, _, _ = bbox
    for local_y, row in enumerate(filled):
        for local_x, value in enumerate(row):
            if value:
                mask_pixels[x0 + local_x, y0 + local_y] = 255
    return mask


def connected_components(
    mask: list[list[bool]],
    offset: tuple[int, int] = (0, 0),
) -> list[list[tuple[int, int]]]:
    height = len(mask)
    width = len(mask[0]) if height else 0
    seen = [[False] * width for _ in range(height)]
    components: list[list[tuple[int, int]]] = []

    for y in range(height):
        for x in range(width):
            if not mask[y][x] or seen[y][x]:
                continue

            queue = deque([(x, y)])
            seen[y][x] = True
            component: list[tuple[int, int]] = []

            while queue:
                current_x, current_y = queue.popleft()
                component.append((current_x + offset[0], current_y + offset[1]))
                for next_x, next_y in (
                    (current_x + 1, current_y),
                    (current_x - 1, current_y),
                    (current_x, current_y + 1),
                    (current_x, current_y - 1),
                ):
                    if (
                        0 <= next_x < width
                        and 0 <= next_y < height
                        and mask[next_y][next_x]
                        and not seen[next_y][next_x]
                    ):
                        seen[next_y][next_x] = True
                        queue.append((next_x, next_y))

            components.append(component)

    return components


def fill_component_holes(mask: list[list[bool]]) -> list[list[bool]]:
    height = len(mask)
    width = len(mask[0]) if height else 0
    visited = [[False] * width for _ in range(height)]
    outside = [[False] * width for _ in range(height)]
    queue = deque()

    for x in range(width):
        if not mask[0][x]:
            queue.append((x, 0))
        if not mask[height - 1][x]:
            queue.append((x, height - 1))
    for y in range(height):
        if not mask[y][0]:
            queue.append((0, y))
        if not mask[y][width - 1]:
            queue.append((width - 1, y))

    while queue:
        x, y = queue.popleft()
        if not (0 <= x < width and 0 <= y < height):
            continue
        if visited[y][x] or mask[y][x]:
            continue
        visited[y][x] = True
        outside[y][x] = True
        queue.extend(
            (
                (x + 1, y),
                (x - 1, y),
                (x, y + 1),
                (x, y - 1),
            )
        )

    return [
        [mask[y][x] or not outside[y][x] for x in range(width)]
        for y in range(height)
    ]


def centroid(component: list[tuple[int, int]]) -> tuple[float, float]:
    return (
        sum(x for x, _ in component) / len(component),
        sum(y for _, y in component) / len(component),
    )


def clamp(value: int, lower: int, upper: int) -> int:
    return max(lower, min(upper, value))


if __name__ == "__main__":
    raise SystemExit(main())
