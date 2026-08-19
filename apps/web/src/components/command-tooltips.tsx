import { useEffect, useState } from 'react'

interface TooltipTarget {
  id: string
  label: string
  shortcut?: string
  x: number
  y: number
  width: number
  height: number
  placement: 'top' | 'bottom'
}

export function CommandTooltips() {
  const [active, setActive] = useState(false)
  const [targets, setTargets] = useState<TooltipTarget[]>([])

  useEffect(() => {
    let holdTimer: number | null = null
    let isHolding = false

    function scanTooltips() {
      const items: TooltipTarget[] = []
      const elements = document.querySelectorAll<HTMLElement>(
        'button, [role="button"], [role="tab"], [role="menuitem"], a, [data-tooltip], [title], [aria-label]'
      )

      elements.forEach((el, index) => {
        // Skip hidden or off-screen elements
        const rect = el.getBoundingClientRect()
        if (
          rect.width === 0 ||
          rect.height === 0 ||
          rect.top < 0 ||
          rect.left < 0 ||
          rect.bottom > window.innerHeight ||
          rect.right > window.innerWidth ||
          window.getComputedStyle(el).display === 'none' ||
          window.getComputedStyle(el).visibility === 'hidden' ||
          window.getComputedStyle(el).opacity === '0'
        ) {
          return
        }

        const label =
          el.getAttribute('data-tooltip') ||
          el.getAttribute('title') ||
          el.getAttribute('aria-label')

        if (!label || label.trim().length === 0) return

        // Extract keyboard shortcut if embedded e.g. " (⌘K)"
        let cleanLabel = label.trim()
        let shortcut: string | undefined

        const shortcutMatch = cleanLabel.match(/\((?:⌘|Ctrl\+|Alt\+|Shift\+|[A-Z0-9⇧⌥⌃⌘])+\)/i)
        if (shortcutMatch) {
          shortcut = shortcutMatch[0].replace(/[()]/g, '')
          cleanLabel = cleanLabel.replace(shortcutMatch[0], '').trim()
        }

        const placement = rect.top < 40 ? 'bottom' : 'top'

        items.push({
          id: `tooltip-${index}-${rect.top}-${rect.left}`,
          label: cleanLabel,
          shortcut,
          x: rect.left + rect.width / 2,
          y: placement === 'top' ? rect.top - 6 : rect.bottom + 6,
          width: rect.width,
          height: rect.height,
          placement,
        })
      })

      setTargets(items)
    }

    function handleKeyDown(event: KeyboardEvent) {
      if ((event.key === 'Meta' || event.key === 'Control') && !isHolding && !event.repeat) {
        isHolding = true
        holdTimer = window.setTimeout(() => {
          scanTooltips()
          setActive(true)
        }, 350)
      }
    }

    function handleKeyUp(event: KeyboardEvent) {
      if (event.key === 'Meta' || event.key === 'Control') {
        isHolding = false
        if (holdTimer !== null) {
          window.clearTimeout(holdTimer)
          holdTimer = null
        }
        setActive(false)
      }
    }

    function handleBlur() {
      isHolding = false
      if (holdTimer !== null) {
        window.clearTimeout(holdTimer)
        holdTimer = null
      }
      setActive(false)
    }

    window.addEventListener('keydown', handleKeyDown)
    window.addEventListener('keyup', handleKeyUp)
    window.addEventListener('blur', handleBlur)

    return () => {
      window.removeEventListener('keydown', handleKeyDown)
      window.removeEventListener('keyup', handleKeyUp)
      window.removeEventListener('blur', handleBlur)
      if (holdTimer !== null) window.clearTimeout(holdTimer)
    }
  }, [])

  if (!active || targets.length === 0) return null

  return (
    <div
      aria-hidden="true"
      className="pointer-events-none fixed inset-0 z-[9999] overflow-hidden animate-in fade-in duration-150"
    >
      {targets.map((target) => (
        <div
          key={target.id}
          className="absolute -translate-x-1/2 flex items-center gap-1.5 rounded-[4px] border border-[#356FE6]/60 bg-[var(--card)] px-2 py-0.5 text-[11px] font-mono text-foreground shadow-[0_4px_16px_rgba(0,0,0,0.5),0_0_8px_rgba(53,111,230,0.25)]"
          style={{
            left: `${target.x}px`,
            top: target.placement === 'top' ? undefined : `${target.y}px`,
            bottom:
              target.placement === 'top'
                ? `${window.innerHeight - target.y}px`
                : undefined,
            maxWidth: '260px',
          }}
        >
          <span className="truncate">{target.label}</span>
          {target.shortcut && (
            <kbd className="rounded-[2px] border border-border/60 bg-[var(--inset)] px-1 py-px text-[9.5px] font-semibold text-[var(--signal)]">
              {target.shortcut}
            </kbd>
          )}
        </div>
      ))}
    </div>
  )
}
