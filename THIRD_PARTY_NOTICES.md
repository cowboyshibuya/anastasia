# Third-party notices

## Comet / Zeron

Anastasia is derived from Comet (Zeron), copyright (c) 2026 Wing, under the
MIT License. The original copyright and permission notice are retained in
`LICENSE` and Git history.

## Fonts and icons

- Geist and Geist Mono are copyright Vercel Inc. and licensed under the SIL
  Open Font License 1.1.
- Solar Icons are by 480 Design and licensed under CC BY 4.0.

## Syntax highlighting

Anastasia bundles the following syntax-highlighting components. Their parsers and
queries are consumed from the pinned Rust crates listed in `Cargo.lock`.

| Component | Version | License | Source |
| --- | --- | --- | --- |
| Tree-sitter | 0.26.11 | MIT | https://github.com/tree-sitter/tree-sitter |
| Tree-sitter highlight | 0.26.11 | MIT | https://github.com/tree-sitter/tree-sitter |
| Tree-sitter Rust grammar and queries | 0.24.2 | MIT | https://github.com/tree-sitter/tree-sitter-rust |
| Tree-sitter JavaScript grammar and queries | 0.25.0 | MIT | https://github.com/tree-sitter/tree-sitter-javascript |
| Tree-sitter TypeScript grammar and queries | 0.23.2 | MIT | https://github.com/tree-sitter/tree-sitter-typescript |
| Tree-sitter Python, Go, JSON, Bash, HTML, CSS, C, C++, C#, Java, Ruby and PHP grammars and queries | pinned in `Cargo.lock` | MIT | https://github.com/tree-sitter |
| Tree-sitter TOML, Markdown, YAML, Kotlin, Swift, SQL, Lua, Nix, Make and Containerfile grammars and queries | pinned in `Cargo.lock` | MIT-compatible; see each crate | Crate repositories recorded in `Cargo.lock` |

The full Anastasia distribution remains licensed under the terms in `LICENSE`.
