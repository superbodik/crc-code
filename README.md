<div align="center">

# ⌘ CRC Code

**A GPU-native code editor with an AI agent built in.**

Rust core, tree-sitter highlighting, `wgpu` renderer, no web view.

[![Rust](https://img.shields.io/badge/Rust-1.96%2B-b7410e?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![wgpu](https://img.shields.io/badge/wgpu-30-7c5cff)](https://wgpu.rs)
[![tree-sitter](https://img.shields.io/badge/tree--sitter-12%20languages-4a7d5f)](https://tree-sitter.github.io)
[![Tests](https://img.shields.io/badge/tests-180%20passing-3d6b50)](#-testing)
[![License](https://img.shields.io/badge/license-MIT-2f6ba8)](LICENSE)
[![Status](https://img.shields.io/badge/status-early%20development-b0873f)]()

</div>

---

## ✦ What this is

CRC Code is an editor built the way Zed is built rather than the way VS Code is:
every pixel is drawn on the GPU, and the core that touches your disk is Rust.
There is no browser inside it.

The agent is **Claude Code**, not a bespoke autocomplete — the loop, the tools
and the permissions already exist, so the editor's job is to host them well.

> ⚠️ **Early development.** The shell renders, the workspace opens, code is
> highlighted. Typing into the buffer is not wired up yet — see
> [Status](#-status) for exactly what works.

## ⚡ Features

| | |
|---|---|
| 🎨 **GPU renderer** | Rounded rectangles from a signed-distance field, text through `glyphon`. The whole shell is a handful of draw calls |
| 🌗 **Light and dark** | Every colour is a role, not a hex code. Contrast is asserted by tests, not eyeballed |
| 🧩 **12 languages** | C, C++, C#, CSS, HTML, Java, JavaScript, JSON, Python, Rust, TSX, TypeScript — parsed incrementally |
| 🪟 **Own window chrome** | Borderless, with mac-style controls, drag, edge resize and DPI scaling |
| 🧘 **Density profiles** | Calm, Balanced and Dense change the whole shell together. Zen hands the window to the code |
| 🔒 **Sandboxed core** | Every path resolves through the workspace root, so a hallucinated `../../.ssh/id_rsa` fails before it reaches the disk |
| 👥 **Collaboration-ready** | Edits carry an author, and undo takes back *your* last edit rather than whatever sits on top of the stack |
| 🧪 **Pixel-tested UI** | Frames render offscreen and the tests read the pixels back |

## 🚀 Quick start

```bash
git clone https://github.com/superbodik/crc-code
cd crc-code
cargo run -p crc-app             # opens the current folder
cargo run -p crc-app -- ../other # opens another one
cargo run -p crc-app -- --smoke  # draws three frames and exits
```

Needs a Rust toolchain (1.96+) and a GPU with Vulkan, Metal, DX12 or GL.

## ⌨️ Keys

| Key | Action |
|---|---|
| `Ctrl+D` | Light ⇄ dark |
| `Ctrl+B` | Explorer |
| `Alt+Z` | Zen — panels go, the code stays |
| `1` `2` `3` | Calm / Balanced / Dense |
| `↑` `↓` `PgUp` `PgDn` | Move the cursor |
| `Esc` / `Ctrl+Q` | Quit |

The three dots on the left close, minimize and maximize. The title bar drags
the window; double-clicking it maximizes. The edges resize.

## 🏗 Architecture

One responsibility per file, one subsystem per crate.

| Crate | What it owns |
|---|---|
| [`crc-core`](crates/crc-core) | The workspace: sandboxed filesystem, search, file watching, open documents |
| [`crc-text`](crates/crc-text) | The buffer: rope storage, edits, undo history, selections, authorship |
| [`crc-theme`](crates/crc-theme) | Design tokens: colours by role, type scale, density profiles |
| [`crc-syntax`](crates/crc-syntax) | tree-sitter: incremental parsing, highlight roles, language detection |
| [`crc-ui`](crates/crc-ui) | Layout, the GPU renderer, the frame builder, the window |
| [`crc-app`](crates/crc-app) | The `crc` binary: window, input, workspace session |

Three ideas hold it together:

**The engine is the only door to the disk.** The UI, plugins and agents are all
clients of it, so the workspace sandbox and the size limits hold for every one
of them — there is no second path to guard.

**Colour is a role.** A panel asks for `chrome.border`, never for `#e0dfdb`.
That is what made a dark theme a new table rather than a search through the
drawing code.

**Drawing is a pure function.** `view::draw` turns layout into primitives with
no GPU involved, so the shell is asserted offscreen: each region on its own
colour, ink in the buffer, the caret on the cursor line.

## 📋 Status

**Working**

- ✅ Workspace opening, gitignore-aware search, file watching
- ✅ Rope buffer with per-author undo and multi-cursor edits
- ✅ Incremental highlighting across 12 languages
- ✅ GPU shell: title bar, rail, explorer, tabs, gutter, buffer, minimap, panel, status bar
- ✅ Light and dark themes, three density profiles, zen mode
- ✅ Borderless window with working controls, drag and resize

**Not yet**

- ⬜ Typing into the buffer — the buffer renders but keys do not reach it
- ⬜ Mouse selection, wheel scrolling, opening a file from the explorer
- ⬜ Auto-save, LSP, the command palette
- ⬜ The Claude Code agent panel
- ⬜ Collaborative editing over the wire (the buffer primitives are in place)

## 🧪 Testing

```bash
cargo test --workspace
```

180 tests. The interesting ones do not mock: contrast ratios are computed from
the actual palette, grammars are loaded and real snippets parsed, and frames are
rendered on the real GPU and read back pixel by pixel.

## 📄 License

MIT

<div align="center">
<sub>Built by <a href="https://github.com/superbodik">CringeCraft</a></sub>
</div>
