# xeditor

A small GUI text editor built with Rust + Iced.

## Status

Early prototype. Expect rough edges.

## Features

- Split view: file explorer + editor
- Open/save files (via native file dialogs)
- Open a directory and browse a tree (expand/collapse)
- Basic syntax highlighting (Iced highlighter)
- Status bar with file path + cursor position

## Keyboard shortcuts

On macOS use Cmd; on Linux/Windows use Ctrl.

- Cmd/Ctrl+O: open file
- Cmd/Ctrl+Shift+O: open directory
- Cmd/Ctrl+S: save
- Cmd/Ctrl+N: new file

## Run

```bash
cargo run
```

Build a release binary:

```bash
cargo build --release
```

## Nix (optional)

If you use Nix flakes:

```bash
nix develop
cargo run
```

Build with Nix:

```bash
nix build
```

## Notes / troubleshooting

- Linux: you may need system libraries for windowing/GPU (Wayland/X11 + Vulkan/OpenGL). The `flake.nix` dev shell wires common runtime deps.
- There are few deps for the `rfd` file dialog crate.
- If the app launches but shows a blank window, try running with `RUST_LOG=debug` and check for graphics backend/runtime library issues.

## License

See `LICENSE`.
