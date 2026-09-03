<div align="center">

<img src="assets/logo.svg" width="88" height="88" alt="">

# CRC Code

**A GPU-native code editor with an AI agent built in.**

Rust core, tree-sitter highlighting, `wgpu` renderer, no web view.

[![Rust](https://img.shields.io/badge/Rust-1.96%2B-b7410e?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![wgpu](https://img.shields.io/badge/wgpu-30-7c5cff)](https://wgpu.rs)
[![tree-sitter](https://img.shields.io/badge/tree--sitter-12%20languages-4a7d5f)](https://tree-sitter.github.io)
[![Tests](https://img.shields.io/badge/tests-245%20passing-3d6b50)](#testing)
[![License](https://img.shields.io/badge/license-MIT-2f6ba8)](LICENSE)
[![Status](https://img.shields.io/badge/status-early%20development-b0873f)]()

</div>

---

## What this is

CRC Code is an editor built the way Zed is built rather than the way VS Code is:
every pixel is drawn on the GPU, and the core that touches your disk is Rust.
There is no browser inside it.

The agent is **Claude Code**, not a bespoke autocomplete — the loop, the tools
and the permissions already exist, so the editor's job is to host them well.

> **Early development.** It opens a folder, edits and saves files, and
> highlights twelve languages. See [Status](#status) for what is and is not
> there yet.

## Features

| | |
|---|---|
| **GPU renderer** | Rounded rectangles from a signed-distance field, text through `glyphon`. The whole shell is a handful of draw calls |
| **Light and dark** | Every colour is a role, not a hex code. Contrast is asserted by tests, not eyeballed |
| **12 languages** | C, C++, C#, CSS, HTML, Java, JavaScript, JSON, Python, Rust, TSX, TypeScript — parsed incrementally |
| **Own window chrome** | Borderless, with mac-style controls, drag, edge resize and DPI scaling |
| **Density profiles** | Calm, Balanced and Dense change the whole shell together. Zen hands the window to the code |
| **Sandboxed core** | Every path resolves through the workspace root, so a hallucinated `../../.ssh/id_rsa` fails before it reaches the disk |
| **Collaboration-ready** | Edits carry an author, and undo takes back *your* last edit rather than whatever sits on top of the stack |
| **Pixel-tested UI** | Frames render offscreen and the tests read the pixels back |
| **Editing that keeps up** | Every keystroke re-parses incrementally, so highlighting never lags behind the text |
| **Auto-save** | Written 800 ms after you stop typing, and on focus loss and close |

## Quick start

```bash
git clone https://github.com/superbodik/crc-code
cd crc-code
cargo run -p crc-app             # opens the current folder
cargo run -p crc-app -- ../other # opens another one
cargo run -p crc-app -- --smoke  # draws three frames and exits
```

Needs a Rust toolchain (1.96+) and a GPU with Vulkan, Metal, DX12 or GL.

## Keys

| Key | Action |
|---|---|
| Typing, `Enter`, `Tab`, `Backspace`, `Delete` | Edit the buffer |
| `Ctrl+S` | Save now — otherwise it saves itself |
| `Ctrl+Z` / `Ctrl+Shift+Z` | Undo and redo |
| `Ctrl+A` | Select all |
| Arrows, `Home`, `End`, `PgUp`, `PgDn` | Move the cursor |
| `Ctrl+Left` / `Ctrl+Right` | By word |
| `Ctrl+Home` / `Ctrl+End` | To the start or end of the file |
| `Shift` with any motion | Extend the selection |
| `Ctrl+D` | Light and dark |
| `Ctrl+B` | Explorer |
| `Alt+Z` | Zen — panels go, the code stays |
| `Alt+1` `Alt+2` `Alt+3` | Calm / Balanced / Dense |
| `Esc` / `Ctrl+Q` | Quit |

Click in the buffer to place the caret, drag to select, and the wheel scrolls.
Click a file in the explorer to open it.

The three dots on the left close, minimize and maximize. The title bar drags
the window; double-clicking it maximizes. The edges resize.

## Architecture

One responsibility per file, one subsystem per crate.

| Crate | What it owns |
|---|---|
| [`crc-core`](crates/crc-core) | The workspace: sandboxed filesystem, search, file watching, open documents |
| [`crc-text`](crates/crc-text) | The buffer: rope storage, edits, undo history, selections, authorship |
| [`crc-editor`](crates/crc-editor) | The document: buffer, syntax tree and cursor kept in step |
| [`crc-theme`](crates/crc-theme) | Design tokens: colours by role, type scale, density profiles |
| [`crc-syntax`](crates/crc-syntax) | tree-sitter: incremental parsing, highlight roles, language detection |
| [`crc-ui`](crates/crc-ui) | Layout, the GPU renderer, the frame builder, the window |
| [`crc-app`](crates/crc-app) | The `crc` binary: window, input, workspace session |

Three ideas hold it together.

**The engine is the only door to the disk.** The UI, plugins and agents are all
clients of it, so the workspace sandbox and the size limits hold for every one
of them — there is no second path to guard.

**Colour is a role.** A panel asks for `chrome.border`, never for `#e0dfdb`.
That is what made a dark theme a new table rather than a search through the
drawing code.

**Drawing is a pure function.** `view::draw` turns layout into primitives with
no GPU involved, so the shell is asserted offscreen: each region on its own
colour, ink in the buffer, the caret on the cursor line.

## Status

**Working**

- Workspace opening, gitignore-aware search, file watching
- Editing: typing, selection, word and line motions, undo and redo
- Mouse: click to place the caret, drag to select, wheel to scroll, click to open a file
- Auto-save 800 ms after the last keystroke, on focus loss and on close
- Incremental highlighting across 12 languages, re-parsed per keystroke
- GPU shell: title bar, rail, explorer, tabs, gutter, buffer, minimap, panel, status bar
- Light and dark themes, three density profiles, zen mode
- Borderless window with working controls, drag and resize

**Not yet**

- Multiple tabs, split panes, the command palette
- Find and replace
- LSP: diagnostics, go to definition, completion
- The Claude Code agent panel
- Collaborative editing over the wire (the buffer primitives are in place)

## Testing

```bash
cargo test --workspace
```

245 tests. The interesting ones do not mock: contrast ratios are computed from
the actual palette, grammars are loaded and real snippets parsed, and frames are
rendered on the real GPU and read back pixel by pixel.

## The mark

<img src="assets/logo.svg" width="56" height="56" alt=""> <img src="assets/logo-dark.svg" width="56" height="56" alt=""> <img src="assets/logo-mono.svg" width="56" height="56" alt="">

A C for CRC, a caret beside it because this is an editor, and a cut corner
because the block is a file of code. Everything about it is a percentage of one
side — corner cut 26%, radius 20%, caret 6.5% by 44%, clear space 12% — so the
same numbers drive [the SVG assets](assets/) and the mark the editor draws
itself in its own title bar. Below 24px the cut goes and the caret becomes the
second stroke of the monogram.

It shares its geometry and its lightness with the
[mineDres Team](https://minedres-team.github.io/legal/) mark, and differs only
in hue: `#4AA8FF` against the team's `#00C48F`. Tests hold both halves of that —
the proportions at every size in the ramp, and the two marks reading at the same
weight.

## Built with an agent

Every commit here is pair-written with [Claude Code](https://claude.com/claude-code),
and the history says so — the co-author trailer is on all of it. That is the
point rather than a footnote: an editor that hosts an agent ought to be able to
show what building with one actually looks like.

What it looks like, concretely: the tests are the interesting part. Asserting
contrast ratios caught four unreadable colours in the design before a line of
the renderer existed. Reading pixels back off the GPU caught a scroll window
that silently rendered the whole file. Parsing real snippets caught two
grammars that disagree about which highlight rule wins. None of that is
reachable by reading the diff.

## License

MIT

<div align="center">
<sub>Built by <a href="https://github.com/superbodik">CringeCraft</a></sub>
</div>
