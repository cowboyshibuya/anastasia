import React from 'react'

export type DaemonGlyphState =
  | 'idle'
  | 'thinking'
  | 'reading'
  | 'editing'
  | 'executing'
  | 'waiting'
  | 'permission'
  | 'error'
  | 'complete'

interface DaemonGlyphProps {
  state?: DaemonGlyphState
  size?: number
  className?: string
}

const BITMAPS: Record<DaemonGlyphState, number[]> = {
  idle: [
    0b00011000,
    0b00100100,
    0b01000010,
    0b10000001,
    0b10000001,
    0b01000010,
    0b00100100,
    0b00011000,
  ],
  complete: [
    0b00011000,
    0b00111100,
    0b01111110,
    0b11111111,
    0b11111111,
    0b01111110,
    0b00111100,
    0b00011000,
  ],
  thinking: [
    0b00011000,
    0b01010100,
    0b10101010,
    0b01010101,
    0b10101010,
    0b01010101,
    0b00101000,
    0b00011000,
  ],
  reading: [
    0b00011000,
    0b00100100,
    0b11111111,
    0b10000001,
    0b10000001,
    0b11111111,
    0b00100100,
    0b00011000,
  ],
  editing: [
    0b00111100,
    0b01100110,
    0b01100110,
    0b01100110,
    0b01100110,
    0b01100110,
    0b01100110,
    0b00111100,
  ],
  executing: [
    0b11110000,
    0b11110000,
    0b11001100,
    0b11001100,
    0b00110011,
    0b00110011,
    0b00001111,
    0b00001111,
  ],
  waiting: [
    0b11111111,
    0b10000001,
    0b10000001,
    0b10000001,
    0b10000001,
    0b10000001,
    0b10000001,
    0b11111111,
  ],
  permission: [
    0b00111100,
    0b01000010,
    0b01000010,
    0b11111111,
    0b11011011,
    0b11011011,
    0b11111111,
    0b11111111,
  ],
  error: [
    0b10000001,
    0b01000010,
    0b00100100,
    0b00011000,
    0b00011000,
    0b00100100,
    0b01000010,
    0b10000001,
  ],
}

const COLOR_CLASSES: Record<DaemonGlyphState, string> = {
  idle: 'text-[var(--text-ghost)]',
  thinking: 'animate-pulse',
  reading: '',
  editing: '',
  executing: '',
  waiting: 'text-[var(--text-secondary)]',
  permission: 'text-[var(--warning)]',
  error: 'text-[var(--destructive)]',
  complete: '',
}

export function DaemonGlyph({
  state = 'idle',
  size = 12,
  className = '',
}: DaemonGlyphProps) {
  const bitmap = BITMAPS[state] || BITMAPS.idle
  const colorClass = COLOR_CLASSES[state] || COLOR_CLASSES.idle
  const isSignal = ['thinking', 'reading', 'editing', 'executing', 'complete'].includes(state)

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 8 8"
      className={`inline-block shrink-0 ${!isSignal ? 'fill-current' : ''} ${colorClass} ${className}`}
      style={{ imageRendering: 'pixelated', shapeRendering: 'crispEdges' }}
      aria-hidden="true"
    >
      <defs>
        <linearGradient id="daemon-blue-gradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#356FE6" />
          <stop offset="100%" stopColor="#81BEFF" />
        </linearGradient>
      </defs>
      {bitmap.map((row, y) =>
        Array.from({ length: 8 }).map((_, x) => {
          const isSet = (row >> (7 - x)) & 1
          if (!isSet) return null
          return (
            <rect
              key={`${x}-${y}`}
              x={x}
              y={y}
              width={1}
              height={1}
              fill={isSignal ? 'url(#daemon-blue-gradient)' : undefined}
            />
          )
        })
      )}
    </svg>
  )
}
