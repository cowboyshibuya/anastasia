import { useEffect, useMemo, useState } from 'react'

// Geometry ported from the app's boot splash (`src/ui/splash.rs`), so the site
// and the first frame of the app speak the same visual language.
const PITCH = 8
// Below 1.0 so even a solid cell keeps the gap that makes the field read as a
// halftone rather than a fill.
const DOT_MAX = 0.72
// Rest opacity of the field; the sweep lifts it to full.
const DIM = 0.5
// How many cells in from each edge the field dissolves over.
const FADE_CELLS = 6
// Midtone lift. The splash's mark is nearly all 0 or 9 ink, so it needs none of
// this; a photographic grid sits mostly in the low levels, and since ink drives
// dot diameter *and* opacity, those cells get attenuated twice and vanish.
const GAMMA = 0.6

/** How much of the field survives at `(x, y)`: 1.0 in the middle, ramping to 0
 *  within `FADE_CELLS` of any edge. `bleedBottom` drops the bottom edge from
 *  the ramp so hero art can run off the viewport instead of fading out above
 *  the fold. */
function edgeFalloff(
  x: number,
  y: number,
  columns: number,
  rows: number,
  bleedBottom: boolean,
) {
  const edges = [x, y, columns - 1 - x]
  if (!bleedBottom) edges.push(rows - 1 - y)
  return Math.min(1, Math.max(0, Math.min(...edges) / FADE_CELLS))
}

/** Paint the ink grid once and hand back a PNG data URL. Cell ink drives both
 *  dot diameter and opacity, so shapes dissolve into the surrounding field
 *  instead of ending on a staircase. */
function renderField(rows: string[], bleedBottom: boolean, scale: number) {
  const columns = rows[0]?.length ?? 0
  const canvas = document.createElement('canvas')
  canvas.width = Math.round(columns * PITCH * scale)
  canvas.height = Math.round(rows.length * PITCH * scale)
  const ctx = canvas.getContext('2d')
  if (!ctx) return null
  ctx.scale(scale, scale)
  ctx.fillStyle = '#fff'

  // One path per alpha bucket: ~16 fill() calls instead of one per cell.
  const buckets = new Map<number, Path2D>()
  rows.forEach((line, y) => {
    for (let x = 0; x < line.length; x++) {
      const ink = (line.charCodeAt(x) - 48) / 9
      if (ink <= 0) continue
      const shade = Math.pow(ink, GAMMA)
      const alpha = shade * edgeFalloff(x, y, columns, rows.length, bleedBottom)
      const bucket = Math.round(alpha * 16)
      if (bucket === 0) continue
      let path = buckets.get(bucket)
      if (!path) buckets.set(bucket, (path = new Path2D()))
      const radius = (shade * PITCH * DOT_MAX) / 2
      path.moveTo(x * PITCH + PITCH / 2 + radius, y * PITCH + PITCH / 2)
      path.arc(x * PITCH + PITCH / 2, y * PITCH + PITCH / 2, radius, 0, Math.PI * 2)
    }
  })
  for (const [bucket, path] of buckets) {
    ctx.globalAlpha = bucket / 16
    ctx.fill(path)
  }
  return canvas.toDataURL('image/png')
}

/** An ink grid from `scripts/halftone.py` as a dot field with the boot splash's
 *  travelling light crossing it on the diagonal.
 *
 *  The field is painted once into a PNG and then used twice: as the resting
 *  layer, and as the mask for a moving band of light. Only that band's
 *  `transform` animates, which the compositor owns — masking the light instead
 *  of animating a mask keeps the main thread completely idle. */
export function Halftone({
  grid,
  bleedBottom = false,
  dim = DIM,
  className,
}: {
  grid: string
  bleedBottom?: boolean
  dim?: number
  className?: string
}) {
  const rows = useMemo(() => grid.split('\n').filter(Boolean), [grid])
  const columns = rows[0]?.length ?? 1
  const [art, setArt] = useState<string | null>(null)

  useEffect(() => {
    // Resolve to roughly what will be on screen rather than the grid's nominal
    // pitch: a dpr-3 phone would otherwise paint a 5376px-wide field to display
    // it at 390px. CSS scales the result, so being off costs sharpness only.
    const displayed = window.innerWidth * (window.devicePixelRatio || 1)
    const scale = Math.min(2, Math.max(0.5, displayed / (columns * PITCH)))
    setArt(renderField(rows, bleedBottom, scale))
  }, [rows, columns, bleedBottom])

  const field = art
    ? { backgroundImage: `url(${art})`, backgroundSize: '100% 100%' }
    : undefined

  return (
    <div
      aria-hidden="true"
      className={`relative ${className ?? ''}`}
      style={{ aspectRatio: `${columns} / ${rows.length}` }}
    >
      <div className="absolute inset-0" style={{ ...field, opacity: dim }} />
      <div
        className="halftone-sweep absolute inset-0"
        style={art ? { maskImage: `url(${art})`, WebkitMaskImage: `url(${art})` } : undefined}
      >
        <div className="halftone-light" />
      </div>
    </div>
  )
}
