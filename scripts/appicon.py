#!/usr/bin/env python3
"""Build the macOS .icns pair and the web icons from the Anastasia logos.

One-shot generator: run it when the logos change, commit the results.

    python3 scripts/appicon.py ~/Desktop/anastasia-logo-2.png \
                               ~/Desktop/anastasia-logo-3.png

The first logo is the release mark, the second the debug one — two distinct
marks so the two apps are tellable apart in the Dock. Writes
`resources/AppIcon.icns`, `resources/AppIconDev.icns`, and the
`website/public/*.png` icons.

Both sources are square art with hard corners. macOS does not round an app
icon for you — a square icon reads as broken next to every other Dock tile —
so the corners are masked to the platform's radius here, with real alpha.

ponytail: shells out to `sips`/`iconutil` and hand-rolls a small PNG writer
instead of taking a Pillow dependency for a script that runs about once a year.
The same sips->BMP decode trick as `scripts/halftone.py`. macOS only.
"""

import struct
import subprocess
import sys
import tempfile
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# macOS icon corner radius as a fraction of the tile — the squircle's radius is
# ~22.37% of its width. A plain rounded rect at that radius is within a pixel or
# two of the real superellipse at every size the Dock draws, and the difference
# is invisible under the icon's own art.
CORNER_RADIUS_RATIO = 0.2237
# Antialiasing samples per axis along the corner arc.
SUPERSAMPLE = 4

# (iconset basename, pixel size) — the set `iconutil` expects.
ICONSET = [
    ("icon_16x16", 16),
    ("icon_16x16@2x", 32),
    ("icon_32x32", 32),
    ("icon_32x32@2x", 64),
    ("icon_128x128", 128),
    ("icon_128x128@2x", 256),
    ("icon_256x256", 256),
    ("icon_256x256@2x", 512),
    ("icon_512x512", 512),
    ("icon_512x512@2x", 1024),
]

# (path under website/public, pixel size)
WEB_ICONS = [
    ("app-icon.png", 256),
    ("apple-touch-icon.png", 180),
    ("favicon.png", 64),
    ("og-icon.png", 512),
]


def load_bmp(path):
    """Decode a 24/32-bit uncompressed BMP into (width, height, RGB rows)."""
    data = path.read_bytes()
    (pixel_offset,) = struct.unpack_from("<I", data, 10)
    width, height, _planes, bpp = struct.unpack_from("<iiHH", data, 18)
    if bpp not in (24, 32):
        raise SystemExit(f"unexpected BMP depth: {bpp}")
    stride = ((width * bpp // 8) + 3) & ~3
    step = bpp // 8
    rows = []
    for y in range(abs(height)):
        start = pixel_offset + y * stride
        row = []
        for x in range(width):
            b, g, r = data[start + x * step : start + x * step + 3]
            row.append((r, g, b))
        rows.append(row)
    # A positive height means the rows are stored bottom-up.
    if height > 0:
        rows.reverse()
    return width, abs(height), rows


def write_png(path, rows):
    """Write RGBA rows (tuples of 4) as an 8-bit truecolor-alpha PNG."""
    height = len(rows)
    width = len(rows[0])
    raw = bytearray()
    for row in rows:
        raw.append(0)  # filter type 0 (None)
        for r, g, b, a in row:
            raw += bytes((r, g, b, a))

    def chunk(kind, payload):
        return (
            struct.pack(">I", len(payload))
            + kind
            + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
        )

    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        # bit depth 8, color type 6 (truecolor + alpha)
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )


def corner_coverage(x, y, width, height, radius):
    """How much of pixel (x, y) falls inside the rounded rect, 0.0..1.0.

    Only the four corner squares need real work; everything else is fully
    inside. Coverage is estimated by supersampling, which is what keeps the
    masked edge from looking like a staircase.
    """
    # Which corner is this pixel near, if any?
    near_left = x < radius
    near_right = x >= width - radius
    near_top = y < radius
    near_bottom = y >= height - radius
    if not ((near_left or near_right) and (near_top or near_bottom)):
        return 1.0

    center_x = radius if near_left else width - radius
    center_y = radius if near_top else height - radius

    inside = 0
    for sub_y in range(SUPERSAMPLE):
        for sub_x in range(SUPERSAMPLE):
            sample_x = x + (sub_x + 0.5) / SUPERSAMPLE
            sample_y = y + (sub_y + 0.5) / SUPERSAMPLE
            # A sample beyond the corner's centre on both axes is in the arc's
            # quadrant; anything else is in the straight part of the edge.
            dx = sample_x - center_x
            dy = sample_y - center_y
            if (near_left and dx > 0) or (near_right and dx < 0):
                dx = 0.0
            if (near_top and dy > 0) or (near_bottom and dy < 0):
                dy = 0.0
            if dx * dx + dy * dy <= radius * radius:
                inside += 1
    return inside / (SUPERSAMPLE * SUPERSAMPLE)


def rounded(rows):
    """Apply the macOS corner mask, returning RGBA rows."""
    height = len(rows)
    width = len(rows[0])
    radius = min(width, height) * CORNER_RADIUS_RATIO
    out = []
    for y in range(height):
        row = []
        for x in range(width):
            r, g, b = rows[y][x]
            coverage = corner_coverage(x, y, width, height, radius)
            row.append((r, g, b, round(coverage * 255)))
        out.append(row)
    return out


def resize(source, destination, size):
    subprocess.run(
        ["sips", "-s", "format", "png", "-z", str(size), str(size), str(source),
         "--out", str(destination)],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def to_bmp(source, destination):
    subprocess.run(
        ["sips", "-s", "format", "bmp", str(source), "--out", str(destination)],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def master(source, work, name):
    """A 1024px, corner-masked PNG master for one logo."""
    square = work / f"{name}-square.png"
    resize(source, square, 1024)
    bmp = work / f"{name}.bmp"
    to_bmp(square, bmp)
    _, _, rows = load_bmp(bmp)
    out = work / f"{name}.png"
    write_png(out, rounded(rows))
    return out


def build_icns(master_png, destination, work):
    iconset = work / f"{destination.stem}.iconset"
    iconset.mkdir()
    for name, size in ICONSET:
        resize(master_png, iconset / f"{name}.png", size)
    subprocess.run(
        ["iconutil", "-c", "icns", str(iconset), "-o", str(destination)],
        check=True,
    )
    print(f"wrote {destination.relative_to(ROOT)}")


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: appicon.py <release-logo.png> <debug-logo.png>")
    release_source = Path(sys.argv[1]).expanduser()
    debug_source = Path(sys.argv[2]).expanduser()
    for source in (release_source, debug_source):
        if not source.is_file():
            raise SystemExit(f"no such file: {source}")

    with tempfile.TemporaryDirectory() as directory:
        work = Path(directory)
        release = master(release_source, work, "release")
        debug = master(debug_source, work, "debug")

        build_icns(release, ROOT / "resources" / "AppIcon.icns", work)
        build_icns(debug, ROOT / "resources" / "AppIconDev.icns", work)

        public = ROOT / "website" / "public"
        if public.is_dir():
            for name, size in WEB_ICONS:
                resize(release, public / name, size)
                print(f"wrote website/public/{name}")


if __name__ == "__main__":
    main()
