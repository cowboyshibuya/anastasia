#!/usr/bin/env python3
"""Sample the Anastasia logo into the halftone dot grid the boot splash draws.

One-shot generator: run it when the logo changes, commit the result.

    python3 scripts/halftone.py ~/Desktop/anastasia-logo-3.png

The centered mark (logo-3, the same one the debug icon uses) is the source:
the release logo is a crop that runs off the edge, which has no silhouette to
sample.

Writes `assets/hero-dots.txt` — one digit per cell, `0` (empty) to
`9` (solid), rows top to bottom. The mark is cropped to its ink bounds first,
so every cell in the grid carries signal.

ponytail: shells out to `sips` for decoding instead of taking a Pillow
dependency for a script that runs about once a year. macOS only; port to
Pillow if this ever needs to run in CI.
"""

import struct
import subprocess
import sys
import tempfile
from pathlib import Path

COLS = 46
ROWS = 46
# Ink threshold, 0..255. The logo is a white mark on near-black, so anything
# above this is glyph.
INK = 40


def load_bmp(path):
    """Decode a 24/32-bit uncompressed BMP into (width, height, luma rows)."""
    data = path.read_bytes()
    (pixel_offset,) = struct.unpack_from("<I", data, 10)
    width, height, _planes, bpp = struct.unpack_from("<iiHH", data, 18)
    if bpp not in (24, 32):
        raise SystemExit(f"unexpected BMP depth: {bpp}")
    stride = ((width * bpp // 8) + 3) & ~3
    step = bpp // 8
    rows = []
    for y in range(abs(height)):
        # Positive height means the rows are stored bottom-up.
        src = y if height < 0 else abs(height) - 1 - y
        base = pixel_offset + src * stride
        row = []
        for x in range(width):
            b, g, r = data[base + x * step : base + x * step + 3]
            row.append((r * 299 + g * 587 + b * 114) // 1000)
        rows.append(row)
    return width, abs(height), rows


def ink_bounds(width, height, rows):
    xs = [x for y in range(height) for x in range(width) if rows[y][x] > INK]
    ys = [y for y in range(height) for x in range(width) if rows[y][x] > INK]
    if not xs:
        raise SystemExit("no ink found — is the mark light-on-dark?")
    return min(xs), min(ys), max(xs) + 1, max(ys) + 1


def main():
    src = Path(sys.argv[1] if len(sys.argv) > 1 else
               Path.home() / "Desktop/anastasia-logo-rounded.png").expanduser()
    out = Path(__file__).resolve().parent.parent / "assets/hero-dots.txt"

    with tempfile.TemporaryDirectory() as tmp:
        bmp = Path(tmp) / "logo.bmp"
        subprocess.run(
            ["sips", "-s", "format", "bmp", str(src), "--out", str(bmp)],
            check=True, stdout=subprocess.DEVNULL,
        )
        width, height, rows = load_bmp(bmp)

    x0, y0, x1, y1 = ink_bounds(width, height, rows)
    box_w, box_h = x1 - x0, y1 - y0

    lines = []
    for gy in range(ROWS):
        line = []
        for gx in range(COLS):
            # Box-average the source pixels under this cell: partial coverage
            # at the glyph's edges becomes a partial dot, which is what makes
            # the halftone read as a smooth shape rather than a staircase.
            sx0 = x0 + box_w * gx // COLS
            sx1 = max(sx0 + 1, x0 + box_w * (gx + 1) // COLS)
            sy0 = y0 + box_h * gy // ROWS
            sy1 = max(sy0 + 1, y0 + box_h * (gy + 1) // ROWS)
            total = sum(rows[y][x] for y in range(sy0, sy1) for x in range(sx0, sx1))
            mean = total / ((sy1 - sy0) * (sx1 - sx0))
            line.append(str(min(9, round(mean / 255 * 9))))
        lines.append("".join(line))

    out.write_text("\n".join(lines) + "\n")
    print(f"wrote {out} ({COLS}x{ROWS} from {src.name} ink box {box_w}x{box_h})")


if __name__ == "__main__":
    main()
