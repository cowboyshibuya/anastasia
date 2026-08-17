#!/usr/bin/env python3
"""Build the macOS .icns pair and the web icons from the Anastasia logo.

One-shot generator: run it when the logo changes, commit the results.

    python3 scripts/appicon.py ~/Desktop/anastasia-logo-rounded.png

Writes `resources/AppIcon.icns` (the logo as-is), `resources/AppIconDev.icns`
(the same mark over a violet ground, so the debug app is tellable from the
release app in the Dock), and the `website/public/*.png` icons.

ponytail: shells out to `sips`/`iconutil` and hand-rolls a ~30-line PNG writer
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

# The debug ground. Dark enough that the white mark still reads, violet enough
# to be unmistakable at Dock size.
DEV_TINT = (86, 58, 214)
# Below this luma a source pixel counts as background and takes the tint;
# above it the pixel is the mark and is left alone.
DEV_TINT_MAX_LUMA = 96

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
    """Write RGB rows as an 8-bit truecolor PNG."""
    height = len(rows)
    width = len(rows[0])
    raw = bytearray()
    for row in rows:
        raw.append(0)  # filter type 0 (None)
        for r, g, b in row:
            raw += bytes((r, g, b))

    def chunk(kind, payload):
        return (
            struct.pack(">I", len(payload))
            + kind
            + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
        )

    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )


def tinted(rows):
    """Replace the near-black ground with DEV_TINT, leaving the mark alone."""
    out = []
    for row in rows:
        new = []
        for r, g, b in row:
            luma = (r * 299 + g * 587 + b * 114) // 1000
            if luma <= DEV_TINT_MAX_LUMA:
                # Keep the ground's own shading by scaling the tint with it, so
                # the rounded corners and any gradient survive.
                scale = 1.0 - (luma / (DEV_TINT_MAX_LUMA * 2))
                new.append(tuple(int(c * scale) for c in DEV_TINT))
            else:
                new.append((r, g, b))
        out.append(new)
    return out


def resize(source, destination, size):
    subprocess.run(
        ["sips", "-s", "format", "png", "-z", str(size), str(size), str(source),
         "--out", str(destination)],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def build_icns(master, destination, work):
    iconset = work / f"{destination.stem}.iconset"
    iconset.mkdir()
    for name, size in ICONSET:
        resize(master, iconset / f"{name}.png", size)
    subprocess.run(
        ["iconutil", "-c", "icns", str(iconset), "-o", str(destination)],
        check=True,
    )
    print(f"wrote {destination.relative_to(ROOT)}")


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: appicon.py <logo.png>")
    source = Path(sys.argv[1]).expanduser()
    if not source.is_file():
        raise SystemExit(f"no such file: {source}")

    with tempfile.TemporaryDirectory() as directory:
        work = Path(directory)

        # Release master: the logo at the largest icns size.
        release = work / "release.png"
        resize(source, release, 1024)

        # Debug master: same mark, violet ground. sips decodes to BMP because
        # the stdlib cannot read PNG.
        bmp = work / "release.bmp"
        subprocess.run(
            ["sips", "-s", "format", "bmp", str(release), "--out", str(bmp)],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        _, _, rows = load_bmp(bmp)
        debug = work / "debug.png"
        write_png(debug, tinted(rows))

        build_icns(release, ROOT / "resources" / "AppIcon.icns", work)
        build_icns(debug, ROOT / "resources" / "AppIconDev.icns", work)

        public = ROOT / "website" / "public"
        if public.is_dir():
            for name, size in WEB_ICONS:
                resize(release, public / name, size)
                print(f"wrote website/public/{name}")


if __name__ == "__main__":
    main()
