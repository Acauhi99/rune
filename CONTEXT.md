# rune — project context

## What it is

`rune` is a terminal UI for everyday Git porcelain. It replaces the command
line for staging, committing, diffing, and browsing git history — think a
minimalist `lazygit` or `gitui`.

## Why it exists

Built as a focused TUI that does one thing well: let you see what changed,
stage what matters, and commit — all without leaving the terminal. Side-by-side
diff with syntax highlighting is the core differentiator.

## Design decisions

- **Side-by-side diff** — the `get_side_by_side` function in `git/diff.rs`
  pairs old/new lines for display. Context lines are mirrored, adds/deletes
  get aligned in pairs.
- **Syntax highlighting** — `syntect` with `base16-ocean.dark` theme,
  applied per-file-extension. Only context lines get highlighted (add/del lines
  use plain red/green).
- **No persistent state** — everything is derived from `git2::Repository` on
  the fly. Refresh (`r`) re-reads status from disk.
- **Dialog stack** — commit message input and branch picker share a single
  `Option<DialogState>` slot. Esc dismisses, Enter confirms.
- **Panel focus** — `PanelFocus::Tree` / `PanelFocus::Diff` controls which
  panel gets keyboard input and cyan border highlight.

## Key types

| Type | Location | Purpose |
|---|---|---|
| `RuneApp` | `app.rs:93` | Mutable app state (files, diff, mode, filter) |
| `ChangedFile` | `app.rs:4` | File path + status + staged flag |
| `FileDiff` | `app.rs:68` | Parsed diff with hunks |
| `DiffLine` | `app.rs:48` | Single diff line (kind, content, line numbers) |
| `Action` | `keybind.rs:4` | Enum of all user actions |

## Data flow

1. `main()` → parse CLI → resolve path → `tui::run(&path)`
2. `tui::run()` → open repo → create `RuneApp` → `refresh_state()` → event loop
3. Events → `map_key()` → `handle_action()` → mutate `app` → redraw
4. `draw()` → render panels based on `app.mode` (Tree/Diff/CommitLog/Help)

## Future ideas

- Merge conflict resolution
- Interactive rebase
- Commit amend
- Stash management
- Custom keybindings config
