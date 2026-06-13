# Agents — rune

## Stack

- Rust 2024 edition (stable toolchain)
- **CLI**: `clap` with derive macros
- **TUI**: `ratatui` + `crossterm` (terminal raw mode, alternate screen)
- **Git**: `git2` (libgit2 bindings, vendored)
- **Diff**: `similar` crate for diff algorithms
- **Syntax highlighting**: `syntect` (Sublime Text syntax defs)

## Architecture

```
main.rs         — entry point, CLI arg parsing
app.rs          — domain types: RuneApp, ChangedFile, FileDiff, AppMode
git/
  mod.rs        — repo ops: open, stage, commit, branches, status
  diff.rs       — diff extraction: workdir, staged, commit diff
tui/
  mod.rs        — main loop, event dispatch, draw
  panels/
    tree.rs     — file tree list
    diff.rs     — syntax-highlighted side-by-side diff
    commitlog.rs— commit history list
    dialog.rs   — commit message input, branch picker
keybind.rs      — key → Action mapping
```

## Conventions

- `anyhow::Result` for fallible functions
- `git2::Repository` passed as `&Repository` through the event loop
- State lives in `RuneApp` (mutable, passed to event handlers)
- `#[allow(dead_code)]` on enums used only for pattern matching
- Panel rendering via `ratatui::Frame` with `Borders::ALL` blocks
- Focused panel gets cyan border, highlighted list items get dark-gray bg

## Testing

- `cargo test` — unit tests
- `cargo clippy -- -D warnings` — lint (must pass before push)
- `cargo fmt --check` — formatting check (must pass before commit)

## Development

```bash
cargo run -- .              # run in current directory
cargo install --path .      # install as `rune` binary
lefthook run pre-commit     # run hooks manually
lefthook run pre-push
```
