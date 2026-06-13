# rune

TUI Git diff viewer and porcelain — inspect staged/unstaged changes, view
commit history, stage files, and commit from inside a terminal UI.

## Features

- **File tree** — list changed files with status icons (+/~/-/&gt;/?)
- **Side-by-side diff** — syntax-highlighted diff panel with scroll
- **Stage/unstage** — toggle individual files, stage all, unstage all
- **Commit** — write message and commit from a dialog
- **Branch switching** — pick a local branch from an interactive list
- **Commit log** — browse recent commits and inspect their diffs
- **Filter** — `/` to filter the file tree by path
- **Keyboard + mouse** — full keyboard navigation and mouse scroll/click

## Install

```bash
cargo install --path .
```

Requires Rust 2024 edition (stable toolchain).

## Usage

```bash
rune            # open current directory
rune .          # same as above
rune /path/to/repo   # open a specific repo
```

## Keybindings

| Key | Action |
|---|---|
| `↑`/`↓` or `j`/`k` | Move selection |
| `Tab` | Cycle panel focus (tree ↔ diff) |
| `Enter` | View diff / select file |
| `Backspace` | Go back |
| `s` | Stage/unstage file |
| `S` | Stage all files |
| `u` | Unstage all files |
| `c` | Commit (enter message) |
| `l` | Toggle commit log |
| `b` | Switch branch |
| `r` | Refresh |
| `/` | Filter file tree |
| `?` | Toggle help |
| `PageUp` / `PageDown` | Scroll diff |
| `q` or `Esc` | Quit |
| `Ctrl+C` | Quit |

## Requirements

- Rust &ge; 1.85 (edition 2024)
- `libgit2` (vendored via `git2` crate)
- Terminal with raw-mode support (most modern terminals)
