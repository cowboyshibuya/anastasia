import {
  ArrowUp,
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  Folder,
  GitBranch,
  History,
  Pencil,
  Sparkles,
  Terminal,
  Zap,
} from 'lucide-react'
import type { ReactNode } from 'react'

function Chip({ children }: { children: ReactNode }) {
  return (
    <span className="inline-flex items-center gap-1 rounded-[5px] border border-white/10 bg-white/5 px-2 py-0.5 font-mono text-[9.5px] leading-none text-zinc-400">
      {children}
    </span>
  )
}

function SessionRow({
  title,
  active,
  running,
}: {
  title: string
  active?: boolean
  running?: boolean
}) {
  return (
    <div
      className={`flex items-center gap-1.5 rounded-md px-2 py-[5px] text-[11px] transition-colors ${
        active
          ? 'bg-white/10 text-zinc-100 font-medium'
          : 'text-zinc-400 hover:bg-white/5 hover:text-zinc-300'
      }`}
    >
      <span className="truncate">{title}</span>
      {running && (
        <span className="ml-auto relative flex size-2 shrink-0">
          <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75" />
          <span className="relative inline-flex size-2 rounded-full bg-emerald-500" />
        </span>
      )}
    </div>
  )
}

// A stylized impression of the native Anastasia GPUI window
export function AppWindow() {
  return (
    <div className="overflow-hidden rounded-xl border border-zinc-200/80 bg-[#121316] text-[11.5px] leading-normal text-zinc-300 shadow-[0_25px_70px_-15px_rgba(0,0,0,0.4)] ring-1 ring-black/5 dark:border-white/10 dark:shadow-[0_30px_90px_-20px_rgba(0,0,0,0.8)] dark:ring-white/10 select-none">
      {/* Title bar */}
      <div className="relative flex h-10 items-center justify-between border-b border-white/8 bg-[#18191e] px-4">
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-[7px]">
            <span className="size-[11px] rounded-full bg-[#ff5f57] shadow-[inset_0_0_0_0.5px_rgba(0,0,0,0.25)]" />
            <span className="size-[11px] rounded-full bg-[#febc2e] shadow-[inset_0_0_0_0.5px_rgba(0,0,0,0.25)]" />
            <span className="size-[11px] rounded-full bg-[#28c840] shadow-[inset_0_0_0_0.5px_rgba(0,0,0,0.25)]" />
          </div>
          <div className="ml-3 hidden items-center gap-1.5 text-zinc-400 sm:flex">
            <img
              src="/anastasia-logo-dark.png"
              alt="Anastasia"
              className="size-4 object-contain"
            />
            <span className="font-medium text-zinc-300 text-[11.5px]">Anastasia</span>
            <span className="text-zinc-600">/</span>
            <span className="text-zinc-400">anastasia-app</span>
          </div>
        </div>

        <div className="absolute left-1/2 -translate-x-1/2 hidden md:flex items-center gap-1.5 rounded-full border border-white/6 bg-white/[0.03] px-3 py-0.5 text-[10.5px] text-zinc-400">
          <GitBranch className="size-3 text-zinc-500" />
          <span className="font-mono text-zinc-300">main</span>
          <span className="text-zinc-600">•</span>
          <span className="truncate max-w-[200px]">fix: zero-copy transcript virtualization</span>
        </div>

        <div className="flex items-center gap-2">
          <div className="flex items-center gap-1.5 rounded-md bg-white/5 px-2 py-1 font-mono text-[10px] text-zinc-300">
            <span
              className="provider-mark size-3 text-emerald-400"
              style={{
                maskImage: 'url(/providers/claude.svg)',
                WebkitMaskImage: 'url(/providers/claude.svg)',
              }}
            />
            <span>Claude 3.7</span>
            <span className="size-1.5 rounded-full bg-emerald-400 animate-pulse" />
          </div>
        </div>
      </div>

      <div className="flex h-[460px]">
        {/* Sidebar */}
        <div className="hidden w-52 shrink-0 flex-col justify-between border-r border-white/8 bg-[#14151a] p-2.5 md:flex">
          <div className="flex flex-col gap-1">
            <div className="flex items-center justify-between px-2 pt-1 pb-1 text-[9.5px] font-semibold tracking-[0.14em] text-zinc-500 uppercase">
              <span>Projects</span>
              <span className="text-[9px] text-zinc-600 font-mono">⌘P</span>
            </div>
            
            <div className="flex items-center gap-1.5 rounded px-2 py-1 text-zinc-200 bg-white/5 font-medium">
              <ChevronDown className="size-3 text-zinc-400" />
              <Folder className="size-3.5 text-zinc-400" />
              <span className="truncate">anastasia</span>
            </div>

            <div className="flex flex-col gap-0.5 pl-3.5 mt-0.5">
              <SessionRow title="zero-copy list rendering" active running />
              <SessionRow title="checkpoint git snapshot ref" />
              <SessionRow title="multi-agent timeline sync" />
              <SessionRow title="sparkle auto-updates" />
            </div>

            <div className="mt-2 flex items-center gap-1.5 px-2 py-1 text-zinc-500 hover:text-zinc-300 transition-colors">
              <ChevronRight className="size-3 text-zinc-600" />
              <Folder className="size-3.5 text-zinc-600" />
              <span className="truncate">neural-indexer</span>
            </div>
          </div>

          <div className="rounded-lg border border-white/6 bg-white/[0.02] p-2 text-[10px] text-zinc-500">
            <div className="flex items-center justify-between text-zinc-400 font-medium">
              <span>Native Engine</span>
              <span className="text-emerald-400 flex items-center gap-1">
                <Zap className="size-2.5" /> 120 FPS
              </span>
            </div>
            <div className="mt-1 text-[9px] text-zinc-600 font-mono">GPUI • 0.2% CPU idle</div>
          </div>
        </div>

        {/* Transcript Pane */}
        <div className="flex min-w-0 flex-1 flex-col bg-[#17181e]">
          <div className="flex-1 space-y-3.5 overflow-hidden p-4 sm:p-5">
            {/* User message */}
            <div className="ml-auto w-fit max-w-[85%] rounded-lg rounded-br-xs border border-white/10 bg-white/8 px-3.5 py-2.5 text-zinc-100 shadow-sm">
              The transcript list should render with zero frame drops on large multi-megabyte sessions.
            </div>

            {/* Agent thinking / reasoning snippet */}
            <div className="flex items-start gap-2 text-zinc-400">
              <div className="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full bg-emerald-500/10 text-emerald-400">
                <Sparkles className="size-2.5" />
              </div>
              <p className="max-w-[90%] leading-relaxed text-[11px] text-zinc-400">
                Hoisting git status probes to a background executor pass with generation counters. Virtualizing visible rows with GPUI <code className="rounded bg-white/10 px-1 py-0.5 font-mono text-[10px] text-zinc-200">list()</code> to keep per-frame work proportional only to the viewport:
              </p>
            </div>

            {/* File edit tool card */}
            <div className="overflow-hidden rounded-lg border border-white/8 bg-black/30 shadow-inner">
              <div className="flex items-center gap-2 border-b border-white/6 bg-white/[0.02] px-3 py-1.5">
                <Pencil className="size-3 text-zinc-400" />
                <span className="font-mono font-medium text-[10.5px] text-zinc-200">Edit</span>
                <span className="truncate font-mono text-[10.5px] text-zinc-500">src/transcript_view.rs</span>
                <span className="ml-auto shrink-0 font-mono text-[10px]">
                  <span className="text-emerald-400">+24</span>{' '}
                  <span className="text-red-400/90">−8</span>
                </span>
              </div>
              <div className="p-3 font-mono text-[10.5px] leading-[1.65]">
                <div className="text-red-300/70">
                  <span className="mr-2.5 text-red-500/60 select-none">-</span>
                  let status = run_git_status_sync(&file_path);
                </div>
                <div className="text-emerald-300/90">
                  <span className="mr-2.5 text-emerald-500/60 select-none">+</span>
                  let status = self.status_cache.get(&file_path);
                </div>
                <div className="text-emerald-300/90">
                  <span className="mr-2.5 text-emerald-500/60 select-none">+</span>
                  cx.background_executor().spawn(async move &#123; refresh_status(file_path).await &#125;);
                </div>
              </div>
            </div>

            {/* Terminal command execution */}
            <div className="overflow-hidden rounded-lg border border-white/8 bg-black/30 shadow-inner">
              <div className="flex items-center gap-2 border-b border-white/6 bg-white/[0.02] px-3 py-1.5">
                <Terminal className="size-3 text-zinc-400" />
                <span className="font-mono font-medium text-[10.5px] text-zinc-200">cargo test --lib transcript</span>
                <CheckCircle2 className="ml-auto size-3.5 shrink-0 text-emerald-400" />
              </div>
              <div className="px-3 py-1.5 font-mono text-[10px] text-zinc-400">
                test result: <span className="text-emerald-400 font-semibold">ok</span>. 12 passed; 0 failed; finished in 0.18s
              </div>
            </div>

            {/* Checkpoint info */}
            <div className="flex items-center gap-2 pt-0.5 text-[10.5px] text-zinc-500">
              <History className="size-3.5 text-zinc-400" />
              <span>Working-tree checkpoint created — <kbd className="font-mono text-[9.5px] bg-white/10 px-1 py-0.5 rounded text-zinc-300">⌘Z</kbd> rolls back code & conversation</span>
            </div>
          </div>

          {/* Composer */}
          <div className="m-3.5 mt-0 rounded-lg border border-white/10 bg-[#1e1f26] p-3 shadow-md">
            <div className="text-zinc-500 text-[11px]">
              Ask a question or steer the agent (⏎ queues follow-up, ⌘⏎ steers turn)...
            </div>
            <div className="mt-2.5 flex flex-wrap items-center gap-1.5">
              <Chip>
                <span
                  className="provider-mark size-2.5 text-emerald-400"
                  style={{
                    maskImage: 'url(/providers/claude.svg)',
                    WebkitMaskImage: 'url(/providers/claude.svg)',
                  }}
                />
                Claude 3.7 Sonnet
              </Chip>
              <Chip>Full access</Chip>
              <Chip>Auto-checkpoint</Chip>
              <button
                type="button"
                className="ml-auto flex size-6 items-center justify-center rounded-md bg-zinc-100 text-zinc-900 transition-transform hover:scale-105 active:scale-95"
              >
                <ArrowUp className="size-3.5" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

