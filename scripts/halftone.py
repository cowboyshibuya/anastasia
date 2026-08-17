#!/usr/bin/env python3
"""Sample an image into the halftone dot grid the boot splash draws.

One-shot generator: run it when the source art changes, commit the result.

    python3 scripts/halftone.py ~/Desktop/anastasia-logo-3.png

The centered mark (logo-3, the same one the debug icon uses) is the source:
the release logo is a crop that runs off the edge, which has no silhouette to
sample.

Writes `assets/hero-dots.txt` — one digit per cell, `0` (empty) to
`9` (solid), rows top to bottom. The mark is cropped to its ink bounds first,
so every cell in the grid carries signal.

The website draws the same grids from wide illustrations rather than a
silhouette, where the whole frame is the composition and there is nothing to
crop to:

    python3 scripts/halftone.py ~/Desktop/anastasia-background.png \\
      --cols 168 --rows 97 --no-crop --out website/src/art/hero-bird.txt

ponytail: shells out to `sips` for decoding instead of taking a Pillow
dependency for a script that runs about once a year. macOS only; port to
Pillow if this ever needs to run in CI.
"""

import argparse
import struct
import subprocess
import tempfile
from pathlib import Path

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
    repo = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?",
                        default=Path.home() / "Desktop/anastasia-logo-rounded.png")
    parser.add_argument("--cols", type=int, default=46)
    parser.add_argument("--rows", type=int, default=46)
    parser.add_argument("--no-crop", action="store_true",
                        help="sample the full frame instead of the ink bounds")
    parser.add_argument("--out", type=Path, default=repo / "assets/hero-dots.txt")
    args = parser.parse_args()

    cols, rows_out = args.cols, args.rows
    src = Path(args.source).expanduser()
    out = args.out

    with tempfile.TemporaryDirectory() as tmp:
        bmp = Path(tmp) / "logo.bmp"
        # ponytail: no pre-downscale. load_bmp decodes pixel by pixel in
        # Python, but a couple of megapixels is a few seconds and resampling
        # first visibly coarsens the box average at these grid sizes.
        subprocess.run(
            ["sips", "-s", "format", "bmp", str(src), "--out", str(bmp)],
            check=True, stdout=subprocess.DEVNULL,
        )
        width, height, rows = load_bmp(bmp)

    if args.no_crop:
        x0, y0, x1, y1 = 0, 0, width, height
    else:
        x0, y0, x1, y1 = ink_bounds(width, height, rows)
    box_w, box_h = x1 - x0, y1 - y0

    grid = []
    for gy in range(rows_out):
        grid_row = []
        for gx in range(cols):
            # Box-average the source pixels under this cell: partial coverage
            # at the glyph's edges becomes a partial dot, which is what makes
            # the halftone read as a smooth shape rather than a staircase.
            sx0 = x0 + box_w * gx // cols
            sx1 = min(x1, max(sx0 + 1, x0 + box_w * (gx + 1) // cols))
            sy0 = y0 + box_h * gy // rows_out
            sy1 = min(y1, max(sy0 + 1, y0 + box_h * (gy + 1) // rows_out))
            total = sum(rows[y][x] for y in range(sy0, sy1) for x in range(sx0, sx1))
            mean = total / ((sy1 - sy0) * (sx1 - sx0))
            grid_row.append(mean)
        grid.append(grid_row)

    max_mean = max(max(r) for r in grid) or 255.0
    lines = [
        "".join(str(min(9, round(v / max_mean * 9))) for v in r)
        for r in grid
    ]

    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n")
    print(f"wrote {out} ({cols}x{rows_out} from {src.name} box {box_w}x{box_h})")



if __name__ == "__main__":
    main()
