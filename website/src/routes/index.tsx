import { createFileRoute } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { Menu } from '@base-ui/react/menu'
import {
  Activity,
  ArrowRight,
  Check,
  ChevronRight,
  Command,
  Cpu,
  Download,
  HardDrive,
  History,
  Layers,
  Moon,
  RefreshCw,
  Sparkles,
  Sun,
  Zap,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from '@/components/ui/accordion'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { FALLBACK_DOWNLOAD_URL, releaseQuery } from '@/lib/release'
import { useTheme } from '@/lib/theme'
import { AppWindow } from '@/components/app-window'
import type { ReactNode } from 'react'

export const Route = createFileRoute('/')({
  loader: ({ context }) => {
    void context.queryClient.prefetchQuery(releaseQuery)
  },
  component: Home,
})

const PROVIDERS = [
  { slug: 'claude', label: 'Claude Code' },
  { slug: 'openai', label: 'Codex' },
  { slug: 'amp', label: 'Amp' },
  { slug: 'cursor', label: 'Cursor' },
  { slug: 'opencode', label: 'OpenCode' },
  { slug: 'grok', label: 'Grok' },
  { slug: 'pi', label: 'Pi' },
]

const FEATURES = [
  {
    icon: Zap,
    title: 'Native down to the frame',
    body: 'Rust and GPUI — the GPU-accelerated engine powering Zed. Instant launch, smooth 120 FPS scrolling through large transcripts, and zero Electron overhead.',
  },
  {
    icon: Layers,
    title: 'Every agent, one timeline',
    body: 'Connects directly to your local agent CLIs via stream-json, JSON-RPC, and live event pipes, normalized into one unified provider-neutral workspace.',
  },
  {
    icon: History,
    title: 'Atomic rewind that means it',
    body: 'Every turn snapshots your working tree under a hidden git ref. Roll back file edits and the AI conversation context simultaneously with ⌘Z.',
  },
  {
    icon: Command,
    title: 'Keyboard-first ergonomics',
    body: '⌘N starts a session, ⌘P switches projects, ⏎ queues follow-ups while the agent works, ⌘⏎ steers mid-turn, and Escape stops execution.',
  },
  {
    icon: HardDrive,
    title: 'Local by architecture',
    body: 'Sessions, transcripts, tool activity, and project logs live strictly on your disk. No telemetry, no middleman cloud, and no extra account required.',
  },
  {
    icon: RefreshCw,
    title: 'Quietly current',
    body: 'Signed, Apple-notarized, and seamlessly auto-updated with binary deltas via Sparkle so Anastasia stays current without friction.',
  },
]

const COMPARISONS = [
  {
    feature: 'Engine',
    anastasia: 'Native Rust + GPUI (Metal GPU)',
    traditional: 'Chromium / Electron Web Engine',
  },
  {
    feature: 'Idle Memory',
    anastasia: '~45 MB',
    traditional: '400 MB – 1.2 GB',
  },
  {
    feature: 'Input Latency',
    anastasia: 'Sub-millisecond direct draw',
    traditional: '16–50 ms DOM paint loop',
  },
  {
    feature: 'Timeline Virtualization',
    anastasia: 'Zero-copy list() renderer',
    traditional: 'DOM node reflow & virtualization lag',
  },
  {
    feature: 'Undo & Rewind',
    anastasia: 'Atomic git working-tree ref snapshots',
    traditional: 'Chat history only (code left dirty)',
  },
]

const FAQ = [
  {
    q: 'How does Anastasia differ from standard web or Electron coding clients?',
    a: 'Anastasia is built from the ground up in native Rust and GPUI (the GPU framework powering Zed). Every pixel is rendered directly by Metal on your Mac. It launches instantaneously, consumes minimal memory, and maintains 120 FPS under immense transcripts without browser engine bloat.',
  },
  {
    q: 'Do I need separate API keys or cloud subscriptions for Anastasia?',
    a: 'No. Anastasia detects and drives the agent CLIs already installed and authenticated on your machine (Claude Code, Codex, Amp, Cursor, OpenCode, Grok, Pi). Your existing logins, subscription plans, and rate limits apply directly.',
  },
  {
    q: 'How does working-tree checkpoint rewind work?',
    a: 'Before each agent turn executes, Anastasia creates a lightweight snapshot of your current git working tree under a private ref. If an agent goes down the wrong path or makes breaking modifications, pressing ⌘Z rolls back both the code edits and the conversation state.',
  },
  {
    q: 'Where does my source code and session data live?',
    a: 'Exclusively on your machine. Anastasia never uploads your code to third-party relay servers or telemetry hubs. All sessions, transcripts, and checkpoints reside in your local project directories.',
  },
  {
    q: 'What platforms are currently supported?',
    a: 'Anastasia is optimized for macOS (Apple Silicon arm64). Linux support can be built directly from source, and Windows support is on the roadmap.',
  },
]

function SectionLabel({ children }: { children: ReactNode }) {
  return (
    <div className="flex items-center gap-2 font-mono text-[11px] font-medium tracking-[0.16em] text-muted-foreground uppercase">
      <span className="size-1.5 rounded-full bg-emerald-500/80" />
      {children}
    </div>
  )
}

function DownloadMenu({
  downloadUrl,
  size,
  align,
  className,
  showIcon = false,
}: {
  downloadUrl: string
  size: 'sm' | 'lg'
  align: 'start' | 'end'
  className?: string
  showIcon?: boolean
}) {
  const itemClassName =
    'flex h-8 cursor-pointer items-center rounded-md px-2.5 text-sm outline-none transition-colors data-highlighted:bg-accent data-highlighted:text-accent-foreground data-disabled:pointer-events-none data-disabled:opacity-40'

  return (
    <Menu.Root>
      <Menu.Trigger render={<Button size={size} className={className} />}>
        {showIcon && <Download data-icon="inline-start" className="size-4" />}
        <span>Download for macOS</span>
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Positioner
          side="bottom"
          sideOffset={6}
          align={align}
          className="isolate z-50"
        >
          <Menu.Popup className="min-w-56 origin-(--transform-origin) rounded-lg border bg-popover p-1 text-popover-foreground shadow-xl outline-none data-[side=bottom]:slide-in-from-top-1 data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95">
            <Menu.LinkItem
              href={downloadUrl}
              closeOnClick
              className={itemClassName}
            >
              <span className="font-medium">macOS</span>
              <span className="ml-auto text-xs text-muted-foreground">Apple Silicon</span>
            </Menu.LinkItem>
            <Menu.Item disabled className={itemClassName}>
              <span>Linux</span>
              <span className="ml-auto text-[11px] text-muted-foreground">from source</span>
            </Menu.Item>
            <Menu.Item disabled className={itemClassName}>
              <span>Windows</span>
              <span className="ml-auto text-[11px] text-muted-foreground">planned</span>
            </Menu.Item>
          </Menu.Popup>
        </Menu.Positioner>
      </Menu.Portal>
    </Menu.Root>
  )
}

function ThemeToggle() {
  const { theme, toggleTheme } = useTheme()

  return (
    <button
      type="button"
      onClick={toggleTheme}
      aria-label="Toggle theme"
      className="flex size-8 items-center justify-center rounded-lg border border-border/80 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring/60 outline-none"
    >
      <Sun className="size-4 hidden dark:block transition-transform" />
      <Moon className="size-4 block dark:hidden transition-transform" />
    </button>
  )
}

function Home() {
  const { data: release } = useQuery(releaseQuery)
  const downloadUrl = release?.url ?? FALLBACK_DOWNLOAD_URL

  return (
    <TooltipProvider>
      <div className="min-h-dvh antialiased bg-dot-grid">
        <div className="mx-auto w-full max-w-[1140px] border-border/70 bg-background/95 backdrop-blur-sm md:border-x shadow-2xl">
          {/* Header */}
          <header className="sticky top-0 z-40 flex h-16 items-center justify-between border-b border-border/70 bg-background/90 px-5 backdrop-blur-md md:px-10">
            <a href="/" className="flex items-center gap-2.5 group">
              <div className="relative flex size-8 items-center justify-center rounded-lg bg-zinc-900 dark:bg-zinc-800 p-1 ring-1 ring-white/10 transition-transform group-hover:scale-105">
                <img
                  src="/anastasia-logo-dark.png"
                  alt="Anastasia logo"
                  className="size-6 object-contain"
                />
              </div>
              <div className="flex flex-col">
                <span className="text-[16px] font-semibold tracking-tight leading-none">
                  Anastasia
                </span>
                <span className="font-mono text-[9px] text-muted-foreground tracking-widest uppercase">
                  Native Agent Client
                </span>
              </div>
            </a>

            <div className="flex items-center gap-3 md:gap-4">
              <nav className="hidden items-center gap-5 text-[13px] font-medium text-muted-foreground md:flex">
                <a href="#features" className="transition-colors hover:text-foreground">
                  Features
                </a>
                <a href="#architecture" className="transition-colors hover:text-foreground">
                  Architecture
                </a>
                <a href="#faq" className="transition-colors hover:text-foreground">
                  FAQ
                </a>
              </nav>

              <div className="h-4 w-px bg-border hidden md:block" />

              <a
                href="https://github.com/cowboyshibuya/anastasia"
                target="_blank"
                rel="noreferrer"
                aria-label="GitHub"
                className="flex size-8 items-center justify-center rounded-lg border border-border/80 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground outline-none"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 24 24"
                  className="size-4"
                >
                  <path
                    fill="currentColor"
                    d="M12 2A10 10 0 0 0 2 12c0 4.42 2.87 8.17 6.84 9.5c.5.08.66-.23.66-.5v-1.69c-2.77.6-3.36-1.34-3.36-1.34c-.46-1.16-1.11-1.47-1.11-1.47c-.91-.62.07-.6.07-.6c1 .07 1.53 1.03 1.53 1.03c.87 1.52 2.34 1.07 2.91.83c.09-.65.35-1.09.63-1.34c-2.22-.25-4.55-1.11-4.55-4.92c0-1.11.38-2 1.03-2.71c-.1-.25-.45-1.29.1-2.64c0 0 .84-.27 2.75 1.02c.79-.22 1.65-.33 2.5-.33s1.71.11 2.5.33c1.91-1.29 2.75-1.02 2.75-1.02c.55 1.35.2 2.39.1 2.64c.65.71 1.03 1.6 1.03 2.71c0 3.82-2.34 4.66-4.57 4.91c.36.31.69.92.69 1.85V21c0 .27.16.59.67.5C19.14 20.16 22 16.42 22 12A10 10 0 0 0 12 2"
                  />
                </svg>
              </a>

              <ThemeToggle />

              <DownloadMenu
                downloadUrl={downloadUrl}
                size="sm"
                align="end"
                className="h-8 text-xs font-medium"
              />
            </div>
          </header>

          <main>
            {/* Hero */}
            <section className="relative px-5 pt-14 pb-16 md:px-10 md:pt-20 md:pb-24">
              <div className="inline-flex items-center gap-2 rounded-full border border-border/80 bg-muted/60 px-3 py-1 text-xs text-muted-foreground mb-6 backdrop-blur-xs">
                <span className="relative flex size-2">
                  <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75" />
                  <span className="relative inline-flex size-2 rounded-full bg-emerald-500" />
                </span>
                <span className="font-mono font-medium tracking-wide">ANASTASIA 1.0</span>
                <span className="text-border">•</span>
                <span>Native AI Coding Agent Workbench</span>
              </div>

              <h1 className="max-w-4xl text-4xl font-semibold tracking-[-0.035em] text-balance md:text-[3.6rem] md:leading-[1.05]">
                One native app for all your coding agents.
              </h1>

              <p className="mt-5 max-w-[38rem] text-[17px] leading-relaxed text-pretty text-muted-foreground">
                Anastasia connects directly to the agent CLIs you already have — sessions,
                transcripts, tool activity, and git working-tree checkpoints in one lightning-fast
                native window. 100% on your machine.
              </p>

              <div className="mt-8 flex flex-wrap items-center gap-x-5 gap-y-3">
                <DownloadMenu
                  downloadUrl={downloadUrl}
                  size="lg"
                  className="h-11 px-5 text-[14px] font-medium shadow-lg shadow-primary/10 hover:shadow-primary/20 transition-all"
                  align="start"
                  showIcon
                />
                {release && (
                  <div className="flex items-center gap-2 rounded-md border border-border/80 bg-muted/30 px-2.5 py-1.5 font-mono text-xs text-muted-foreground">
                    <span className="font-medium text-foreground">v{release.version}</span>
                    <span className="text-border">•</span>
                    <span>Apple Silicon</span>
                  </div>
                )}
              </div>

              {/* Providers bar */}
              <div className="mt-14 border-t border-border/60 pt-6">
                <SectionLabel>Drives the coding agents you already use</SectionLabel>
                <div className="mt-4 flex flex-wrap items-center gap-x-8 gap-y-4">
                  {PROVIDERS.map((p) => (
                    <Tooltip key={p.slug}>
                      <TooltipTrigger
                        render={
                          <div
                            aria-label={p.label}
                            className="group flex items-center gap-2 rounded-md py-1 text-muted-foreground transition-colors hover:text-foreground cursor-default"
                          >
                            <span
                              className="provider-mark size-5 opacity-75 group-hover:opacity-100 transition-opacity"
                              style={{
                                maskImage: `url(/providers/${p.slug}.svg)`,
                                WebkitMaskImage: `url(/providers/${p.slug}.svg)`,
                              }}
                            />
                            <span className="text-xs font-medium">{p.label}</span>
                          </div>
                        }
                      />
                      <TooltipContent>{p.label} native CLI interface</TooltipContent>
                    </Tooltip>
                  ))}
                </div>
              </div>
            </section>

            {/* Showcase App Window */}
            <section className="px-3 pb-16 md:px-8 md:pb-24">
              <div className="relative">
                <div className="absolute -inset-2 rounded-2xl bg-gradient-to-b from-primary/10 via-transparent to-transparent blur-xl pointer-events-none" />
                <AppWindow />
              </div>

              <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-3">
                <div className="flex items-center gap-3 rounded-lg border border-border/70 bg-card/60 p-3.5 backdrop-blur-xs">
                  <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-emerald-500/10 text-emerald-500">
                    <Zap className="size-4" />
                  </div>
                  <div className="text-xs leading-snug">
                    <div className="font-medium text-foreground">Metal GPU Accelerated</div>
                    <div className="text-muted-foreground text-[11px]">120 FPS high-refresh timeline</div>
                  </div>
                </div>

                <div className="flex items-center gap-3 rounded-lg border border-border/70 bg-card/60 p-3.5 backdrop-blur-xs">
                  <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-sky-500/10 text-sky-500">
                    <History className="size-4" />
                  </div>
                  <div className="text-xs leading-snug">
                    <div className="font-medium text-foreground">Working-Tree Checkpoints</div>
                    <div className="text-muted-foreground text-[11px]">Rewind code & chat together</div>
                  </div>
                </div>

                <div className="flex items-center gap-3 rounded-lg border border-border/70 bg-card/60 p-3.5 backdrop-blur-xs">
                  <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-amber-500/10 text-amber-500">
                    <HardDrive className="size-4" />
                  </div>
                  <div className="text-xs leading-snug">
                    <div className="font-medium text-foreground">Zero Cloud Proxy</div>
                    <div className="text-muted-foreground text-[11px]">Direct local CLI process pipes</div>
                  </div>
                </div>
              </div>
            </section>

            {/* Features */}
            <section id="features" className="border-t border-border/70">
              <div className="px-5 pt-14 md:px-10">
                <SectionLabel>Engineered for Speed</SectionLabel>
                <h2 className="mt-2 text-2xl font-semibold tracking-tight md:text-3xl">
                  Built natively for high-velocity coding.
                </h2>
              </div>
              <div className="mt-8 grid grid-cols-1 gap-px border-t border-border/70 bg-border/70 sm:grid-cols-2 lg:grid-cols-3">
                {FEATURES.map((f) => (
                  <div
                    key={f.title}
                    className="bg-card p-6 md:p-8 transition-colors hover:bg-muted/40"
                  >
                    <div className="flex items-center gap-2.5">
                      <f.icon className="size-4 text-emerald-500" />
                      <h3 className="text-sm font-semibold tracking-tight">{f.title}</h3>
                    </div>
                    <p className="mt-3 text-sm leading-relaxed text-muted-foreground">
                      {f.body}
                    </p>
                  </div>
                ))}
              </div>
            </section>

            {/* Architecture / Performance Comparison */}
            <section id="architecture" className="border-t border-border/70 px-5 py-16 md:px-10 md:py-20">
              <SectionLabel>Architecture</SectionLabel>
              <h2 className="mt-2 text-2xl font-semibold tracking-tight md:text-3xl">
                Native Rust + GPUI vs Web Wrappers
              </h2>
              <p className="mt-3 max-w-[34rem] text-sm text-muted-foreground">
                Anastasia eliminates the Chromium/DOM layers entirely, talking straight to the GPU
                and your local filesystem.
              </p>

              <div className="mt-8 overflow-hidden rounded-xl border border-border/80 bg-card">
                <div className="grid grid-cols-3 border-b border-border/80 bg-muted/60 px-4 py-3 font-mono text-[11px] font-semibold tracking-wider text-muted-foreground uppercase">
                  <div>Capability</div>
                  <div className="text-foreground">Anastasia (Native)</div>
                  <div>Typical Web / Electron App</div>
                </div>
                {COMPARISONS.map((item, idx) => (
                  <div
                    key={item.feature}
                    className={`grid grid-cols-3 px-4 py-3.5 text-xs transition-colors hover:bg-muted/30 ${
                      idx !== COMPARISONS.length - 1 ? 'border-b border-border/60' : ''
                    }`}
                  >
                    <div className="font-medium text-foreground">{item.feature}</div>
                    <div className="font-medium text-emerald-500 dark:text-emerald-400 flex items-center gap-1.5">
                      <Check className="size-3.5 shrink-0" />
                      <span>{item.anastasia}</span>
                    </div>
                    <div className="text-muted-foreground">{item.traditional}</div>
                  </div>
                ))}
              </div>
            </section>

            {/* Download CTA Banner */}
            <section id="download" className="border-t border-border/70 px-5 py-16 md:px-10 md:py-20 bg-muted/20">
              <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-6">
                <div>
                  <SectionLabel>Ready to experience native speed?</SectionLabel>
                  <h2 className="mt-2 text-3xl font-semibold tracking-tight md:text-4xl">
                    Get Anastasia for Mac
                  </h2>
                  <p className="mt-2 text-sm text-muted-foreground max-w-md">
                    Universal Apple Silicon build with automatic Sparkle binary delta updates.
                  </p>
                </div>
                <div className="flex flex-wrap items-center gap-4">
                  <DownloadMenu
                    downloadUrl={downloadUrl}
                    size="lg"
                    className="h-12 px-6 text-sm font-medium shadow-lg"
                    align="start"
                    showIcon
                  />
                </div>
              </div>
            </section>

            {/* FAQ */}
            <section id="faq" className="border-t border-border/70 px-5 py-16 md:px-10 md:py-20">
              <SectionLabel>Frequently Asked Questions</SectionLabel>
              <h2 className="mt-2 text-2xl font-semibold tracking-tight md:text-3xl">
                Everything you need to know.
              </h2>
              <Accordion className="mt-8 max-w-3xl">
                {FAQ.map((item) => (
                  <AccordionItem key={item.q} value={item.q} className="border-border/70">
                    <AccordionTrigger className="text-[15px] font-medium hover:no-underline py-4">
                      {item.q}
                    </AccordionTrigger>
                    <AccordionContent className="text-sm leading-relaxed text-muted-foreground pb-4">
                      {item.a}
                    </AccordionContent>
                  </AccordionItem>
                ))}
              </Accordion>
            </section>
          </main>

          {/* Footer */}
          <footer className="flex flex-col sm:flex-row items-center justify-between gap-4 border-t border-border/70 px-5 py-10 text-xs text-muted-foreground md:px-10 bg-muted/10">
            <div className="flex items-center gap-3">
              <img
                src="/anastasia-logo-dark.png"
                alt="Anastasia"
                className="size-5 object-contain"
              />
              <span className="font-semibold text-foreground">Anastasia</span>
              <span className="text-border">•</span>
              <span>© {new Date().getFullYear()} Anastasia. Open Source.</span>
            </div>

            <div className="flex items-center gap-6">
              <a
                href="https://github.com/cowboyshibuya/anastasia"
                target="_blank"
                rel="noreferrer"
                className="hover:text-foreground transition-colors"
              >
                GitHub
              </a>
              <a
                href="https://github.com/cowboyshibuya/anastasia/blob/main/CHANGELOG.md"
                target="_blank"
                rel="noreferrer"
                className="hover:text-foreground transition-colors"
              >
                Changelog
              </a>
              <span className="font-mono text-[11px] text-muted-foreground/80">
                macOS Sonoma & Sequoia
              </span>
            </div>
          </footer>
        </div>
      </div>
    </TooltipProvider>
  )
}

