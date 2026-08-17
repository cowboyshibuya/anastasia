# Changelog

All notable changes to Anastasia. This file is the **source of truth for the release
notes shown in the in-app updater**: [`scripts/release.ts`](scripts/release.ts)
extracts the section whose heading matches the version being released
(`MARKETING_VERSION`) and publishes it next to the update, so Sparkle shows it in
the update prompt.

Format follows [Keep a Changelog](https://keepachangelog.com). Add a new
`## [<version>]` section at the top for each release, matching the version you
set in the Xcode project.

Write release notes for the final product users receive, not the development
history. When a feature is still unreleased, fold its fixes and refinements into
the original feature bullet instead of adding separate entries for them.

## [unreleased]

## [0.3.1]

- Anastasia now updates itself. New versions are downloaded and installed from
  within the app, verified against Anastasia's signing key before they run.
  **Check for Updates…** is in the app menu, and Settings → General controls
  whether it checks on its own.

  If you installed 0.3.0, download 0.3.1 by hand once — 0.3.0 shipped before the
  updater existed and cannot reach it. Every version after this one updates in
  place.

## [0.3.0]

Anastasia is rebuilt on Waku, replacing the previous foundation.

- Dark by default, on a near-black plane the interface is designed for rather
  than a charcoal the content competes with. One accent colour now marks the
  active thing; the gauge and resize handle no longer paint their own.
- Editable keyboard shortcuts in Settings. `⌘,` opens settings, `⌘B` and `⌘I`
  toggle the sidebars, `⌘J` the terminal, and `⇧⇥` switches Plan and Build.
  Recording rejects a combination another shortcut owns, and says which.
- Notification sounds and desktop banners, each with its own switch, announcing
  only when a run finishes or the agent asks a question — and by default only
  while Anastasia is in the background.
- The sidebars glide open and closed instead of snapping, by pointer or by
  keyboard, and hold still under reduce-motion.
- A halftone boot mark on launch.
- A roomier composer with type sized for writing in.
- English only for now.

## [0.1.0]

- Add standalone Anastasia daemon and browser client
- Add Linux support (X11 and Wayland, you need to build from source for now)
- Answer agent questions directly in the composer
- Redesign queued follow-ups as composer cards with per-message steering
- Add DeepSeek agent preset selection (Standard, Code, Minimal, and Creator)
- Add Claude context window and ultracode effort options
- Add /fast command to toggle fast mode for Codex
- Show the latest activity in live transcript headers
- Add soft wrapping and keyboard copy feedback
- Add terminal overlay scrollbar and measure cell width from the font
- Restore window position, size, and display across launches
- Contain wheel scrolling in activity and command output viewports
- Smooth streaming markdown and reduce CPU usage while streaming

## [0.0.13]

- Add DeepSeek Harness provider
- Render user message as Markdown and linkify bare URLs
- Share one resident OpenCode serve per workspace across sessions

## [0.0.12]

- Inherit the login-shell environment for provider commands
- Fix model traits across provider switches
- Keep branch change counts current and include untracked files
- Normalize SIGCHLD for provider children
- Fix Grok model discovery

## [0.0.11]

- Fix provider detection for CLIs installed through shell PATH managers such as
  nvm and fnm
- Show models registered by Pi extensions
- Fix the model picker closing when entering a space in search
- Fix duplicate transcript history and lost interaction mode when resuming ACP
  sessions

## [0.0.10]

- Fix crash in due to IME composition
- Fix typo

## [0.0.9]

- Add OpenCode Go support in usage popover
- Fix app icon
- Fix Cursor model detection

## [0.0.8]

- Initial release
