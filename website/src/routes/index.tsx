import { createFileRoute } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import {
  FALLBACK_DOWNLOAD_URL,
  FALLBACK_VERSION,
  releaseQuery,
} from '@/lib/release'
import { Halftone } from '@/components/halftone'
import bird from '@/art/hero-bird.txt?raw'
import mac from '@/art/hero-mac.txt?raw'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import type { LucideIcon } from 'lucide-react'
import {
  GitBranch,
  History,
  Lock,
  RotateCcw,
  Terminal,
  Zap,
} from 'lucide-react'
import type { ReactNode } from 'react'

export const Route = createFileRoute('/')({
  loader: ({ context }) => {
    void context.queryClient.prefetchQuery(releaseQuery)
  },
  component: Home,
})

const REPO = 'https://github.com/cowboyshibuya/anastasia'

const PROVIDERS = [
  { slug: 'claude', label: 'Claude Code' },
  { slug: 'openai', label: 'Codex' },
  { slug: 'amp', label: 'Amp' },
  { slug: 'cursor', label: 'Cursor' },
  { slug: 'opencode', label: 'OpenCode' },
  { slug: 'grok', label: 'Grok' },
  { slug: 'pi', label: 'Pi' },
]

interface Feature {
  icon: LucideIcon
  title: string
  body: string
}

const FEATURES: Feature[] = [
  {
    icon: Zap,
    title: 'Native GPUI speed',
    body: 'Zero Electron overhead. Anastasia is compiled directly in Rust with Metal hardware acceleration for smooth 120 FPS virtualization.',
  },
  {
    icon: RotateCcw,
    title: 'Instant checkpoint rewind',
    body: 'Working-tree snapshots captured automatically on each agent turn. Rewind both the conversation and your code with one click.',
  },
  {
    icon: Lock,
    title: 'Completely local & private',
    body: 'Anastasia connects directly to local CLIs and your provider keys. No intermediate relays or telemetry leaving your machine.',
  },
  {
    icon: History,
    title: 'Unified agent timeline',
    body: 'Switch seamlessly between Claude Code, Codex, Amp, Cursor, Grok, and Pi while keeping all session history and tool traces in one place.',
  },
  {
    icon: Terminal,
    title: 'Native subprocess executor',
    body: 'Live background command execution, automatic liveness polling, and real-time ANSI terminal rendering with zero UI thread blocking.',
  },
  {
    icon: GitBranch,
    title: 'Multi-branch worktree support',
    body: 'Run separate agent turns on isolated branches concurrently without repository duplication or merge conflicts.',
  },
]

function SectionLabel({ children }: { children: ReactNode }) {
  return (
    <div className="font-mono text-xs text-muted-foreground uppercase tracking-wider">
      {children}
    </div>
  )
}

function GitHubMark() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" className="size-5">
      <path
        fill="currentColor"
        d="M12 2A10 10 0 0 0 2 12c0 4.42 2.87 8.17 6.84 9.5c.5.08.66-.23.66-.5v-1.69c-2.77.6-3.36-1.34-3.36-1.34c-.46-1.16-1.11-1.47-1.11-1.47c-.91-.62.07-.6.07-.6c1 .07 1.53 1.03 1.53 1.03c.87 1.52 2.34 1.07 2.91.83c.09-.65.35-1.09.63-1.34c-2.22-.25-4.55-1.11-4.55-4.92c0-1.11.38-2 1.03-2.71c-.1-.25-.45-1.29.1-2.64c0 0 .84-.27 2.75 1.02c.79-.22 1.65-.33 2.5-.33s1.71.11 2.5.33c1.91-1.29 2.75-1.02 2.75-1.02c.55 1.35.2 2.39.1 2.64c.65.71 1.03 1.6 1.03 2.71c0 3.82-2.34 4.66-4.57 4.91c.36.31.69.92.69 1.85V21c0 .27.16.59.67.5C19.14 20.16 22 16.42 22 12A10 10 0 0 0 12 2"
      />
    </svg>
  )
}

function Home() {
  const { data: release } = useQuery(releaseQuery)
  const version = release?.version ?? FALLBACK_VERSION
  const downloadUrl = release?.url ?? FALLBACK_DOWNLOAD_URL

  return (
    <TooltipProvider>
      <div className="grain min-h-dvh antialiased">
        <div className="mx-auto w-full max-w-[1100px] border-border/70 md:border-x">
          {/* Header matching Waku */}
          <header className="flex h-16 items-center justify-between px-5 md:px-10">
            <a
              href="/"
              className="flex items-center gap-2.5 outline-none focus-visible:ring-2 focus-visible:ring-ring/60"
            >
              <img
                src="/anastasia-logo-dark.png"
                alt=""
                className="size-7 object-contain"
              />
              <span className="text-[15px] font-semibold tracking-tight">
                Anastasia
              </span>
            </a>
            <div className="flex items-center gap-5">
              <a
                href={REPO}
                target="_blank"
                rel="noreferrer"
                aria-label="GitHub"
                className="rounded-full text-muted-foreground transition-colors outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring/60"
              >
                <GitHubMark />
              </a>
              <a
                href={downloadUrl}
                className="inline-flex h-8 items-center justify-center rounded-md bg-foreground px-3.5 text-xs font-medium text-background transition-opacity hover:opacity-90 outline-none focus-visible:ring-2 focus-visible:ring-ring/60"
              >
                Download
              </a>
            </div>
          </header>

          <main>
            {/* Hero with Halftone Crow Background Art inside */}
            {/* <section className="relative overflow-hidden px-5 pt-14 pb-14 md:px-10 md:pt-20 md:pb-20">

              <div className="pointer-events-none absolute inset-0 z-0 flex items-center justify-center overflow-hidden">
                <div className="w-full max-w-[1100px]">
                  <Halftone grid={bird} dim={0.7} className="w-full" />
                </div>
                <div className="absolute inset-0 bg-gradient-to-b from-background/30 via-transparent to-background/90" />
              </div>

              <div className="relative z-10">
                <h1 className="max-w-4xl text-4xl font-semibold tracking-[-0.03em] text-balance md:text-[3.4rem] md:leading-[1.04]">
                  One native app for all your coding agents.
                </h1>
                <p className="mt-5 max-w-[36rem] text-[17px] leading-relaxed text-pretty text-muted-foreground">
                  Anastasia drives the agent CLIs you already have — sessions,
                  transcripts, tool activity, and checkpoints in one fast
                  window, entirely on your machine.
                </p> 
                <div className="mt-8 flex flex-wrap items-center gap-x-5 gap-y-3">
                  <a
                    href={downloadUrl}
                    className="inline-flex h-10 items-center justify-center rounded-md bg-foreground px-5 text-sm font-medium text-background transition-opacity hover:opacity-90 outline-none focus-visible:ring-2 focus-visible:ring-ring/60"
                  >
                    Download for macOS
                  </a>
                  {/* <span className="font-mono text-xs text-muted-foreground">
                    v{version}
                  </span>
                </div>

                <div className="mt-16">
                  <SectionLabel>Drives the agents you already use</SectionLabel>
                  <div className="mt-4 flex flex-wrap items-center gap-x-7 gap-y-4">
                    {PROVIDERS.map((p) => (
                      <Tooltip key={p.slug}>
                        <TooltipTrigger
                          render={
                            <button
                              type="button"
                              aria-label={p.label}
                              className="cursor-default rounded-sm text-muted-foreground/70 transition-colors outline-none hover:text-foreground focus-visible:text-foreground focus-visible:ring-2 focus-visible:ring-ring/60"
                            />
                          }
                        >
                          <span
                            className="provider-mark size-[22px]"
                            style={{
                              maskImage: `url(/providers/${p.slug}.svg)`,
                              WebkitMaskImage: `url(/providers/${p.slug}.svg)`,
                            }}
                          />
                        </TooltipTrigger>
                        <TooltipContent>{p.label}</TooltipContent>
                      </Tooltip>
                    ))}
                  </div>
                </div>
              </div>
            </section>  */}

            {/* Features */}
            {/* <section className="border-t border-border/70">
              <div className="px-5 pt-14 md:px-10">
                <SectionLabel>Why native</SectionLabel>
              </div>
              <div className="mt-8 grid grid-cols-1 gap-px border-t border-border/70 bg-border/70 sm:grid-cols-2 lg:grid-cols-3">
                {FEATURES.map((f) => (
                  <div key={f.title} className="bg-background p-6 md:p-8">
                    <div className="flex items-center gap-2.5">
                      <f.icon className="size-4 text-muted-foreground" />
                      <h3 className="text-sm font-medium">{f.title}</h3>
                    </div>
                    <p className="mt-2.5 text-sm leading-relaxed text-muted-foreground">
                      {f.body}
                    </p>
                  </div>
                ))}
              </div>
            </section> */}

            {/* Centered Download Section with Halftone Mac perfectly fitting below */}
            <section
              id="download"
              className="relative border-t border-border/70 pt-20 pb-0 text-center md:pt-24 flex flex-col items-center justify-center overflow-hidden"
            >
              {/* Title & Action controls revealed cleanly above */}
              <div className="relative z-10 flex flex-col items-center justify-center max-w-xl mx-auto px-5">
                {/* <SectionLabel>Download</SectionLabel> */}
                <h2 className="mt-3 text-3xl font-semibold tracking-tight sm:text-4xl">
                  One native app for all your coding agents.
                </h2>
                <p className="mt-3 text-sm leading-relaxed text-muted-foreground max-w-md">
                  Anastasia drives the agent CLIs you already have : sessions,
                  transcripts, tool activity, and checkpoints in one fast
                  window, entirely on your machine.                </p>
                <div className="mt-7 flex flex-wrap items-center justify-center gap-x-4 gap-y-3">
                  <a
                    href={downloadUrl}
                    className="inline-flex h-10 items-center justify-center rounded-md bg-foreground px-6 text-sm font-medium text-background transition-opacity hover:opacity-90 outline-none focus-visible:ring-2 focus-visible:ring-ring/60 shadow-lg"
                  >
                    Download for macOS
                  </a>
                  {/* <span className="font-mono text-xs text-muted-foreground">
                    v{version}
                  </span> */}
                </div>
                {/* <p className="mt-4 font-mono text-xs text-muted-foreground">
                  Apple Silicon (macOS 13+) · Linux and x86_64 from source
                </p> */}
              </div>

              {/* Halftone Mac illustration perfectly fitting the container */}
              <div className="relative z-0 w-full max-w-[1100px]">
                <Halftone grid={mac} dim={0.9} className="w-full" />
              </div>
            </section>
          </main>

          {/* Footer matching Waku */}
          <footer className="flex items-center justify-between border-t border-border/70 px-5 py-10 text-xs text-muted-foreground md:px-10">
            <div className="flex items-center gap-2">
              <img
                src="/anastasia-logo-dark.png"
                alt=""
                className="size-4 opacity-80"
              />
              <span>© {new Date().getFullYear()} Anastasia</span>
            </div>
            <div className="flex items-center gap-5">
              <a
                href={REPO}
                target="_blank"
                rel="noreferrer"
                className="hover:text-foreground transition-colors"
              >
                GitHub
              </a>
              <a
                href={`${REPO}/blob/main/CHANGELOG.md`}
                target="_blank"
                rel="noreferrer"
                className="hover:text-foreground transition-colors"
              >
                Changelog
              </a>
            </div>
          </footer>
        </div>
      </div>
    </TooltipProvider>
  )
}
