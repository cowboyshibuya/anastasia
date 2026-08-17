# Third-party notices

Anastasia bundles or derives from the components below. Anastasia itself is
GPL-3.0-only; see [LICENSE](LICENSE).

## Waku

<https://github.com/egoist/waku> — GPL-3.0-only, © EGOIST.

Anastasia is a fork of Waku and retains its architecture, daemon, provider
drivers and settings. Modifications are summarized in [README.md](README.md).

## Comet / Zeron

<https://github.com/zeronsh/comet> — MIT, © Wing.

Portions of the interface derive from Comet: the motion kit and pulse clock
(`src/ui/motion.rs`), the pane width tween, the editable keymap and its
settings page, the notification chimes and their preferences, and the boot
splash treatment.

```
MIT License

Copyright (c) 2026 Wing

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Solar Icons

<https://www.figma.com/community/file/1166831539721848736> — CC BY 4.0, ©
480 Design.

Interface glyphs in `assets/icons/` are from the Solar icon set, used under
the Creative Commons Attribution 4.0 International license.

## Geist and Geist Mono

<https://github.com/vercel/geist-font> — SIL Open Font License 1.1, © Vercel.

Bundled in `assets/fonts/`. Geist is the interface typeface; Geist Mono is used
for code and shortcut chips. Full license text: `assets/fonts/OFL-geist.txt`.

## JetBrains Mono

<https://github.com/JetBrains/JetBrainsMono> — SIL Open Font License 1.1, ©
The JetBrains Mono Project Authors.

Bundled in `assets/fonts/`, used by the terminal emulator. Full license text:
`assets/fonts/OFL.txt`.

## Symbols Nerd Font

<https://github.com/ryanoasis/nerd-fonts> — MIT, © Ryan L McIntyre.

Bundled in `assets/fonts/`, supplying the terminal's file-type glyphs. Full
license text: `assets/fonts/LICENSE-nerd-fonts.txt`.

## GPUI

<https://github.com/zed-industries/zed> — Apache-2.0, © Zed Industries.

Anastasia builds against the `egoist/zed` fork of GPUI pinned in `Cargo.toml`.

## Rust dependencies

Every crate in `Cargo.lock` carries its own license. Generate the full
inventory with `cargo license` or `cargo about`.
